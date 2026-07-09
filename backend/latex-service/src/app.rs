use axum::{routing::{get, post}, Router};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use crate::config::Config;
pub async fn run() -> anyhow::Result<()> {
    let cfg = Config::from_env();
    let app = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/api/latex/compile", post(compile))
        .route("/api/latex/to-docx", post(to_docx))
        .layer(TraceLayer::new_for_http());
    let addr: SocketAddr = format!("{}:{}", cfg.host, cfg.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    Ok(axum::serve(listener, app).await?)
}
async fn compile() -> &'static str { "{"status":"queued"}" }
async fn to_docx() -> &'static str { "{"status":"queued"}" }
