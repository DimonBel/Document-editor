use sqlx::PgPool;
use crate::error::PgError;
#[derive(Clone)]
pub struct PlatformDb { pub pool: PgPool }
impl PlatformDb {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
    pub async fn acquire(&self) -> Result<sqlx::PgConnection, PgError> { Ok(self.pool.acquire().await?) }
    pub async fn begin(&self) -> Result<sqlx::Transaction<'_, sqlx::Postgres>, PgError> { Ok(self.pool.begin().await?) }
    pub fn pool(&self) -> &PgPool { &self.pool }
}
