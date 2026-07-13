//! Document WebSocket handler -- relays CRDT ops to peers + outbox.
//!
//! Per #146: this mirrors the room-service WS handler: thin axum
//! endpoint that fans out ops via an in-process `DocHub`, persists
//! ops to the outbox, and lets the relay publish to RabbitMQ.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

use crate::app::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsFrame {
    Op { op: serde_json::Value },
    Cursor { x: f64, y: f64 },
    Pong,
}

#[derive(Default, Clone)]
pub struct DocHub {
    inner: Arc<Mutex<HashMap<String, Vec<UnboundedSender<WsFrame>>>>>,
}
impl DocHub {
    pub fn subscribe(&self, doc: &str) -> UnboundedSender<WsFrame> {
        let (tx, _rx) = unbounded_channel();
        self.inner.lock().entry(doc.to_string()).or_default().push(tx.clone());
        tx
    }
    pub fn unsubscribe(&self, doc: &str, tx: &UnboundedSender<WsFrame>) {
        let mut g = self.inner.lock();
        if let Some(subs) = g.get_mut(doc) {
            subs.retain(|s| !s.same_channel(tx));
            if subs.is_empty() { g.remove(doc); }
        }
    }
    pub fn broadcast(&self, doc: &str, msg: WsFrame) {
        if let Some(subs) = self.inner.lock().get(doc) {
            for s in subs { let _ = s.send(msg.clone()); }
        }
    }
}

pub async fn ws_handler(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
    ws: WebSocketUpgrade,
) -> Response {
    let hub = DocHub::default();
    let outbox = state.outbox.clone();
    ws.on_upgrade(move |socket| async move {
        run(socket, hub, outbox, doc_id).await;
    })
}

async fn run(
    socket: WebSocket,
    hub: DocHub,
    outbox: Arc<dyn ed_persistence_postgres::OutboxStore>,
    doc_id: String,
) {
    let (out_tx, mut out_rx) = unbounded_channel::<WsFrame>();
    hub.inner.lock().entry(doc_id.clone()).or_default().push(out_tx.clone());

    let (mut ws_tx, mut ws_rx) = socket.split();

    let drainer = tokio::spawn(async move {
        while let Some(frame) = out_rx.recv().await {
            let text = match serde_json::to_string(&frame) {
                Ok(s) => s,
                Err(_) => continue,
            };
            if ws_tx.send(Message::Text(text)).await.is_err() { break; }
        }
    });

    while let Some(Ok(msg)) = ws_rx.next().await {
        match msg {
            Message::Text(s) => {
                let Ok(frame) = serde_json::from_str::<WsFrame>(&s) else { continue };
                if matches!(frame, WsFrame::Op { .. }) {
                    use ed_contracts::{topics::document as DT, EventMessage};
                    let envelope = EventMessage::new(
                        DT::UPDATED, "doc.op",
                        serde_json::json!({ "doc_id": doc_id, "frame": frame }),
                        "doc-service",
                    );
                    let _ = outbox.append(&ed_persistence_postgres::make_outbox(
                        DT::UPDATED, "Document", &doc_id, &envelope,
                    )).await;
                }
                hub.broadcast(&doc_id, frame);
            }
            Message::Ping(p) => { let _ = ws_tx.send(Message::Pong(p)).await; }
            Message::Close(_) => break,
            _ => {}
        }
    }
    hub.unsubscribe(&doc_id, &out_tx);
    drainer.abort();
}
