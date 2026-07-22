//! Build the `axum::Router` for the gateway.

use axum::{
    http::{header, Method},
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
    let st_auth = state.clone();
    let st_rate = state.clone();
    let st_idem = state.clone();

    // Issue #243: replace CorsLayer::permissive() with a constrained
    // allowlist sourced from `GATEWAY_ALLOWED_ORIGINS` (comma-separated).
    // Local dev defaults to `http://localhost:5173` (Vite) and the SPA
    // hostnames.
    let allowed_origins = std::env::var("GATEWAY_ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:5173,http://localhost:8080".into());
    let origins: Vec<axum::http::HeaderValue> = allowed_origins
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
        .allow_credentials(true);

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
        .layer(cors)
        .with_state(state)
}

pub async fn serve(router: Router, addr: SocketAddr) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "gateway listening");
    axum::serve(listener, router).await?;
    Ok(())
}
