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
use crate::middleware::{correlation::correlation_middleware, idempotency::idempotency_middleware,
                       logging::logging_middleware, rate_limit::rate_limit_middleware};
use crate::proxy::proxy;
use crate::realtime::sse;
use crate::security::{jwks::jwks, middleware::auth_middleware};
use crate::state::AppState;
use crate::ws::ws_handler;

pub fn build_router(state: AppState) -> Router {
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
        // Reverse proxy: /api/v1/{svc}/{path:path}
        .route("/api/v1/{svc}/{*path}", any(proxy))
        // WebSocket proxy: /ws/{svc}/{path:path}
        .route("/ws/{svc}/{*path}", get(ws_handler))
        // The auth + rate-limit middlewares need access to
        // `JwtVerifier` from `AppState`. The cleanest way to
        // express that in axum 0.7 is an inline closure that
        // captures the state and lets axum fill in the request
        // type; this avoids the generic-type parameter dance
        // that `from_fn_with_state(auth_middleware, ...)` would
        // require.
        .layer(middleware::from_fn(move |req, next| {
            let st = state.clone();
            crate::security::middleware::auth_middleware(
                axum::extract::State(st),
                req,
                next,
            )
        }))
        .layer(middleware::from_fn(move |req, next| {
            let st = state.clone();
            crate::middleware::rate_limit::rate_limit_middleware(
                axum::extract::State(st),
                req,
                next,
            )
        }))
        .layer(middleware::from_fn(idempotency_middleware))
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
