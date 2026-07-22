//! Reverse-proxy: forward `/api/v1/{svc}/{path:path}` to an upstream service.
//!
//! Request flow:
//!   1. Look up the service in `AppState::services` (404 if unknown).
//!   2. Verify JWT (auth middleware).
//!   3. Build the upstream URL: `<base_url>/<path>`.
//!   4. Forward method, query, headers (filtered), and body.
//!   5. Inject `Authorization: Internal <internal_token>` and `X-Correlation-Id`.
//!   6. Stream the upstream response back to the client.

use axum::{
    body::{to_bytes, Body},
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use std::collections::HashMap;
use std::str::FromStr;

use crate::error::{AppError, AppResult};
use crate::security::jwt::issue_internal_token;
use crate::state::AppState;

const MAX_BODY: usize = 16 * 1024 * 1024; // 16 MiB

pub async fn proxy(
    State(state): State<AppState>,
    method: Method,
    Path(captured_path): Path<String>,
    Query(query): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Body,
) -> AppResult<Response> {
    let (svc, path) = captured_path
        .split_once('/')
        .map(|(svc, path)| (svc.to_string(), path.to_string()))
        .ok_or_else(|| AppError::NotFound {
            what: "service path is missing".to_string(),
        })?;

    let upstream = state
        .upstream(&svc)
        .ok_or_else(|| AppError::NotFound { what: format!("unknown service '{svc}'") })?;

    let url = format!("{}/{}", upstream.base_url.trim_end_matches('/'), path);

    // Read body (cap at 16 MiB)
    let body_bytes: Bytes = to_bytes(body, MAX_BODY).await
        .map_err(|e| AppError::BadRequest(format!("body read: {e}")))?;

    // Filter incoming headers
    let mut req = state.http.request(method.clone(), &url);
    let mut header_count = 0;
    for (k, v) in headers.iter() {
        // Strip hop-by-hop, content-length, host
        let name = k.as_str().to_lowercase();
        if matches!(name.as_str(),
            "host" | "content-length" | "connection" | "transfer-encoding" | "keep-alive" | "upgrade"
        ) { continue; }
        req = req.header(k.as_str(), v.as_bytes());
        header_count += 1;
    }

    // Inject internal service-to-service auth
    let internal = issue_internal_token(
        &state.config.internal_service_token_secret,
        &state.config.jwt_issuer,
        "gateway",
        60,
    )?;
    req = req.header("Authorization", format!("Bearer {internal}"));

    // Forward correlation id (or generate one)
    let cid = headers
        .get("x-correlation-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    req = req.header("X-Correlation-Id", cid.clone());

    // Forward query string
    if !query.is_empty() {
        req = req.query(&query);
    }

    // Attach body
    if !body_bytes.is_empty() {
        req = req.body(body_bytes);
    }

    // Send
    let upstream_resp = req.send().await
        .map_err(|e| AppError::Upstream(format!("send: {e}")))?;

    // Build response
    let status = StatusCode::from_u16(upstream_resp.status().as_u16())
        .unwrap_or(StatusCode::BAD_GATEWAY);
    let mut response_headers = HeaderMap::new();
    for (k, v) in upstream_resp.headers().iter() {
        let name = k.as_str().to_lowercase();
        if matches!(name.as_str(),
            "content-encoding" | "transfer-encoding" | "content-length" | "connection" | "keep-alive"
        ) { continue; }
        if let (Ok(hname), Ok(hval)) = (HeaderName::from_str(k.as_str()), HeaderValue::from_bytes(v.as_bytes())) {
            response_headers.insert(hname, hval);
        }
    }
    response_headers.insert("X-Correlation-Id", HeaderValue::from_str(&cid).unwrap());
    let body = upstream_resp.bytes().await
        .map_err(|e| AppError::Upstream(format!("body read: {e}")))?;

    tracing::info!(
        method = %method, svc = %svc, path = %path, status = %status, headers = header_count,
        bytes = body.len(), "proxied"
    );

    Ok((status, response_headers, body).into_response())
}
