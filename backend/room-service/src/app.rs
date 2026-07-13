//! `room-service` -- axum + tokio, real vertical slice.
//!
//! Per #146:
//!   - REST CRUD for rooms is real (Mongo + Redis read-through cache).
//!   - WS collaboration at `/api/v1/room-service/ws/room/{id}` relays
//!     ops to peers and persists them to the Postgres outbox.
//!   - Background Postgres outbox relay publishes events to RabbitMQ.

use axum::{
    extract::State,
    routing::{delete, get, post},
    Json, Router,
};
use ed_cache::Cache;
use ed_messaging_rabbitmq::{IEventBus, OutboxRelayService};
use ed_persistence_mongo::MongoRepo;
use parking_lot::Mutex;
use sqlx::PgPool;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

use crate::config::Config;
use crate::handlers::{create_room, delete_room, get_room, list_rooms, RoomAppState, RoomDoc};
use crate::ws::{ws_handler, RoomHub};

#[derive(Clone)]
pub struct AppState {
    pub room: RoomAppState,
    pub hub: RoomHub,
    pub event_bus: Arc<dyn IEventBus>,
    pub relay: Arc<OutboxRelayService>,
    pub outbox: Arc<dyn ed_persistence_postgres::OutboxStore>,
}

pub async fn run() -> anyhow::Result<()> {
    let cfg = Config::from_env();
    ed_observability::init_tracing("room-service", true);

    let mongo = ed_persistence_mongo::MongoDb::connect(&cfg.mongo_url, "ed")
        .await
        .map_err(|e| anyhow::anyhow!("mongo: {e}"))?;
    let mongo_repo = MongoRepo::new(mongo);

    let pool = PgPool::connect(&cfg.database_url).await?;
    let outbox: Arc<dyn ed_persistence_postgres::OutboxStore> =
        Arc::new(ed_persistence_postgres::EfOutboxStore::new(pool.clone()));
    sqlx::migrate!("packages/persistence-postgres/src/migrations").run(&pool).await.ok();

    let redis_cfg = deadpool_redis::Config::from_url(&cfg.redis_url);
    let redis = redis_cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))?;
    let cache = Cache::new(redis.clone());

    let event_bus = ed_messaging_rabbitmq::RabbitEventBus::connect(
        &cfg.rabbitmq_url,
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
        backoff_max_ms: 60_000,
    });
    let relay_clone = Arc::clone(&relay);
    tokio::spawn(async move { relay_clone.run().await; });

    let hub = RoomHub::default();

    let room_state = RoomAppState { repo: mongo_repo, cache };
    let app = AppState { room: room_state, hub, event_bus, relay, outbox };

    let router = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/api/rooms", get(list_rooms).post(create_room))
        .route("/api/rooms/{id}", get(get_room).delete(delete_room))
        .route("/api/v1/room-service/ws/room/{id}",
               get({
                   let s = app.clone();
                   move |Path(id), ws| ws_handler(State(s), Path(id), ws)
               }))
        .with_state(app.clone())
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = format!("{}:{}", cfg.host, cfg.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(addr = %addr, "room-service listening");
    Ok(axum::serve(listener, router).await?)
}
