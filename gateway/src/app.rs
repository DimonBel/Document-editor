//! Build the `axum::Router` for the gateway.

use axum::{
    middleware,
    routing::{any, get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::auth::{internal_token, login, refresh};
use crate::health::healthz;
use crate::middleware::{
    correlation::correlation_middleware, idempotency::idempotency_middleware,
    logging::logging_middleware, rate_limit::rate_limit_middleware,
};
use crate::proxy::proxy;
use crate::realtime::sse;
use crate::security::{jwks::jwks, middleware::auth_middleware};
use crate::state::AppState;
use crate::ws::ws_handler;

pub fn build_router(state: AppState) -> Router {
    // Pre-clone `AppState` so each closure can move its own
    // owned copy (axum's `from_fn` requires `Fn` + `Send +
    // 'static`, and `move` closures capture by value). The
    // original `state` is reserved for `with_state` at the end.
    let st_auth = state.clone();
    let st_rate = state.clone();
    let st_idem = state.clone();

    Router::new()
        // Public
        .route("/healthz", get(healthz))
        .route("/.well-known/jwks.json", get(jwks))
        // Auth (no JWT required)
        .route("/auth/login", post(login))
        .route("/auth/refresh", post(refresh))
        .route("/auth/internal", post(internal_token))
        // Realtime SSE
        .route("/api/realtime/sse", get(sse))
        // Reverse proxy: /api/v1/{svc}/{path:path}. Axum 0.8 requires a
        // catch-all parameter to be the only parameter in its route, so the
        // handlers split the service name from the captured path.
        .route("/api/v1/*path", any(proxy))
        // WebSocket proxy: /ws/{svc}/{path:path}
        .route("/ws/*path", get(ws_handler))
        .layer(middleware::from_fn(move |req, next| {
            auth_middleware(axum::extract::State(st_auth.clone()), req, next)
        }))
        .layer(middleware::from_fn(move |req, next| {
            rate_limit_middleware(axum::extract::State(st_rate.clone()), req, next)
        }))
        .layer(middleware::from_fn(move |req, next| {
            idempotency_middleware(axum::extract::State(st_idem.clone()), req, next)
        }))
        .layer(middleware::from_fn(correlation_middleware))
        .layer(middleware::from_fn(logging_middleware))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

pub async fn serve(router: Router, addr: SocketAddr) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "gateway listening");
    axum::serve(listener, router).await?;
    Ok(())
}
