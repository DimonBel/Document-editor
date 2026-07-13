use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;

use crate::config::Config;

pub async fn run() -> anyhow::Result<()> {
    let cfg = Config::from_env();
    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/api/rooms", get(list_rooms).post(create_room))
        .route("/api/rooms/{id}", get(get_room).delete(delete_room))
        .layer(TraceLayer::new_for_http());
    let addr: SocketAddr = format!("{}:{}", cfg.host, cfg.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(addr = %addr, "room-service listening");
    Ok(axum::serve(listener, app).await?)
}

async fn healthz() -> &'static str { "ok" }
async fn list_rooms() -> Json<Value> { Json(json!([])) }
async fn create_room() -> Json<Value> { Json(json!({"id": ""})) }
async fn get_room() -> Json<Value> { Json(json!({})) }
async fn delete_room() -> Json<Value> { Json(json!({})) }
