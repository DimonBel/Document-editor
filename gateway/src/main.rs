//! `gateway` -- binary entrypoint.

use ed_observability::init_tracing;
use gateway::{app::build_router, config::Config, realtime::start_rabbit_consumer, state::AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing("gateway", true);

    let config = Config::from_env()?;
    let bind = config.bind_addr();
    let state = AppState::new(config).await?;

    // Best-effort: start the RabbitMQ consumer. If Rabbit is down at
    // startup, the gateway still serves health/auth/proxy; SSE will 503
    // until the consumer is up.
    if let Err(e) = start_rabbit_consumer(state.clone()).await {
        tracing::warn!(error = %e, "rabbit consumer not started at boot; will be retried on first SSE connection");
    }

    let router = build_router(state);
    let addr: std::net::SocketAddr = bind.parse()?;
    gateway::app::serve(router, addr).await
}
