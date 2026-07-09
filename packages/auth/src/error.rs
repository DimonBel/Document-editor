use thiserror::Error;
#[derive(Debug, Clone, Error)]
pub enum AuthError {
    #[error("invalid token: {0}")] InvalidToken(String),
    #[error("missing scope: {0}")] MissingScope(String),
    #[error("missing role: {0}")] MissingRole(String),
}
