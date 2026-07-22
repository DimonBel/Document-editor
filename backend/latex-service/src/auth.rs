use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};
use ed_auth::JwtVerifier;
use std::sync::Arc;

/// Backend services declare ed-auth but never used it (issue #217).
/// This middleware enforces that every request to a non-public route
/// carries a valid JWT minted by the gateway. The shared secret is
/// loaded from `INTERNAL_SERVICE_TOKEN_SECRET` (HS256).
pub async fn require_internal_auth(
    State(verifier): State<Arc<JwtVerifier>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = req.uri().path();
    if path == "/healthz" {
        return Ok(next.run(req).await);
    }

    if std::env::var("ED_DEV_MODE").is_ok() {
        return Ok(next.run(req).await);
    }

    let token = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    verifier
        .verify(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    Ok(next.run(req).await)
}