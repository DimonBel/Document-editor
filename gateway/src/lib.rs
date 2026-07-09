//! `gateway` -- Rust API gateway (replaces the previous Python FastAPI one).
//!
//! Responsibilities:
//! - **Auth**: RS256 JWT issuer, JWKS endpoint, login/refresh/internal-token routes
//! - **Reverse-proxy**: `/api/v1/{svc}/{path:path}` -> upstream Rust service
//! - **WebSocket proxy**: `/ws/{svc}/{path:path}` -> upstream Rust service
//! - **Rate limiting**: Redis token-bucket per (user or IP)
//! - **Idempotency**: `Idempotency-Key` header replay (Redis-backed)
//! - **Correlation**: `X-Correlation-Id` propagation across hops
//! - **Realtime push**: RabbitMQ -> Server-Sent Events fanout at `/api/realtime/sse`
//!
//! Configuration via env (see `config.rs`).

pub mod config;
pub mod error;
pub mod state;
pub mod security;
pub mod auth;
pub mod proxy;
pub mod ws;
pub mod middleware;
pub mod realtime;
pub mod health;
pub mod app;
