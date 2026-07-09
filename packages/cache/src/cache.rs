use deadpool_redis::Pool;
use redis::AsyncCommands;
use crate::error::CacheError;
#[derive(Clone)]
pub struct Cache { pub pool: Pool }
impl Cache {
    pub fn new(pool: Pool) -> Self { Self { pool } }
    pub async fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Option<T>, CacheError> {
        let mut c = self.pool.get().await?;
        let v: Option<String> = c.get(key).await?;
        Ok(match v { Some(s) => Some(serde_json::from_str(&s)?), None => None })
    }
    pub async fn set_ex<T: serde::Serialize>(&self, key: &str, value: &T, ttl_secs: u64) -> Result<(), CacheError> {
        let mut c = self.pool.get().await?;
        let s = serde_json::to_string(value)?;
        let _: () = c.set_ex(key, s, ttl_secs).await?;
        Ok(())
    }
    pub async fn delete(&self, key: &str) -> Result<(), CacheError> {
        let mut c = self.pool.get().await?;
        let _: () = c.del(key).await?;
        Ok(())
    }
}
