//! Correlation-id middleware: read or generate `X-Correlation-Id`, attach to
//! request extensions, and echo it back in the response.

use axum::{
    extract::Request,
    http::HeaderValue,
    middleware::Next,
    response::Response,
};

pub const HEADER: &str = "x-correlation-id";

pub async fn correlation_middleware(mut req: Request, next: Next) -> Response {
    let cid = req
        .headers()
        .get(HEADER)
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    req.extensions_mut().insert(cid.clone());
    let mut resp = next.run(req).await;
    if let Ok(v) = HeaderValue::from_str(&cid) {
        resp.headers_mut().insert(HEADER, v);
    }
    resp
}
