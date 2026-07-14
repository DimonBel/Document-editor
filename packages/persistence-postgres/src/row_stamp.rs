use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use crate::platform_db::PlatformDb;

pub struct RowStampInterceptor;

impl RowStampInterceptor {
    /// Run the workspace's SQL migrations. `sqlx::migrate!` resolves
    /// the path relative to the current source file at compile time,
    /// so we point it at `src/migrations/` (the directory this file
    /// lives in).
    pub async fn ensure(pool: &sqlx::PgPool) -> Result<(), sqlx::migrate::MigrateError> {
        sqlx::migrate!("src/migrations").run(pool).await?;
        Ok(())
    }
}

pub async fn connect(database_url: &str) -> Result<PlatformDb, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await?;
    RowStampInterceptor::ensure(&pool)
        .await
        .map_err(|e| sqlx::Error::Migrate(Box::new(e)))?;
    Ok(PlatformDb::new(pool))
}
