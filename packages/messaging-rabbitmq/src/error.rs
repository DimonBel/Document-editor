use thiserror::Error;
#[derive(Debug, Error)]
pub enum BrokerError {
    #[error("lapin: {0}")] Lapin(#[from] lapin::Error),
    #[error("json: {0}")] Json(#[from] serde_json::Error),
    #[error("pg: {0}")] Pg(#[from] ed_persistence_postgres::PgError),
    #[error("not connected")] NotConnected,
    #[error("topology mismatch: {0}")] TopologyMismatch(String),
    /// Catch-all for other internal errors that don't map to one
    /// of the dedicated variants (e.g. serialization of a T that
    /// serde_json::to_value rejected).
    #[error("internal: {0}")] Internal(String),
}
