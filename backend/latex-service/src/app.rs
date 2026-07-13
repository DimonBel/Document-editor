use axum::{routing::{get, post}, Json, Router};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;

use crate::config::Config;

pub async fn run() -> anyhow::Result<()> {
    let cfg = Config::from_env();
    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/api/latex/compile", post(compile))
        .route("/api/latex/to-docx", post(to_docx))
        .layer(TraceLayer::new_for_http());
    let addr: SocketAddr = format!("{}:{}", cfg.host, cfg.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(addr = %addr, "latex-service listening");
    Ok(axum::serve(listener, app).await?)
}

async fn healthz() -> &'static str { "ok" }
async fn compile() -> Json<Value> { Json(json!({"status": "queued"})) }
async fn to_docx() -> Json<Value> { Json(json!({"status": "queued"})) }
