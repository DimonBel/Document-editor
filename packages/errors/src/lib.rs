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
impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        tracing::error!(error = ?e, "json error");
        AppError::Internal("serialisation error".into())
    }
}
// Issue #250: ed-errors is the lowest-level crate and MUST NOT depend
// on sqlx, mongodb, or lapin. Driver-specific From impls live in
// each service (or its nearest infrastructure crate). A single,
// minimal helper is provided here so services can `map_err` without
// re-implementing the same body.
pub fn log_and_into<T: std::fmt::Display>(label: &str, err: T) -> AppError {
    tracing::error!(error = %err, "{label}");
    AppError::Infra(label.to_string())
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