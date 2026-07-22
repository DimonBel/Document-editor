use std::env;
#[derive(Clone, Debug)]
pub struct Config {
    pub host: String, pub port: u16,
    pub database_url: String, pub redis_url: String, pub rabbit_url: String,
    pub service_name: String,
}
impl Config { pub fn from_env() -> Self { Self {
    host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
    port: env::var("PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8080),
    database_url: env::var("DATABASE_URL").expect("DATABASE_URL"),
    redis_url: env::var("REDIS_URL").expect("REDIS_URL"),
    rabbit_url: env::var("RABBITMQ_URL").unwrap_or_else(|_| "amqp://guest:guest@rabbit:5672/%2f".into()),
    service_name: "doc-service".into(),
}}}
