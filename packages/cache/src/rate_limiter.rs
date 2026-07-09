use redis::AsyncCommands;
use deadpool_redis::Pool;
use crate::error::CacheError;
#[derive(Debug, Clone, Copy)]
pub enum RateLimitDecision { Allow, Deny }
pub struct RateLimiter { pub pool: Pool, pub capacity: u32, pub refill_per_sec: u32 }
impl RateLimiter {
    pub fn new(pool: Pool, capacity: u32, refill_per_sec: u32) -> Self { Self { pool, capacity, refill_per_sec } }
    pub async fn try_acquire(&self, key: &str) -> Result<RateLimitDecision, CacheError> {
        let mut c = self.pool.get().await?;
        let bucket = (chrono::Utc::now().timestamp() as u32) / self.refill_per_sec.max(1);
        let full_key = format!("rl:{key}:{bucket}");
        let count: u32 = c.incr(&full_key, 1u32).await?;
        if count == 1 { let _: () = c.expire(&full_key, 60).await?; }
        if count > self.capacity { Ok(RateLimitDecision::Deny) } else { Ok(RateLimitDecision::Allow) }
    }
}
