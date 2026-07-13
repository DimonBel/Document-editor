use axum::{routing::{get, post}, Json, Router};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;

use crate::config::Config;

pub async fn run() -> anyhow::Result<()> {
    let cfg = Config::from_env();
    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/api/documents", get(list_documents).post(create_document))
        .route("/api/documents/{id}", get(get_document).delete(delete_document))
        .layer(TraceLayer::new_for_http());
    let addr: SocketAddr = format!("{}:{}", cfg.host, cfg.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(addr = %addr, "doc-service listening");
    Ok(axum::serve(listener, app).await?)
}

async fn healthz() -> &'static str { "ok" }
async fn list_documents() -> Json<Value> { Json(json!([])) }
async fn create_document() -> Json<Value> { Json(json!({})) }
async fn get_document() -> Json<Value> { Json(json!({})) }
async fn delete_document() -> Json<Value> { Json(json!({})) }
