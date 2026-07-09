//! Health + JWKS endpoints.

use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::state::AppState;

pub async fn healthz(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": state.config.service_name,
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
