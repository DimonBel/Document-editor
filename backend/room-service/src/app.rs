use axum::{routing::get, Router};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use crate::config::Config;
pub async fn run() -> anyhow::Result<()> {
    let cfg = Config::from_env();
    let app = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/api/rooms", get(list_rooms).post(create_room))
        .route("/api/rooms/{id}", get(get_room).delete(delete_room))
        .layer(TraceLayer::new_for_http());
    let addr: SocketAddr = format!("{}:{}", cfg.host, cfg.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(?addr, "room-service listening");
    Ok(axum::serve(listener, app).await?)
}
async fn list_rooms() -> &'static str { "[]" }
async fn create_room() -> &'static str { "{"id":""}" }
async fn get_room() -> &'static str { "{}" }
async fn delete_room() -> &'static str { "" }
