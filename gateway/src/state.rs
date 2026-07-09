//! Shared application state, wired into every handler via `axum::extract::State`.

use std::collections::HashMap;
use std::sync::Arc;

use deadpool_redis::Pool as RedisPool;
use parking_lot::RwLock;

use crate::config::{Config, UpstreamConfig};
use crate::security::jwt::KeyManager;

/// Process-wide state. Cheap to clone (everything inside is `Arc`).
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub keys: Arc<KeyManager>,
    pub redis: RedisPool,
    pub http: reqwest::Client,
    pub ws_clients: Arc<RwLock<HashMap<String, Vec<tokio::sync::mpsc::UnboundedSender<serde_json::Value>>>>>,
    pub rabbit_channel: Arc<tokio::sync::Mutex<Option<lapin::Channel>>>,
    pub rabbit_url: String,
}

impl AppState {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let cfg = Arc::new(config);
        let keys = Arc::new(KeyManager::new()?);

        // Redis pool
        let redis_cfg = deadpool_redis::Config::from_url(&cfg.redis_url);
        let redis = redis_cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))?;

        // HTTP client for upstream proxy
        let http = reqwest::Client::builder()
            .user_agent("ed-gateway/0.1")
            .timeout(std::time::Duration::from_secs(30))
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .build()?;

        Ok(Self {
            config: cfg,
            keys,
            redis,
            http,
            ws_clients: Arc::new(RwLock::new(HashMap::new())),
            rabbit_channel: Arc::new(tokio::sync::Mutex::new(None)),
            rabbit_url: cfg.rabbitmq_url.clone(),
        })
    }

    /// Look up an upstream config by service name.
    pub fn upstream(&self, svc: &str) -> Option<UpstreamConfig> {
        self.config.services.get(svc).cloned()
    }
}
