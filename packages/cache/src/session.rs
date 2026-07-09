use crate::Cache;
use crate::error::CacheError;
use uuid::Uuid;
#[derive(Clone)]
pub struct Session { cache: Cache, ttl_secs: u64 }
impl Session {
    pub fn new(cache: Cache, ttl_secs: u64) -> Self { Self { cache, ttl_secs } }
    pub fn new_id() -> String { Uuid::new_v4().to_string() }
    pub async fn put<T: serde::Serialize>(&self, id: &str, value: &T) -> Result<(), CacheError> { self.cache.set_ex(&format!("sess:{id}"), value, self.ttl_secs).await }
    pub async fn get<T: serde::de::DeserializeOwned>(&self, id: &str) -> Result<Option<T>, CacheError> { self.cache.get(&format!("sess:{id}")).await }
    pub async fn drop(&self, id: &str) -> Result<(), CacheError> { self.cache.delete(&format!("sess:{id}")).await }
}
