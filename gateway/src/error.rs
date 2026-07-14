//! `AppError` -> RFC-7807 `ProblemDetails` + axum `IntoResponse` integration.
//!
//! Per issue #143, internal errors (broker / Redis / reqwest / serde /
//! JWT-parse / upstream-connect details) MUST NOT leak to callers.
//! They are split into:
//! - `Public(..)` variants that include `detail` (validation, not-found,
//!   bad-request, etc.);
//! - `Internal(..)` variants that NEVER include the underlying message;
//!   they emit a generic `Internal server error` and the detail is
//!   added to the structured logs only.

use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::error;

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
    #[error("rate limited")]
    RateLimited { retry_after_secs: u64 },
    /// Public-facing upstream error: do NOT leak the underlying URL /
    /// status / message. Detail is generic ("upstream unavailable").
    #[error("upstream unavailable")]
    Upstream(String),
    /// Public-facing broker error: do NOT leak transport details.
    #[error("broker unavailable")]
    Broker(String),
    /// Internal error: detail is logged, never sent to the client.
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
        // We build the values manually instead of using a single
        // `match` because the `kind` field on `ProblemDetails` is
        // a fully owned `String`; using a match with a borrowed
        // pattern (`Some(k)` where `k: &str`) would require the
        // arms to return `Option<&str>` (the type of `kind_suffix`),
        // not `String`, which is what `ProblemDetails.kind` expects.
        let (title, kind_suffix, detail) = match self {
            AppError::NotFound { what } => ("Not found", "not-found", Some(what.clone())),
            AppError::BadRequest(d)    => ("Bad request", "bad-request", Some(d.clone())),
            AppError::Unauthorized(d)  => ("Unauthorized", "unauthorized", Some(d.clone())),
            AppError::Forbidden(d)     => ("Forbidden", "forbidden", Some(d.clone())),
            AppError::Validation(d)    => ("Validation failed", "validation", Some(d.clone())),
            AppError::Conflict(d)      => ("Conflict", "conflict", Some(d.clone())),
            AppError::RateLimited { .. } => ("Rate limit exceeded", "rate-limited", Some("too many requests".into())),
            // Public errors: detail is intentionally generic.
            AppError::Upstream(_)  => ("Upstream unavailable", "upstream", None),
            AppError::Broker(_)    => ("Broker unavailable",  "broker", None),
            AppError::Internal(_)  => ("Internal server error", "internal", None),
        };
        let s = self.status();
        let kind = match kind_suffix {
            Some(k) => format!("https://docs.ed/errors/{k}"),
            None => format!("about:blank#{}", s.as_u16()),
        };
        ProblemDetails { kind,
            title: title.to_string(),
            status: s.as_u16(),
            detail,
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
        // Log the full underlying error for ops; only the sanitised
        // detail (if any) ever reaches the client.
        if matches!(self, AppError::Internal(_) | AppError::Upstream(_) | AppError::Broker(_)) {
            error!(error = %self, "request failed");
        }
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
    fn from(_e: lapin::Error) -> Self { AppError::Broker("broker connection failed".into()) }
}
impl From<reqwest::Error> for AppError {
    fn from(_e: reqwest::Error) -> Self { AppError::Upstream("upstream connection failed".into()) }
}
impl From<serde_json::Error> for AppError {
    fn from(_e: serde_json::Error) -> Self { AppError::Internal("json".into()) }
}
impl From<redis::RedisError> for AppError {
    fn from(_e: redis::RedisError) -> Self { AppError::Internal("redis".into()) }
}
impl From<deadpool_redis::PoolError> for AppError {
    fn from(_e: deadpool_redis::PoolError) -> Self { AppError::Internal("redis pool".into()) }
}

pub type AppResult<T> = std::result::Result<T, AppError>;
