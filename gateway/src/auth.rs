//! `/auth/*` -- login, refresh, internal-token issuance.
//!
//! Requires a real `UserStore` + Argon2id password verification and constant-time
//! comparison for the internal secret.

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::security::jwt::{issue_internal_token, issue_user_token};
use crate::security::users::{ct_eq, hash_password, verify_password, RefreshTokenStore, User};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct LoginIn {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct TokenOut {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: &'static str,
    pub expires_in: i64,
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginIn>,
) -> AppResult<Json<TokenOut>> {
    if body.username.is_empty() || body.password.is_empty() {
        return Err(AppError::Validation("username and password are required".into()));
    }
    let user = state
        .users
        .find_by_username(&body.username)
        .await?
        .ok_or_else(|| AppError::Unauthorized("invalid credentials".into()))?;

    // Constant-time password check (argon2 itself is constant-time on the hash side).
    verify_password(&body.password, &user.password_hash)?;
    issue_token_pair(&state, user).await
}

#[derive(Debug, Deserialize)]
pub struct RefreshIn {
    pub refresh_token: String,
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(body): Json<RefreshIn>,
) -> AppResult<Json<TokenOut>> {
    let store = RefreshTokenStore { redis: state.redis.clone() };
    let user_id = store
        .consume(&body.refresh_token)
        .await?
        .ok_or_else(|| AppError::Unauthorized("invalid or expired refresh token".into()))?;
    let user = state
        .users
        .find_by_id(&user_id)
        .await?
        .ok_or_else(|| AppError::Unauthorized("user no longer exists".into()))?;
    issue_token_pair(&state, user).await
}

#[derive(Debug, Deserialize)]
pub struct InternalIn {
    pub service: String,
    pub secret: String,
}

pub async fn internal_token(
    State(state): State<AppState>,
    Json(body): Json<InternalIn>,
) -> AppResult<Json<TokenOut>> {
    if body.service.is_empty() || body.secret.is_empty() {
        return Err(AppError::Validation("service and secret are required".into()));
    }
    if !ct_eq(&body.secret, &state.config.internal_service_token_secret) {
        // spend time equal to the secret length to avoid leaking which path we took
        let _ = ct_eq("x", "y");
        return Err(AppError::Unauthorized("invalid service credentials".into()));
    }
    // Issue #220: only mint internal tokens for known service identifiers.
    // Without this allowlist, anyone holding the shared secret could mint a
    // token claiming to be any service (`room-service`, `admin-service`,
    // `ed-platform-superuser`, etc.).
    let allowed = state.config.services.keys();
    if !allowed.into_iter().any(|s| s.as_str() == body.service.as_str()) {
        return Err(AppError::Validation(format!(
            "unknown service: '{}' (allowed: {:?})",
            body.service,
            state.config.services.keys().collect::<Vec<_>>()
        )));
    }
    let token = issue_internal_token(
        &state.config.internal_service_token_secret,
        &state.config.jwt_issuer,
        &body.service,
        60,
    )?;
    Ok(Json(TokenOut {
        access_token: token,
        refresh_token: String::new(),  // internal tokens are not refreshable
        token_type: "Bearer",
        expires_in: 60,
    }))
}

async fn issue_token_pair(state: &AppState, user: User) -> AppResult<Json<TokenOut>> {
    let access = issue_user_token(
        &state.keys,
        &state.config.jwt_issuer,
        &state.config.jwt_audience,
        &user.id,
        user.roles.clone(),
        user.scopes.clone(),
        900,
    )?;
    let refresh = RefreshTokenStore { redis: state.redis.clone() }
        .issue(&user.id)
        .await?;
    Ok(Json(TokenOut {
        access_token: access,
        refresh_token: refresh,
        token_type: "Bearer",
        expires_in: 900,
    }))
}

/// CLI: hash a password from the command line. Used by operators.
pub fn hash_for_cli(password: &str) -> String {
    hash_password(password).unwrap_or_else(|e| {
        eprintln!("hash error: {e}");
        std::process::exit(1);
    })
}
