use serde::{Deserialize, Serialize};
use thiserror::Error;
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[serde(tag = "type", content = "details")]
pub enum DomainError {
    #[error("validation: {0}")] Validation(String),
    #[error("not found: {entity} #{id}")] NotFound { entity: String, id: String },
    #[error("conflict: {0}")] Conflict(String),
    #[error("unauthorized: {0}")] Unauthorized(String),
    #[error("forbidden: {0}")] Forbidden(String),
    #[error("invariant: {0}")] Invariant(String),
}
pub type DomainResult<T> = std::result::Result<T, DomainError>;
