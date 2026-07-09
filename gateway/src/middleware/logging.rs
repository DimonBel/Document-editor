//! Structured request-logging middleware.

use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;

pub async fn logging_middleware(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let started = Instant::now();
    let resp = next.run(req).await;
    let elapsed = started.elapsed();
    let status = resp.status().as_u16();
    tracing::info!(method = %method, path = %path, status, ?elapsed, "request");
    resp
}
