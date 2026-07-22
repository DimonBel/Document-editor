//! Shared application state, wired into every handler via `axum::extract::State`.

use std::sync::Arc;

use deadpool_redis::Pool as RedisPool;
use parking_lot::Mutex;

use crate::config::{Config, UpstreamConfig};
use crate::realtime::SubscriberTable;
use crate::security::jwt::KeyManager;
use crate::security::users::{InMemoryUserStore, User, UserStore};

/// Process-wide state. Cheap to clone (everything inside is `Arc`).
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub keys: Arc<KeyManager>,
    pub redis: RedisPool,
    pub http: reqwest::Client,
    pub ws_clients: Arc<Mutex<SubscriberTable>>,
    pub rabbit_channel: Arc<tokio::sync::Mutex<Option<lapin::Channel>>>,
    pub rabbit_url: String,
    pub users: Arc<dyn UserStore>,
}

impl AppState {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let cfg = Arc::new(config);
        // Issue #212: load the RSA keypair from disk if present, else
        // generate a new one and persist it (0600).
        let keys_path = std::path::PathBuf::from(&cfg.keys_path);
        let keys = Arc::new(KeyManager::new_persisted(Some(&keys_path))?);

        // Redis pool
        let redis_cfg = deadpool_redis::Config::from_url(&cfg.redis_url);
        let redis = redis_cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))?;

        // HTTP client for upstream proxy
        let http = reqwest::Client::builder()
            .user_agent("ed-gateway/0.1")
            .timeout(std::time::Duration::from_secs(30))
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .build()?;

        // Seed the in-memory user store from env (operator-seeded; production
        // would use a `users` Postgres table behind the same `UserStore` trait).
        let users = Arc::new(InMemoryUserStore::new());
        if let (Some(username), Some(hash_or_plain)) = (
            std::env::var("SEED_USERNAME").ok(),
            std::env::var("SEED_PASSWORD_HASH").ok().or_else(|| std::env::var("SEED_PASSWORD").ok()),
        ) {
            // Derive a stable user-id from the seed username (no v5
            // available in uuid 1.x); use a v4 UUID derived from a
            // hash instead.
            let id = {
                use sha2::{Digest, Sha256};
                let mut h = Sha256::new();
                h.update(b"ed-gateway-seed");
                h.update(username.as_bytes());
                let digest = h.finalize();
                let mut bytes = [0u8; 16];
                bytes.copy_from_slice(&digest[..16]);
                // Set version (4) and variant (10) bits per RFC 4122.
                bytes[6] = (bytes[6] & 0x0f) | 0x40;
                bytes[8] = (bytes[8] & 0x3f) | 0x80;
                uuid::Uuid::from_bytes(bytes).to_string()
            };
            let password_hash = std::env::var("SEED_PASSWORD_HASH").ok()
                .unwrap_or_else(|| crate::security::users::hash_password(&hash_or_plain).unwrap_or_default());
            users.insert(User {
                id,
                username: username.clone(),
                email: None,
                roles: vec!["user".into()],
                scopes: vec![
                    "rooms:read".into(), "rooms:write".into(),
                    "documents:read".into(), "documents:write".into(),
                ],
                password_hash,
            });
            tracing::info!(%username, "seeded user");
        }

        Ok(Self {
            config: cfg.clone(),
            keys,
            redis,
            http,
            ws_clients: Arc::new(Mutex::new(SubscriberTable::default())),
            rabbit_channel: Arc::new(tokio::sync::Mutex::new(None)),
            rabbit_url: cfg.rabbitmq_url.clone(),
            users,
        })
    }

    /// Look up an upstream config by service name.
    pub fn upstream(&self, svc: &str) -> Option<UpstreamConfig> {
        self.config.services.get(svc).cloned()
    }
}
