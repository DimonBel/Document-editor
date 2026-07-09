//! Idempotency-Key middleware: replays cached responses for the same
//! `(user, route, key)` tuple.
//!
//! - Only non-GET requests with an `Idempotency-Key` header are processed.
//! - On hit: replay the cached status + body + headers.
//! - On miss: forward, then cache the response (24h TTL).

use axum::{
    body::{to_bytes, Body},
    extract::{Request, State},
    http::HeaderName,
    middleware::Next,
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::state::AppState;

const HEADER_KEY: &str = "idempotency-key";
const TTL_SECS: i64 = 24 * 60 * 60;
const MAX_BODY: usize = 4 * 1024 * 1024;

#[derive(Debug, Serialize, Deserialize)]
struct Cached {
    status: u16,
    headers: HashMap<String, String>,
    body: String, // base64 not needed; UTF-8 with replacement
}

pub async fn idempotency_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    if req.method() == axum::http::Method::GET {
        return next.run(req).await;
    }
    let key = match req.headers().get(HEADER_KEY).and_then(|v| v.to_str().ok()) {
        Some(k) if !k.is_empty() => k.to_string(),
        _ => return next.run(req).await,
    };
    let user = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "anon".to_string());
    let path = req.uri().path().to_string();
    let redis_key = format!("idem:{user}:{path}:{key}");

    // Check cache
    if let Ok(mut conn) = state.redis.get().await {
        if let Ok(Some(cached)) = conn.get::<_, Option<String>>(&redis_key).await {
            if let Ok(c) = serde_json::from_str::<Cached>(&cached) {
                let mut resp = Response::builder().status(c.status);
                for (k, v) in &c.headers {
                    if let (Ok(name), Ok(val)) = (
                        HeaderName::try_from(k.as_str()),
                        axum::http::HeaderValue::from_str(v),
                    ) {
                        resp = resp.header(name, val);
                    }
                }
                return resp
                    .header("X-Idempotent-Replay", "true")
                    .body(axum::body::Body::from(c.body))
                    .unwrap_or_else(|_| Response::new(Body::from("idempotency replay build error")));
            }
        }
    }

    // Forward
    let (parts, body) = req.into_parts();
    let body_bytes: Bytes = match to_bytes(body, MAX_BODY).await {
        Ok(b) => b,
        Err(_) => return next.run(Request::from_parts(parts, Body::empty())).await,
    };
    let req2 = Request::from_parts(parts, Body::from(body_bytes.clone()));
    let resp = next.run(req2).await;

    // Cache the response
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
    let cached = Cached {
        status: parts.status.as_u16(),
        headers: headers_map,
        body: String::from_utf8_lossy(&body_bytes).to_string(),
    };
    if let Ok(json) = serde_json::to_string(&cached) {
        if let Ok(mut conn) = state.redis.get().await {
            let _: Result<(), _> = conn.set_ex::<_, _, ()>(&redis_key, json, TTL_SECS as u64).await;
        }
    }

    Response::from_parts(parts, Body::from(body_bytes))
}
