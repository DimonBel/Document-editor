use std::env;
#[derive(Clone, Debug)]
pub struct Config { pub host: String, pub port: u16, pub database_url: String, pub mongo_url: String, pub rabbit_url: String, pub artefacts_dir: String }
impl Config { pub fn from_env() -> Self { Self {
    host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
    port: env::var("PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8080),
    database_url: env::var("DATABASE_URL").expect("DATABASE_URL"),
    mongo_url: env::var("MONGO_URL").expect("MONGO_URL"),
    rabbit_url: env::var("RABBITMQ_URL").expect("RABBITMQ_URL"),
    artefacts_dir: env::var("LATEX_ARTEFACTS_DIR").unwrap_or_else(|_| "/var/lib/latex".into()),
}}}
