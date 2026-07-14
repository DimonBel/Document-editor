use sqlx::{PgPool, Postgres};
use sqlx::pool::PoolConnection;
use crate::error::PgError;
#[derive(Clone)]
pub struct PlatformDb { pub pool: PgPool }
impl PlatformDb {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
    /// Acquire a pooled connection. The new sqlx 0.8 returns a
    /// `PoolConnection<Postgres>` (not `PgConnection`); callers that
    /// want a `PgConnection` should call `.detach()` on the result.
    pub async fn acquire(&self) -> Result<PoolConnection<Postgres>, PgError> {
        Ok(self.pool.acquire().await?)
    }
    pub async fn begin(&self) -> Result<sqlx::Transaction<'_, Postgres>, PgError> {
        Ok(self.pool.begin().await?)
    }
    pub fn pool(&self) -> &PgPool { &self.pool }
}
