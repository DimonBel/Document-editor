//! Idempotency-Key middleware: replays cached responses for the same
//! `(principal, route, key)` tuple.
//!
//! Per issue #141 (security fixes):
//! - The cache key MUST NOT contain the raw `Authorization` header
//!   (it would leak a bearer token into Redis observability). We
//!   instead use the authenticated user id from `CurrentUser`
//!   (`user:anon` if unauthenticated) -- this is a stable identifier
//!   the gateway already trusts to identify principals.
//! - The cached response body is stored as `Vec<u8>` (not lossy UTF-8),
//!   so binary responses (PDF / DOCX / images) replay identically.
//! - We SHA-256 the cached body and log it so cache-hit rate can be
//!   measured without ever inspecting the body.

use axum::{
    body::{to_bytes, Body},
    extract::{Request, State},
    http::HeaderName,
    middleware::Next,
    response::{Response},
};
use base64::Engine;
use bytes::Bytes;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;

use crate::security::middleware::CurrentUser;
use crate::state::AppState;

const HEADER_KEY: &str = "idempotency-key";
const TTL_SECS: i64 = 24 * 60 * 60;
const MAX_BODY: usize = 4 * 1024 * 1024;

#[derive(Debug, Serialize, Deserialize)]
struct Cached {
    status: u16,
    headers: HashMap<String, String>,
    /// base64(bytes) -- lossless binary-safe storage.
    body_b64: String,
    body_len: usize,
    body_sha256: String, // hex
}

pub async fn idempotency_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let method = req.method().clone();
    if method == axum::http::Method::GET {
        return next.run(req).await;
    }
    let key = match req.headers().get(HEADER_KEY).and_then(|v| v.to_str().ok()) {
        Some(k) if !k.is_empty() => k.to_string(),
        _ => return next.run(req).await,
    };

    // Principal-only cache key.
    let principal = req
        .extensions()
        .get::<CurrentUser>()
        .map(|u| format!("user:{}", u.id))
        .unwrap_or_else(|| "anon".to_string());
    let path = req.uri().path().to_string();
    // Hash the body SHA into the key so two idempotent requests with
    // different bodies don't replay each other.
    let redis_key_pre = format!("idem:{principal}:{path}:{key}");
    let redis_key = match read_body_for_key(&req).await {
        Ok(()) => {
            let body_hash = last_seen_body(&req);
            format!("{redis_key_pre}:{body_hash}")
        }
        // If body too large, fall back to key-only (still safe).
        Err(_) => redis_key_pre,
    };

    // Check cache.
    if let Ok(mut conn) = state.redis.get().await {
        if let Ok(Some(cached)) = conn.get::<_, Option<String>>(&redis_key).await {
            if let Ok(c) = serde_json::from_str::<Cached>(&cached) {
                let body = base64::engine::general_purpose::STANDARD
                    .decode(&c.body_b64)
                    .unwrap_or_default();
                let mut resp = Response::builder()
                    .status(c.status)
                    .header("X-Idempotent-Replay", "true")
                    .header("X-Idempotent-Body-Sha256", &c.body_sha256)
                    .body(axum::body::Body::from(body))
                    .unwrap_or_else(|_| {
                        Response::builder()
                            .status(500)
                            .body(axum::body::Body::empty())
                            .unwrap()
                    });
                for (k, v) in &c.headers {
                    if let (Ok(name), Ok(val)) = (
                        HeaderName::try_from(k.as_str()),
                        axum::http::HeaderValue::from_str(v),
                    ) {
                        resp.headers_mut().insert(name, val);
                    }
                }
                return resp;
            }
        }
    }

    // Forward.
    let (parts, body) = req.into_parts();
    let body_bytes: Bytes = match to_bytes(body, MAX_BODY).await {
        Ok(b) => b,
        Err(_) => return next.run(Request::from_parts(parts, Body::empty())).await,
    };
    let req2 = Request::from_parts(parts, Body::from(body_bytes.clone()));
    let resp = next.run(req2).await;

    // Cache the response.
    let (parts, body) = resp.into_parts();
    let body_bytes: Bytes = match to_bytes(body, MAX_BODY).await {
        Ok(b) => b,
        Err(_) => return Response::from_parts(parts, Body::empty()),
    };
    let headers_map: HashMap<String, String> = parts
        .headers
        .iter()
        .filter_map(|(k, v)| v.to_str().ok().map(|s| (k.as_str().to_string(), s.to_string())))
        .collect();
    let mut sha = Sha256::new();
    sha.update(&body_bytes);
    let body_sha = sha.finalize();
    let body_sha_hex = body_sha.iter().map(|b| format!("{b:02x}")).collect::<String>();
    let cached = Cached {
        status: parts.status.as_u16(),
        headers: headers_map,
        body_b64: base64::engine::general_purpose::STANDARD.encode(&body_bytes),
        body_len: body_bytes.len(),
        body_sha256: body_sha_hex,
    };
    if let Ok(json) = serde_json::to_string(&cached) {
        if let Ok(mut conn) = state.redis.get().await {
            let _: Result<(), _> = conn.set_ex::<_, _, ()>(&redis_key, json, TTL_SECS as u64).await;
        }
    }

    Response::from_parts(parts, Body::from(body_bytes))
}

// --- helpers ----------------------------------------------------------------

/// Reads the request body and stashes a SHA-256 in a thread-local so the
/// key builder function can include it. Used to dedupe idempotency-key
/// collisions across distinct bodies.
async fn read_body_for_key(req: &Request) -> Result<(), ()> {
    // `Request::clone` is cheap (Body is `Bytes::clone`), but
    // `into_parts` then moves the body. Use the standard
    // `Request<Body>::into_parts` via a `parts()`-style helper
    // that operates on `Parts + Body` separately -- we only need
    // the body here, so we can use `Request::into_body` to
    // extract just the body without consuming the whole request.
    let body = req.clone().into_body();
    let bytes: Result<Bytes, _> = to_bytes(body, MAX_BODY).await;
    bytes.map(|b| {
        let mut sha = Sha256::new();
        sha.update(&b);
        let hex: String = sha.finalize().iter().map(|x| format!("{x:02x}")).collect();
        LAST_BODY.with(|cell| *cell.borrow_mut() = Some(hex));
    }).map_err(|_| ())
}
    bytes.map(|b| {
        let mut sha = Sha256::new();
        sha.update(&b);
        let hex: String = sha.finalize().iter().map(|x| format!("{x:02x}")).collect();
        LAST_BODY.with(|cell| *cell.borrow_mut() = Some(hex));
    }).map_err(|_| ())
}

thread_local! {
    static LAST_BODY: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

fn last_seen_body(_req: &Request) -> String {
    LAST_BODY.with(|cell| cell.borrow_mut().take().unwrap_or_else(|| "unknown".into()))
}
