pub mod problem;
pub use problem::ProblemDetails;
use serde::{Deserialize, Serialize};
use thiserror::Error;
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[serde(tag = "type", content = "details")]
pub enum AppError {
    #[error(transparent)] Domain(ed_domain::DomainError),
    #[error("infra: {0}")] Infra(String),
    #[error("broker: {0}")] Broker(String),
    #[error("auth: {0}")] Auth(String),
    #[error("validation: {0}")] Validation(String),
    #[error("not found")] NotFound,
    #[error("internal: {0}")] Internal(String),
}
impl AppError {
    pub fn http_status(&self) -> u16 {
        match self {
            AppError::Domain(d) => match d {
                ed_domain::DomainError::NotFound { .. } => 404,
                ed_domain::DomainError::Validation(_) => 422,
                ed_domain::DomainError::Conflict(_) => 409,
                ed_domain::DomainError::Unauthorized(_) => 401,
                ed_domain::DomainError::Forbidden(_) => 403,
                ed_domain::DomainError::Invariant(_) => 400,
            },
            AppError::NotFound => 404,
            AppError::Validation(_) => 422,
            AppError::Auth(_) => 401,
            AppError::Infra(_) | AppError::Broker(_) => 502,
            AppError::Internal(_) => 500,
        }
    }
    pub fn to_problem(&self, instance: Option<String>) -> ProblemDetails { ProblemDetails::from_app(self, instance) }
}
pub type AppResult<T> = std::result::Result<T, AppError>;
impl From<ed_domain::DomainError> for AppError { fn from(v: ed_domain::DomainError) -> Self { AppError::Domain(v) } }
// Issue #227: do not leak driver strings to API clients. Log the full
// detail server-side, return a generic detail to the caller.
impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        tracing::error!(error = ?e, "sqlx error");
        AppError::Infra("database error".into())
    }
}
impl From<mongodb::error::Error> for AppError {
    fn from(e: mongodb::error::Error) -> Self {
        tracing::error!(error = ?e, "mongo error");
        AppError::Infra("database error".into())
    }
}
impl From<lapin::Error> for AppError {
    fn from(e: lapin::Error) -> Self {
        tracing::error!(error = ?e, "lapin error");
        AppError::Broker("broker error".into())
    }
}
impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        tracing::error!(error = ?e, "json error");
        AppError::Internal("serialisation error".into())
    }
}

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        use axum::response::IntoResponse;
        let status = http::StatusCode::from_u16(self.http_status())
            .unwrap_or(http::StatusCode::INTERNAL_SERVER_ERROR);
        (status, axum::Json(self.to_problem(None))).into_response()
    }
}
