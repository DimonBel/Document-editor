//! Auth middleware: verifies `Authorization: Bearer <jwt>` and injects a
//! `CurrentUser` into request extensions for downstream handlers.

use axum::{
    extract::Request,
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};
use ed_domain::UserId;
use std::sync::Arc;

use crate::error::AppError;
use crate::security::jwt::Claims;
use crate::state::AppState;

/// Verified user attached to a request after the auth middleware runs.
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: UserId,
    pub sub: String,
    pub roles: Vec<String>,
    pub scopes: Vec<String>,
    pub correlation_id: Option<String>,
}

impl CurrentUser {
    pub fn has_role(&self, r: &str) -> bool { self.roles.iter().any(|x| x == r) }
    pub fn has_scope(&self, s: &str) -> bool { self.scopes.iter().any(|x| x == s) }
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Skip auth for well-known public paths
    let path = req.uri().path();
    if path == "/healthz"
        || path == "/.well-known/jwks.json"
        || path == "/.well-known/openid-configuration"
        || path.starts_with("/auth/")
    {
        return Ok(next.run(req).await);
    }

    let token = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::Unauthorized("missing bearer token".into()))?;

    let claims: Claims = state
        .keys
        .verify(token, &state.config.jwt_issuer, &state.config.jwt_audience)?;

    let id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("sub is not a UUID".into()))?;
    let user = CurrentUser {
        id: id.into(),
        sub: claims.sub.clone(),
        roles: claims.roles.clone(),
        scopes: claims.scopes.clone(),
        correlation_id: claims.correlation_id.clone(),
    };
    req.extensions_mut().insert(Arc::new(user));
    Ok(next.run(req).await)
}
