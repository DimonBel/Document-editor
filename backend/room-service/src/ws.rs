//! Whiteboard collaboration WebSocket handler.
//!
//! Per #146: replaces `backend/src/handlers/websocket.rs` with a thin
//! axum WS endpoint. Responsibilities:
//!
//!   1. Forward every inbound client frame to other connected clients
//!      on the same replica via the in-process `RoomHub`.
//!   2. Persist `Op` frames to the Postgres outbox (relay publishes
//!      to RabbitMQ) so other replicas see them.
//!
//! External connections are terminated by `gateway/src/ws.rs` and
//! forwarded with `Authorization: Internal <internal-token>`; no
//! per-message JWT here.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

use crate::handlers::RoomAppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsFrame {
    Op { op: serde_json::Value },
    Cursor { x: f64, y: f64 },
    Pong,
}

/// In-process broadcast hub.
#[derive(Default, Clone)]
pub struct RoomHub {
    inner: Arc<Mutex<HashMap<String, Vec<UnboundedSender<WsFrame>>>>>,
}
impl RoomHub {
    /// Register a new subscriber and return its sender. The
    /// receiver is dropped when the consumer task ends -- at which
    /// point `unsubscribe` should be called explicitly to remove
    /// the sender from the registry.
    pub fn subscribe(&self, room: &str) -> UnboundedSender<WsFrame> {
        let (tx, _rx) = unbounded_channel();
        self.inner
            .lock()
            .entry(room.to_string())
            .or_default()
            .push(tx.clone());
        // Drop the local receiver; the consumer task will create
        // its own with `mpsc::unbounded_channel`.
        tx
    }
    pub fn unsubscribe(&self, room: &str, tx: &UnboundedSender<WsFrame>) {
        let mut g = self.inner.lock();
        if let Some(subs) = g.get_mut(room) {
            subs.retain(|s| !s.same_channel(tx));
            if subs.is_empty() { g.remove(room); }
        }
    }
    pub fn broadcast(&self, room: &str, msg: WsFrame) {
        if let Some(subs) = self.inner.lock().get(room) {
            for s in subs { let _ = s.send(msg.clone()); }
        }
    }
}

pub async fn ws_handler(
    State(state): State<RoomAppState>,
    Path(room_id): Path<String>,
    ws: WebSocketUpgrade,
) -> Response {
    let hub = state.hub.clone();
    let outbox = state.outbox.clone();
    ws.on_upgrade(move |socket| async move {
        run(socket, hub, outbox, room_id).await;
    })
}

async fn run(
    socket: WebSocket,
    hub: RoomHub,
    outbox: Arc<dyn ed_persistence_postgres::OutboxStore>,
    room_id: String,
) {
    let (out_tx, mut out_rx) = unbounded_channel::<WsFrame>();
    hub.inner.lock()
        .entry(room_id.clone())
        .or_default()
        .push(out_tx.clone());

    let (mut ws_tx, mut ws_rx) = socket.split();

    // Drainer: hub -> WS.
    // Process hub messages and client messages on one task so the WebSocket
    // sink is never moved into a competing writer task.
    loop {
        let msg = tokio::select! {
            Some(frame) = out_rx.recv() => {
                let text = match serde_json::to_string(&frame) { Ok(s) => s, Err(_) => continue };
                if ws_tx.send(Message::Text(text)).await.is_err() { break; }
                continue;
            }
            message = ws_rx.next() => message,
        };
        let Some(Ok(msg)) = msg else { break; };
        match msg {
            Message::Text(s) => {
                let Ok(frame) = serde_json::from_str::<WsFrame>(&s) else { continue };
                match frame {
                    WsFrame::Op { .. } => {
                        use ed_contracts::{topics::room as RT, EventMessage};
                        let envelope = EventMessage::new(
                            RT::UPDATED, "room.op",
                            serde_json::json!({ "room_id": room_id, "frame": frame.clone() }),
                            "room-service",
                        );
                        let _ = outbox.append(&ed_persistence_postgres::make_outbox(
                            RT::UPDATED, "Room", &room_id, &envelope,
                        )).await;
                    }
                    _ => {}
                }
                hub.broadcast(&room_id, frame);
            }
            Message::Ping(p) => { let _ = ws_tx.send(Message::Pong(p)).await; }
            Message::Close(_) => break,
            _ => {}
        }
    }
    hub.unsubscribe(&room_id, &out_tx);
}
