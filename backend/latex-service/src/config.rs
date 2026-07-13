use std::env;
#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub rabbit_url: String,
    pub artefacts_dir: String,
}
impl Config {
    pub fn from_env() -> Self {
        Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: env::var("PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8080),
            database_url: env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://ed:ed@postgres:5432/ed".into()),
            redis_url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://redis:6379".into()),
            rabbit_url: env::var("RABBITMQ_URL").unwrap_or_else(|_| "amqp://guest:guest@rabbit:5672/".into()),
            artefacts_dir: env::var("LATEX_ARTEFACTS_DIR").unwrap_or_else(|_| "/var/lib/latex".into()),
        }
    }
}
