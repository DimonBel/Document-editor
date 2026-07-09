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
impl From<sqlx::Error> for AppError { fn from(e: sqlx::Error) -> Self { AppError::Infra(format!("sqlx: {e}")) } }
impl From<mongodb::error::Error> for AppError { fn from(e: mongodb::error::Error) -> Self { AppError::Infra(format!("mongo: {e}")) } }
impl From<lapin::Error> for AppError { fn from(e: lapin::Error) -> Self { AppError::Broker(format!("lapin: {e}")) } }
impl From<serde_json::Error> for AppError { fn from(e: serde_json::Error) -> Self { AppError::Internal(format!("json: {e}")) } }
