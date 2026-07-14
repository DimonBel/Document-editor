//! Rate-limit middleware: Redis token-bucket per (authenticated user or
//! hashed client IP).
//!
//! Per issue #142: the limiter MUST use the authenticated principal as
//! the key (not a spoofable `X-Forwarded-For`); only fall back to
//! `X-Forwarded-For` when the request is unauthenticated, hashing the
//! IP so we never store raw client IPs at rest. Login + refresh + WS
//! + SSE all get their own buckets. Redis failures log + fail closed
//! (refuse) rather than fail open.

use axum::{
    extract::{Request, State},
    http::{header, HeaderName, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use redis::AsyncCommands;
use sha2::{Digest, Sha256};

use crate::error::ProblemDetails;
use crate::security::middleware::CurrentUser;
use crate::state::AppState;

const HEADER_LIMIT: &str = "x-ratelimit-remaining";
const HEADER_RESET: &str = "x-ratelimit-reset";

/// Buckets: per-prefix `(capacity, refill_per_sec, kind)` where `kind`
/// selects which `RATE_PREFIX` to apply.
const RATE_AUTH:    &str = "auth";      // 10 req / 60 sec
const RATE_WRITE:   &str = "write";     // 60 req / 60 sec
const RATE_READ:    &str = "read";      // 200 req / 60 sec
const RATE_REALTIME:&str = "realtime";  // 30 req / 60 sec
const RATE_WS:      &str = "ws";        // 60 req / 60 sec

fn rate_for_path(path: &str, method: &axum::http::Method) -> Option<(u32, u32, &'static str)> {
    if path.starts_with("/auth/") {
        return Some((10, 60, RATE_AUTH));
    }
    if path.starts_with("/api/realtime/") {
        return Some((30, 60, RATE_REALTIME));
    }
    if path.starts_with("/ws/") {
        return Some((60, 60, RATE_WS));
    }
    if path.starts_with("/api/v1/") {
        let is_write = matches!(*method, axum::http::Method::POST | axum::http::Method::PUT | axum::http::Method::PATCH | axum::http::Method::DELETE);
        return Some(if is_write { (60, 60, RATE_WRITE) } else { (200, 60, RATE_READ) });
    }
    None
}

pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let path = req.uri().path();
    let method = req.method().clone();
    let (capacity, refill_per_sec, kind) = match rate_for_path(path, &method) {
        Some(t) => t,
        None => return next.run(req).await,
    };

    // Identity: prefer the authenticated user; fall back to a hashed
    // client IP (read from `X-Forwarded-For` and **hashed** with
    // SHA-256 so we never persist raw IPs in Redis).
    let key = match req.extensions().get::<CurrentUser>().cloned() {
        Some(u) => format!("u:{}", u.id),
        None => {
            let ip = req
                .headers()
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.split(',').next())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "anon".into());
            let mut h = Sha256::new();
            h.update(ip.as_bytes());
            format!("ip:{:x}", h.finalize()[..8].iter().fold(0u64, |acc, b| (acc << 8) | *b as u64))
        }
    };

    let bucket = chrono::Utc::now().timestamp() as u32 / refill_per_sec.max(1);
    let full_key = format!("rl:{kind}:{key}:{bucket}");

    let mut conn = match state.redis.get().await {
        Ok(c) => c,
        Err(e) => {
            // Fail closed: a busted rate-limiter must not let traffic through unrestricted.
            tracing::error!(error = %e, "rate-limit redis error; refusing request");
            return rate_limit_error(path);
        }
    };

    let count: u32 = match conn.incr(&full_key, 1u32).await {
        Ok(n) => n,
        Err(e) => {
            tracing::error!(error = %e, "rate-limit incr error; refusing request");
            return rate_limit_error(path);
        }
    };
    if count == 1 {
        let _: Result<(), _> = conn.expire(&full_key, 60).await;
    }

    if count > capacity {
        return rate_limit_response(path, refill_per_sec);
    }

    let mut resp = next.run(req).await;
    let remaining = capacity.saturating_sub(count);
    if let (Ok(n), Ok(r)) = (
        HeaderValue::from_str(remaining.to_string().as_str()),
        HeaderValue::from_str(refill_per_sec.to_string().as_str()),
    ) {
        resp.headers_mut().insert(HeaderName::from_static(HEADER_LIMIT), n);
        resp.headers_mut().insert(HeaderName::from_static(HEADER_RESET), r);
    }
    resp
}

fn rate_limit_response(path: &str, refill_per_sec: u32) -> Response {
    let problem = ProblemDetails {
        kind: "https://docs.ed/errors/rate-limited".into(),
        title: "Rate limit exceeded".into(),
        status: 429,
        detail: None,  // generic; never leak the bucket identity
        instance: Some(path.to_string()),
    };
    let mut resp = (StatusCode::TOO_MANY_REQUESTS, axum::Json(problem)).into_response();
    if let Ok(v) = HeaderValue::from_str(refill_per_sec.to_string().as_str()) {
        resp.headers_mut().insert(header::RETRY_AFTER, v);
    }
    resp.headers_mut().insert(HeaderName::from_static(HEADER_LIMIT), HeaderValue::from_static("0"));
    resp
}

fn rate_limit_error(path: &str) -> Response {
    rate_limit_response(path, 60)
}
