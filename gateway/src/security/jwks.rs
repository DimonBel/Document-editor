//! JWKS endpoint: expose the public key in JWK format.

use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::state::AppState;

pub async fn jwks(State(state): State<AppState>) -> Json<Value> {
    Json(json!({ "keys": [state.keys.public_jwk()] }))
}
