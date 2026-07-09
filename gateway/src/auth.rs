//! `/auth/*` -- login, refresh, internal-token issuance.

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::security::jwt::{issue_internal_token, issue_user_token};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct LoginIn {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct TokenOut {
    pub access_token: String,
    pub token_type: &'static str,
    pub expires_in: i64,
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginIn>,
) -> AppResult<Json<TokenOut>> {
    // Dev-only stub auth: accept any (username, password).
    // In production: look up the user in Postgres, verify Argon2 hash, etc.
    if body.password.is_empty() {
        return Err(AppError::Validation("password is required".into()));
    }
    let user_id = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, body.username.as_bytes());
    let token = issue_user_token(
        &state.keys,
        &state.config.jwt_issuer,
        &state.config.jwt_audience,
        &user_id.to_string(),
        vec!["user".into()],
        vec!["rooms:read".into(), "rooms:write".into(), "documents:read".into(), "documents:write".into()],
        900,
    )?;
    Ok(Json(TokenOut { access_token: token, token_type: "Bearer", expires_in: 900 }))
}

#[derive(Debug, Deserialize)]
pub struct RefreshIn {
    pub refresh_token: String,
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(body): Json<RefreshIn>,
) -> AppResult<Json<TokenOut>> {
    // Stub: accept any non-empty refresh token and mint a new user token.
    // Production: validate the refresh token's jti against a Redis store, rotate.
    if body.refresh_token.is_empty() {
        return Err(AppError::Validation("refresh_token is required".into()));
    }
    let user_id = uuid::Uuid::new_v4();
    let token = issue_user_token(
        &state.keys,
        &state.config.jwt_issuer,
        &state.config.jwt_audience,
        &user_id.to_string(),
        vec!["user".into()],
        vec!["rooms:read".into(), "rooms:write".into()],
        900,
    )?;
    Ok(Json(TokenOut { access_token: token, token_type: "Bearer", expires_in: 900 }))
}

#[derive(Debug, Deserialize)]
pub struct InternalIn {
    pub service: String,
    pub secret: Option<String>,
}

pub async fn internal_token(
    State(state): State<AppState>,
    Json(body): Json<InternalIn>,
) -> AppResult<Json<TokenOut>> {
    // In production: validate `body.secret` against the configured internal secret.
    if body.service.is_empty() {
        return Err(AppError::Validation("service is required".into()));
    }
    let token = issue_internal_token(
        &state.config.internal_service_token_secret,
        &state.config.jwt_issuer,
        &body.service,
        60,
    )?;
    Ok(Json(TokenOut { access_token: token, token_type: "Bearer", expires_in: 60 }))
}
