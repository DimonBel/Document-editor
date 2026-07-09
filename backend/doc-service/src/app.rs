use axum::{routing::get, Router};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use crate::config::Config;
pub async fn run() -> anyhow::Result<()> {
    let cfg = Config::from_env();
    let app = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/api/documents", get(list_documents).post(create_document))
        .route("/api/documents/{id}", get(get_document).delete(delete_document))
        .layer(TraceLayer::new_for_http());
    let addr: SocketAddr = format!("{}:{}", cfg.host, cfg.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    Ok(axum::serve(listener, app).await?)
}
async fn list_documents() -> &'static str { "[]" }
async fn create_document() -> &'static str { "{}" }
async fn get_document() -> &'static str { "{}" }
async fn delete_document() -> &'static str { "" }
