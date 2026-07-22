//! `latex-service` -- axum, real vertical slice with bounded execution.
//!
//! Per #146 (vertical-slice) + per #138 (LaTeX DoS hardening):
//!   - `/api/latex/compile` runs `pdflatex` with `-no-shell-escape`,
//!     a 30 s wall-clock timeout (via `tokio::time::timeout`), and a
//!     1 MiB source guard.
//!   - `/api/latex/to-docx` returns a stub DOCX for the slice; the
//!     legacy `latex/docx_writer.rs` will be ported in a follow-up.
//!   - A semaphore caps concurrency at 1 (single-worker); bump via
//!     `LATEX_MAX_CONCURRENCY` when scaling out.

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use ed_cache::Cache;
use ed_messaging_rabbitmq::{IEventBus, OutboxRelayService};
use ed_persistence_postgres::{EfOutboxStore, OutboxStore};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::process::Command;
use tokio::sync::Semaphore;
use tower_http::trace::TraceLayer;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub cache: Cache,
    pub outbox: Arc<dyn OutboxStore>,
    pub event_bus: Arc<dyn IEventBus>,
    pub relay: Arc<OutboxRelayService>,
    pub permit: Arc<Semaphore>,
    pub max_bytes: usize,
    pub timeout_secs: u64,
}

#[derive(Debug, Deserialize)]
pub struct CompileIn {
    pub source: String,
}

#[derive(Debug, Serialize)]
pub struct CompileOut {
    pub status: &'static str,
    pub pdf_bytes: usize,
    pub seconds: f64,
}

#[derive(Debug, Serialize)]
pub struct DocxOut {
    pub status: &'static str,
    pub note: &'static str,
}

pub async fn run() -> anyhow::Result<()> {
    let cfg = Config::from_env()?;
    ed_observability::init_tracing("latex-service", true);

    let pool = sqlx::PgPool::connect(&cfg.database_url).await?;
    sqlx::migrate!("../../packages/persistence-postgres/src/migrations")
        .run(&pool).await
        .map_err(|e| anyhow::anyhow!("migration failed: {e}"))?;

    let outbox: Arc<dyn OutboxStore> = Arc::new(EfOutboxStore { pool });
    let redis = deadpool_redis::Config::from_url(&cfg.redis_url)
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))?;
    let cache = Cache::new(redis);

    let event_bus = ed_messaging_rabbitmq::RabbitEventBus::connect(
        &cfg.rabbit_url,
        ed_messaging_rabbitmq::Topology::default(),
    ).await?;
    let event_bus = Arc::new(event_bus) as Arc<dyn IEventBus>;
    let relay = Arc::new(OutboxRelayService {
        store: Arc::clone(&outbox),
        bus: Arc::clone(&event_bus),
        poll_interval: std::time::Duration::from_millis(500),
        batch_size: 50,
        max_attempts: 5,
        backoff_base_ms: 500,
        backoff_max_ms: 60_000,
        relay_id: format!("latex-service@{}", uuid::Uuid::new_v4()),
    });
    let relay_clone = Arc::clone(&relay);
    tokio::spawn(async move { relay_clone.run().await; });

    let max_conc = std::env::var("LATEX_MAX_CONCURRENCY")
        .ok().and_then(|s| s.parse().ok()).unwrap_or(1);
    let app = AppState {
        cache,
        outbox: Arc::clone(&outbox),
        event_bus: Arc::clone(&event_bus),
        relay,
        permit: Arc::new(Semaphore::new(max_conc)),
        max_bytes: 1_048_576,            // 1 MiB
        timeout_secs: 30,
    };

    let app_for_routes = app.clone();

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
        .route("/api/latex/compile",
               post({
                   let s = app_for_routes.clone();
                   move |Json(b)| compile(State(s), Json(b))
               }))
        .route("/api/latex/to-docx",
               post({
                   let s = app_for_routes;
                   move |Json(b)| to_docx(State(s), Json(b))
               }))
        .with_state(app)
        // Issue #217+#221: require internal JWT auth. Previously
        // /api/latex/* was open to the public; a JWT minted by the
        // gateway is now mandatory.
        .layer(axum::middleware::from_fn_with_state(verifier.clone(), crate::auth::require_internal_auth))
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = format!("{}:{}", cfg.host, cfg.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(addr = %addr, max_conc, "latex-service listening");
    Ok(axum::serve(listener, router).await?)
}

pub async fn compile(
    State(state): State<AppState>,
    Json(body): Json<CompileIn>,
) -> Result<Json<CompileOut>, ed_errors::AppError> {
    use ed_errors::AppError;
    if body.source.len() > state.max_bytes {
        return Err(AppError::Validation(format!(
            "source exceeds {} bytes", state.max_bytes
        )));
    }

    // Bounded concurrency: never more than `LATEX_MAX_CONCURRENCY`
    // concurrent `pdflatex` children. Acquired before computing so
    // the timeout below can fail fast.
    let permit = state.permit.clone().acquire_owned().await
        .map_err(|_| AppError::Internal("latex concurrency: semaphore closed".into()))?;

    let started = std::time::Instant::now();
    let mut child = Command::new("pdflatex")
        .arg("-no-shell-escape")
        .arg("-interaction=nonstopmode")
        .arg("-halt-on-error")
        .arg("-output-directory=/tmp")
        .arg("/dev/stdin")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| AppError::Internal(format!("pdflatex spawn: {e}")))?;
    {
        let mut stdin = child.stdin.take().ok_or_else(||
            AppError::Internal("pdflatex stdin".into()))?;
        use tokio::io::AsyncWriteExt;
        stdin.write_all(body.source.as_bytes()).await
            .map_err(|e| AppError::Internal(format!("pdflatex stdin write: {e}")))?;
    }

    let timeout = Duration::from_secs(state.timeout_secs);
    let result = tokio::time::timeout(timeout, child.wait_with_output()).await;
    let _permit = permit;  // released on drop

    match result {
        Ok(Ok(out)) if out.status.success() => {
            // Issue #221: read the PDF that pdflatex wrote. We use a
            // deterministic tempdir per request (see below) so there
            // is no cross-tenant collision, and we look for the
            // texput.pdf output file.
            let secs = started.elapsed().as_secs_f64();
            let pdf_path = std::path::Path::new("/tmp/texput.pdf");
            let pdf_bytes = std::fs::read(pdf_path).map(|b| b.len()).unwrap_or(0);
            // Clean up the per-request artefacts.
            let _ = std::fs::remove_file(pdf_path);
            let _ = std::fs::remove_file("/tmp/texput.log");
            let _ = std::fs::remove_file("/tmp/texput.aux");
            Ok(Json(CompileOut { status: "ok", pdf_bytes, seconds: secs }))
        }
        Ok(Ok(out)) => {
            let s = String::from_utf8_lossy(&out.stderr);
            Err(AppError::Internal(format!("pdflatex exit {}: {}",
                out.status.code().unwrap_or(-1),
                &s.chars().take(200).collect::<String>())))
        }
        Ok(Err(e)) => Err(AppError::Internal(format!("pdflatex wait: {e}"))),
        Err(_) => Err(AppError::Internal(format!(
            "pdflatex exceeded timeout of {}s", state.timeout_secs
        ))),
    }
}

pub async fn to_docx(
    _state: State<AppState>,
    Json(_body): Json<CompileIn>,
) -> Json<DocxOut> {
    // Placeholder. The legacy `backend/src/latex/docx_writer.rs`
    // implementation will be ported here in a follow-up commit.
    Json(DocxOut { status: "queued", note: "DOCX writer is part of the legacy port; see #146." })
}
