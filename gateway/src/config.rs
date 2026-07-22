//! Environment configuration.

use std::collections::HashMap;
use std::env;

#[derive(Clone, Debug)]
pub struct UpstreamConfig {
    pub name: String,
    pub base_url: String,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub service_name: String,
    pub database_url: String,
    pub mongo_url: String,
    pub redis_url: String,
    pub rabbitmq_url: String,
    pub jwt_issuer: String,
    pub jwt_audience: String,
    pub jwks_url: String,
    pub internal_service_token_secret: String,
    pub otel_endpoint: String,
    pub rate_limit: HashMap<String, (u32, u32)>,  // prefix -> (capacity, refill_per_sec)
    pub services: HashMap<String, UpstreamConfig>,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let dev_mode = env::var("ED_DEV_MODE").is_ok();

        let host = env::var("GATEWAY_HOST").unwrap_or_else(|_| "0.0.0.0".into());
        let port: u16 = env::var("GATEWAY_PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8080);
        let service_name = env::var("SERVICE_NAME").unwrap_or_else(|_| "gateway".into());
        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://ed:ed@postgres:5432/ed".into());
        let mongo_url = env::var("MONGO_URL").unwrap_or_else(|_| "mongodb://mongo:27017/ed".into());
        let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://redis:6379".into());
        let rabbitmq_url = env::var("RABBITMQ_URL").unwrap_or_else(|_| "amqp://guest:guest@rabbit:5672/%2f".into());
        let jwt_issuer = env::var("JWT_ISSUER").unwrap_or_else(|_| "ed-gateway".into());
        let jwt_audience = env::var("JWT_AUDIENCE").unwrap_or_else(|_| "ed-services".into());
        let jwks_url = env::var("JWKS_URL").unwrap_or_else(|_| "http://gateway:8080/.well-known/jwks.json".into());

        // Required production secrets: refuse weak defaults unless `ED_DEV_MODE` is set.
        let internal_service_token_secret = env::var("INTERNAL_SERVICE_TOKEN_SECRET")
            .unwrap_or_else(|_| {
                if dev_mode { "dev-only-secret".to_string() }
                else {
                    eprintln!("FATAL: INTERNAL_SERVICE_TOKEN_SECRET is not set. Refusing to start with the placeholder.");
                    std::process::exit(78);
                }
            });
        if !dev_mode && (internal_service_token_secret.len() < 32 || internal_service_token_secret == "changeme") {
            eprintln!("FATAL: INTERNAL_SERVICE_TOKEN_SECRET is weak (length {}). Refusing to start.", internal_service_token_secret.len());
            std::process::exit(78);
        }

        let otel_endpoint = env::var("OTEL_EXPORTER_OTLP_ENDPOINT").unwrap_or_default();

        // Default rate-limit: 100 req / 60 sec for /api/v1/*
        let mut rate_limit = HashMap::new();
        rate_limit.insert("/api/v1/room-service".to_string(),  (100, 60));
        rate_limit.insert("/api/v1/doc-service".to_string(),   (100, 60));
        rate_limit.insert("/api/v1/latex-service".to_string(), ( 20, 60));

        // Upstream services
        let mut services = HashMap::new();
        for svc in ["room-service", "doc-service", "latex-service"] {
            let base = env::var(format!("{}_URL", svc.to_uppercase().replace('-', "_")))
                .unwrap_or_else(|_| format!("http://{}:8080", svc));
            services.insert(svc.to_string(), UpstreamConfig { name: svc.into(), base_url: base });
        }

        Ok(Self { host, port, service_name, database_url, mongo_url, redis_url, rabbitmq_url,
                  jwt_issuer, jwt_audience, jwks_url, internal_service_token_secret, otel_endpoint,
                  rate_limit, services })
    }

    pub fn bind_addr(&self) -> String { format!("{}:{}", self.host, self.port) }
}
