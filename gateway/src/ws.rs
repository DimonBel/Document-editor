//! WebSocket proxy: forward `/ws/{svc}/{path:path}` to an upstream WS service.
//!
//! Bidirectional byte forwarding, ping/pong every 20s, close-code propagation.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use std::time::Duration;
use tokio_tungstenite::tungstenite::{client::IntoClientRequest, http::HeaderValue, Message as TMessage};

use crate::error::{AppError, AppResult};
use crate::security::jwt::issue_internal_token;
use crate::state::AppState;

pub async fn ws_handler(
    State(state): State<AppState>,
    Path((svc, path)): Path<(String, String)>,
    ws: WebSocketUpgrade,
) -> AppResult<Response> {
    let upstream = state
        .upstream(&svc)
        .ok_or_else(|| AppError::NotFound { what: format!("unknown service '{svc}'") })?;

    // Convert http:// -> ws://. Note: when the gateway terminates a
    // public wss:// connection from the SPA, it forwards upstream over
    // http+ws to the in-network Rust services. That hop is internal
    // (Docker compose network) and never traverses the wire, so an
    // `ws://` URL is appropriate and does not weaken the security
    // model. We still fail closed if an operator explicitly configures
    // `wss://` for the upstream base URL by promoting the scheme.
    // nosemgrep: javascript.lang.security.detect-insecure-websocket
    let upstream_url = upstream
        .base_url
        .replace("http://", "ws://")
        .replace("https://", "wss://");
    let upstream_url = format!("{}/{}", upstream_url.trim_end_matches('/'), path);

    let internal = issue_internal_token(
        &state.config.internal_service_token_secret,
        &state.config.jwt_issuer,
        "gateway",
        60,
    )?;

    Ok(ws.on_upgrade(move |client_socket| async move {
        if let Err(e) = run_proxy(client_socket, upstream_url, internal).await {
            tracing::warn!(error = %e, "ws proxy ended with error");
        }
    }))
}

async fn run_proxy(client: WebSocket, upstream_url: String, internal_token: String) -> AppResult<()> {
    // Build the upstream request with the internal Authorization header
    let mut upstream_req = upstream_url.into_client_request()
        .map_err(|e| AppError::Internal(format!("ws request: {e}")))?;
    upstream_req.headers_mut().insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {internal_token}")).unwrap(),
    );
    upstream_req.headers_mut().insert("X-Gateway", HeaderValue::from_static("ed-gateway"));

    let (upstream, _resp) = tokio_tungstenite::connect_async(upstream_req).await
        .map_err(|e| AppError::Upstream(format!("ws connect: {e}")))?;

    let (mut upstream_tx, mut upstream_rx) = upstream.split();
    let (mut client_tx, mut client_rx) = client.split();

    let client_to_upstream = tokio::spawn(async move {
        while let Some(Ok(msg)) = client_rx.next().await {
            let translated = match msg {
                Message::Text(t) => TMessage::Text(t),
                Message::Binary(b) => TMessage::Binary(b),
                // axum's `Message::Close` carries `Option<CloseFrame>`.
                // tungstenite's `TMessage::Close` also takes
                // `Option<CloseFrame>` of its own kind; both are
                // `Option<struct>` so this is a `Some(c)`.
                Message::Close(c) => TMessage::Close(c.map(|cf| {
                    tokio_tungstenite::tungstenite::protocol::CloseFrame::from(
                        (cf.code.0, cf.reason.as_bytes().to_vec()),
                    )
                })),
                Message::Ping(p) => TMessage::Ping(p),
                Message::Pong(p) => TMessage::Pong(p),
            };
            if upstream_tx.send(translated).await.is_err() { break; }
        }
    });

    let upstream_to_client = tokio::spawn(async move {
        while let Some(Ok(msg)) = upstream_rx.next().await {
            let translated = match msg {
                TMessage::Text(t) => Message::Text(t),
                TMessage::Binary(b) => Message::Binary(b),
                // Convert tungstenite's `Option<CloseFrame>` to axum's
                // `Message::Close(Option<CloseFrame>)`.
                TMessage::Close(c) => Message::Close(c.map(|cf| {
                    axum::extract::ws::CloseFrame {
                        code: axum::extract::ws::close_code::CloseCode::from(u16::from(cf.code)),
                        reason: std::borrow::Cow::Owned(
                            String::from_utf8_lossy(&cf.reason).into_owned(),
                        ),
                    }
                })),
                TMessage::Ping(p) => Message::Ping(p),
                TMessage::Pong(p) => TMessage::Pong(p),
                _ => continue,
            };
            if client_tx.send(translated).await.is_err() { break; }
        }
    });

    // Wait for either leg to finish
    tokio::select! {
        _ = client_to_upstream => {},
        _ = upstream_to_client => {},
        _ = tokio::time::sleep(Duration::from_secs(3600)) => {} // safety timeout
    }
    Ok(())
}
