//! `AppError` -> RFC-7807 `ProblemDetails` + axum `IntoResponse` integration.

use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[serde(tag = "type", content = "details")]
pub enum AppError {
    #[error("not found")]
    NotFound { what: String },
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    #[error("forbidden: {0}")]
    Forbidden(String),
    #[error("validation: {0}")]
    Validation(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("upstream error: {0}")]
    Upstream(String),
    #[error("broker error: {0}")]
    Broker(String),
    #[error("rate limited")]
    RateLimited { retry_after_secs: u64 },
    #[error("internal: {0}")]
    Internal(String),
}

impl AppError {
    pub fn status(&self) -> StatusCode {
        match self {
            AppError::NotFound { .. } => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Upstream(_) => StatusCode::BAD_GATEWAY,
            AppError::Broker(_) => StatusCode::BAD_GATEWAY,
            AppError::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    pub fn to_problem(&self, instance: Option<String>) -> ProblemDetails {
        let (title, kind_suffix) = match self {
            AppError::NotFound { .. } => ("Not found", Some("not-found")),
            AppError::BadRequest(_) => ("Bad request", Some("bad-request")),
            AppError::Unauthorized(_) => ("Unauthorized", Some("unauthorized")),
            AppError::Forbidden(_) => ("Forbidden", Some("forbidden")),
            AppError::Validation(_) => ("Validation failed", Some("validation")),
            AppError::Conflict(_) => ("Conflict", Some("conflict")),
            AppError::Upstream(_) => ("Upstream error", Some("upstream")),
            AppError::Broker(_) => ("Broker error", Some("broker")),
            AppError::RateLimited { .. } => ("Rate limit exceeded", Some("rate-limited")),
            AppError::Internal(_) => ("Internal server error", Some("internal")),
        };
        let s = self.status();
        ProblemDetails {
            kind: kind_suffix.map(|k| format!("https://docs.ed/errors/{k}"))
                .unwrap_or_else(|| format!("about:blank#{}", s.as_u16())),
            title: title.to_string(),
            status: s.as_u16(),
            detail: Some(self.to_string()),
            instance,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemDetails {
    #[serde(rename = "type")]
    pub kind: String,
    pub title: String,
    pub status: u16,
    pub detail: Option<String>,
    pub instance: Option<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        let problem = self.to_problem(None);
        let mut resp = (status, Json(problem)).into_response();
        if let AppError::RateLimited { retry_after_secs } = &self {
            if let Ok(v) = retry_after_secs.to_string().parse() {
                resp.headers_mut().insert(axum::http::header::RETRY_AFTER, v);
            }
        }
        resp.headers_mut().insert(
            axum::http::header::CONTENT_TYPE,
            "application/problem+json".parse().unwrap(),
        );
        resp
    }
}

impl From<lapin::Error> for AppError {
    fn from(e: lapin::Error) -> Self { AppError::Broker(e.to_string()) }
}
impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self { AppError::Upstream(e.to_string()) }
}
impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self { AppError::Internal(format!("json: {e}")) }
}
impl From<redis::RedisError> for AppError {
    fn from(e: redis::RedisError) -> Self { AppError::Internal(format!("redis: {e}")) }
}
impl From<deadpool_redis::PoolError> for AppError {
    fn from(e: deadpool_redis::PoolError) -> Self { AppError::Internal(format!("redis pool: {e}")) }
}

pub type AppResult<T> = std::result::Result<T, AppError>;
