use std::env;
#[derive(Clone, Debug)]
pub struct Config {
    pub host: String, pub port: u16,
    pub database_url: String, pub redis_url: String, pub rabbit_url: String,
    pub service_name: String,
}
impl Config {
    /// #242: refuse to start with hardcoded DSN fallbacks. Operators
    /// MUST supply URLs (compose passes them, local devs should use
    /// `.env`); a missing var is a fatal startup error.
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: env::var("PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8080),
            database_url: env::var("DATABASE_URL")
                .map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?,
            redis_url: env::var("REDIS_URL")
                .map_err(|_| anyhow::anyhow!("REDIS_URL must be set"))?,
            rabbit_url: env::var("RABBITMQ_URL")
                .map_err(|_| anyhow::anyhow!("RABBITMQ_URL must be set"))?,
            service_name: "doc-service".into(),
        })
    }
}