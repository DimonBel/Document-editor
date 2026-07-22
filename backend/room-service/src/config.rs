use std::env;
#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub mongo_url: String,
    pub redis_url: String,
    pub rabbit_url: String,
    pub service_name: String,
}
impl Config {
    /// #242: refuse to start with hardcoded DSN fallbacks. Operators
    /// MUST supply URLs (compose passes them, local devs should use
    /// `.env`); a missing var is a fatal startup error.
    pub fn from_env() -> anyhow::Result<Self> {
        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
        let port: u16 = env::var("PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8080);
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?;
        let mongo_url = env::var("MONGO_URL")
            .map_err(|_| anyhow::anyhow!("MONGO_URL must be set"))?;
        let redis_url = env::var("REDIS_URL")
            .map_err(|_| anyhow::anyhow!("REDIS_URL must be set"))?;
        let rabbit_url = env::var("RABBITMQ_URL")
            .map_err(|_| anyhow::anyhow!("RABBITMQ_URL must be set"))?;
        Ok(Self {
            host, port, database_url, mongo_url, redis_url, rabbit_url,
            service_name: "room-service".into(),
        })
    }
}
