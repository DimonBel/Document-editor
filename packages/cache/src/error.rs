use thiserror::Error;
#[derive(Debug, Error)]
pub enum CacheError {
    #[error("redis: {0}")] Redis(#[from] redis::RedisError),
    #[error("pool: {0}")] Pool(String),
    #[error("json: {0}")] Json(#[from] serde_json::Error),
}
impl From<deadpool_redis::PoolError> for CacheError { fn from(e: deadpool_redis::PoolError) -> Self { Self::Pool(e.to_string()) } }
