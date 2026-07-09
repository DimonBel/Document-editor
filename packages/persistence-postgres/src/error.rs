use thiserror::Error;
#[derive(Debug, Error)]
pub enum PgError {
    #[error("sqlx: {0}")] Sqlx(#[from] sqlx::Error),
    #[error("not found")] NotFound,
    #[error("migration: {0}")] Migration(String),
}
