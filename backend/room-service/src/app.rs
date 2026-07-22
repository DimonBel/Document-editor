//! `room-service` -- axum + tokio, real vertical slice.
//!
//! Per #146:
//!   - REST CRUD for rooms is real (Mongo + Redis read-through cache).
//!   - WS collaboration at `/api/v1/room-service/ws/room/{id}` relays
//!     ops to peers and persists them to the Postgres outbox.
//!   - Background Postgres outbox relay publishes events to RabbitMQ.

use axum::{
    routing::{get},
    Router,
};
use ed_cache::Cache;
use ed_messaging_rabbitmq::{IEventBus, OutboxRelayService};
use ed_persistence_mongo::MongoRepo;
use sqlx::PgPool;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

use crate::config::Config;
use crate::handlers::{create_room, delete_room, get_room, list_rooms, RoomAppState};
use crate::ws::ws_handler;

pub async fn run() -> anyhow::Result<()> {
    let cfg = Config::from_env()?;
    ed_observability::init_tracing("room-service", true);

    let mongo = ed_persistence_mongo::MongoDb::connect(&cfg.mongo_url, "ed")
        .await
        .map_err(|e| anyhow::anyhow!("mongo: {e}"))?;
    let mongo_repo = MongoRepo::new(mongo);

    let pool = PgPool::connect(&cfg.database_url).await?;
    let outbox: Arc<dyn ed_persistence_postgres::OutboxStore> =
    Arc::new(ed_persistence_postgres::EfOutboxStore { pool: pool.clone() });
    sqlx::migrate!("../../packages/persistence-postgres/src/migrations")
        .run(&pool).await
        .map_err(|e| anyhow::anyhow!("migration failed: {e}"))?;

    let redis_cfg = deadpool_redis::Config::from_url(&cfg.redis_url);
    let redis = redis_cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))?;
    let cache = Cache::new(redis.clone());

    let event_bus = ed_messaging_rabbitmq::RabbitEventBus::connect(
        &cfg.rabbit_url,
        ed_messaging_rabbitmq::Topology::default(),
    )
    .await?;
    let event_bus = Arc::new(event_bus) as Arc<dyn IEventBus>;
    let relay = Arc::new(OutboxRelayService {
        store: Arc::clone(&outbox) as Arc<dyn ed_persistence_postgres::OutboxStore>,
        bus: Arc::clone(&event_bus),
        poll_interval: std::time::Duration::from_millis(500),
        batch_size: 50,
        max_attempts: 5,
        backoff_base_ms: 500,
        relay_id: format!("room-service@{}", uuid::Uuid::new_v4()),
        backoff_max_ms: 60_000,
    });
    let relay_clone = Arc::clone(&relay);
    tokio::spawn(async move { relay_clone.run().await; });

    let room_state = RoomAppState { repo: mongo_repo, cache, hub: crate::ws::RoomHub::default(), outbox };

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
        .route("/api/rooms", get(list_rooms).post(create_room))
        .route("/api/rooms/{id}", get(get_room).delete(delete_room))
        .route("/api/v1/room-service/ws/room/{id}", get(ws_handler))
        .with_state(room_state)
        .layer(axum::middleware::from_fn_with_state(verifier.clone(), crate::auth::require_internal_auth))
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = format!("{}:{}", cfg.host, cfg.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(addr = %addr, "room-service listening");
    Ok(axum::serve(listener, router).await?)
}
