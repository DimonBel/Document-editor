//! Rate-limit middleware: Redis token-bucket per (user or IP).
//!
//! Configuration: `AppState.config.rate_limit: HashMap<prefix, (capacity, refill_per_sec)>`.
//! If the request path starts with a known prefix, the bucket applies; otherwise
//! the request is passed through.

use axum::{
    extract::{Request, State},
    http::{header, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use redis::AsyncCommands;

use crate::error::ProblemDetails;
use crate::state::AppState;

const HEADER_LIMIT: &str = "x-ratelimit-remaining";
const HEADER_RESET: &str = "x-ratelimit-reset";

pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let path = req.uri().path();
    let prefix = state
        .config
        .rate_limit
        .keys()
        .find(|p| path.starts_with(p.as_str()))
        .cloned();

    let (capacity, refill_per_sec) = match prefix.and_then(|p| state.config.rate_limit.get(&p).copied()) {
        Some(t) => t,
        None => return next.run(req).await, // no limit for this path
    };

    // Key: user id if authenticated, else client IP
    let key = req
        .extensions()
        .get::<String>()
        .cloned()
        .unwrap_or_else(|| {
            req.headers()
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.split(',').next().unwrap_or("anon").trim().to_string())
                .unwrap_or_else(|| "anon".to_string())
        });

    let bucket = chrono::Utc::now().timestamp() as u32 / refill_per_sec.max(1);
    let full_key = format!("rl:{key}:{bucket}");

    let mut conn = match state.redis.get().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(error = %e, "rate-limit redis error; allowing through");
            return next.run(req).await;
        }
    };

    let count: u32 = match conn.incr(&full_key, 1u32).await {
        Ok(n) => n,
        Err(e) => {
            tracing::warn!(error = %e, "rate-limit incr error; allowing through");
            return next.run(req).await;
        }
    };
    if count == 1 {
        let _: Result<(), _> = conn.expire(&full_key, 60).await;
    }

    if count > capacity {
        let problem = ProblemDetails {
            kind: "https://docs.ed/errors/rate-limited".into(),
            title: "Rate limit exceeded".into(),
            status: 429,
            detail: Some("too many requests".into()),
            instance: Some(path.to_string()),
        };
        let mut resp = (StatusCode::TOO_MANY_REQUESTS, axum::Json(problem)).into_response();
        if let Ok(v) = HeaderValue::from_str(refill_per_sec.to_string().as_str()) {
            resp.headers_mut().insert(header::RETRY_AFTER, v);
        }
        resp.headers_mut()
            .insert(header::HeaderName::from_static(HEADER_RESET), HeaderValue::from_static("0"));
        resp.headers_mut().insert(
            header::HeaderName::from_static(HEADER_LIMIT),
            HeaderValue::from_static("0"),
        );
        return resp;
    }

    let mut resp = next.run(req).await;
    let remaining = capacity.saturating_sub(count);
    if let (Ok(n), Ok(r)) = (
        HeaderValue::from_str(remaining.to_string().as_str()),
        HeaderValue::from_str(refill_per_sec.to_string().as_str()),
    ) {
        resp.headers_mut().insert(
            header::HeaderName::from_static(HEADER_LIMIT),
            n,
        );
        resp.headers_mut().insert(
            header::HeaderName::from_static(HEADER_RESET),
            r,
        );
    }
    resp
}
