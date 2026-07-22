//! `doc-service` -- axum, real vertical slice.
//!
//! Per #146: REST CRUD persists to Postgres; WS at
//! `/api/v1/doc-service/ws/doc/{id}` relays ops to peers and persists
//! them to the Postgres outbox. Background relay publishes to RabbitMQ.

use axum::{routing::get, Router};
use ed_cache::Cache;
use ed_messaging_rabbitmq::{IEventBus, OutboxRelayService};
use sqlx::PgPool;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

use crate::config::Config;
use crate::handlers::{create_document, delete_document, get_document, list_documents, update_document};
use crate::ws::ws_handler;
use crate::ws::DocHub;
use ed_persistence_postgres::{EfOutboxStore, OutboxStore};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub cache: Cache,
    pub outbox: Arc<dyn OutboxStore>,
    pub event_bus: Arc<dyn IEventBus>,
    pub relay: Arc<OutboxRelayService>,
    pub hub: DocHub,
}

pub async fn run() -> anyhow::Result<()> {
    let cfg = Config::from_env()?;
    ed_observability::init_tracing("doc-service", true);

    let pool = PgPool::connect(&cfg.database_url).await?;
    // Issue #225: don't swallow migration failures -- the previous
    // `.await.ok()` would let the service start on a stale schema.
    sqlx::migrate!("../../packages/persistence-postgres/src/migrations")
        .run(&pool).await
        .map_err(|e| anyhow::anyhow!("migration failed: {e}"))?;

    let outbox: Arc<dyn OutboxStore> = Arc::new(EfOutboxStore { pool: pool.clone() });
    let redis = deadpool_redis::Config::from_url(&cfg.redis_url)
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))?;
    let cache = Cache::new(redis);

    let event_bus = ed_messaging_rabbitmq::RabbitEventBus::connect(
        &cfg.rabbit_url,
        ed_messaging_rabbitmq::Topology::default(),
    )
    .await?;
    let event_bus = Arc::new(event_bus) as Arc<dyn IEventBus>;
    let relay = Arc::new(OutboxRelayService {
        store: Arc::clone(&outbox),
        bus: Arc::clone(&event_bus),
        poll_interval: std::time::Duration::from_millis(500),
        batch_size: 50,
        max_attempts: 5,
        backoff_base_ms: 500,
        backoff_max_ms: 60_000,
        relay_id: format!("doc-service@{}", uuid::Uuid::new_v4()),
    });
    let relay_clone = Arc::clone(&relay);
    tokio::spawn(async move { relay_clone.run().await; });

    let app = AppState { pool: pool.clone(), cache, outbox: Arc::clone(&outbox), event_bus: Arc::clone(&event_bus), relay: Arc::clone(&relay), hub: DocHub::default() };

    // Issue #217: enforce internal-JWT auth on every non-healthz route.
    let internal_secret = std::env::var("INTERNAL_SERVICE_TOKEN_SECRET")
        .unwrap_or_else(|_| "dev-only-secret".into());
    let verifier = Arc::new(ed_auth::JwtVerifier::new_from_secret(
        internal_secret.as_bytes(),
        "ed-gateway",
        "internal",
    ));

    let router = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/api/documents", get(list_documents).post(create_document))
        // Issue #222: register the previously-orphaned update handler on PUT.
        .route("/api/documents/{id}",
               get(get_document)
                   .put(update_document)
                   .delete(delete_document))
        .route("/api/v1/doc-service/ws/doc/{id}", get(ws_handler))
        .with_state(app.clone())
        .layer(axum::middleware::from_fn_with_state(verifier.clone(), crate::auth::require_internal_auth))
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = format!("{}:{}", cfg.host, cfg.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(addr = %addr, "doc-service listening");
    Ok(axum::serve(listener, router).await?)
}
