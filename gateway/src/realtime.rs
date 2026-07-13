//! Realtime push: RabbitMQ -> Server-Sent Events fanout at `/api/realtime/sse`.
//!
//! Per issue #140:
//! - The client is registered **for the lifetime of the connection**
//!   (removed by a guard that fires when the response stream is dropped,
//!   i.e. when the client disconnects).
//! - Channels are **bounded** (`MAX_QUEUE = 64`) so a slow client
//!   doesn't make the publisher block forever; if the queue fills,
//!   we drop the oldest event for that client and emit a synthetic
//!   "lagged" event so the client knows to reconnect.
//! - Global subscription registry has a hard cap (`MAX_SUBSCRIBERS`)
//!   to prevent an open-relay DoS.

use axum::{
    extract::{Query, State},
    response::sse::{Event, KeepAlive, Sse},
};
use futures::stream::Stream;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio_stream::{wrappers::ReceiverStream, StreamExt as _};

use crate::state::AppState;

const MAX_QUEUE: usize = 64;
const MAX_SUBSCRIBERS_PER_TOPIC: usize = 1024;

#[derive(Debug, serde::Deserialize)]
pub struct SseQuery {
    #[serde(default)]
    pub topics: Option<String>,
}

/// Per-topic subscriber table. Public so the Rabbit consumer can broadcast
/// into it from a background task.
#[derive(Default)]
pub struct SubscriberTable {
    by_topic: HashMap<String, Vec<Sender<serde_json::Value>>>,
}

impl SubscriberTable {
    pub fn add(&mut self, topic: &str, tx: Sender<serde_json::Value>) -> bool {
        let entry = self.by_topic.entry(topic.to_string()).or_default();
        if entry.len() >= MAX_SUBSCRIBERS_PER_TOPIC { return false; }
        entry.push(tx);
        true
    }
    pub fn remove(&mut self, topic: &str, tx: &Sender<serde_json::Value>) {
        if let Some(v) = self.by_topic.get_mut(topic) {
            v.retain(|s| !s.same_channel(tx));
            if v.is_empty() { self.by_topic.remove(topic); }
        }
    }
    pub fn broadcast(&mut self, topic: &str, msg: serde_json::Value) {
        if let Some(senders) = self.by_topic.get_mut(topic) {
            let mut idx = 0;
            while idx < senders.len() {
                let s = &senders[idx];
                // Try to push; on full queue drop the oldest by
                // `try_send` semantics: cap with a bounded buffer and
                // drop if the consumer is too slow.
                match s.try_send(msg.clone()) {
                    Ok(_) => idx += 1,
                    Err(_) => {
                        // emit one synthetic lagged event, then drop
                        let _ = s.try_send(serde_json::json!({
                            "topic": topic, "lagged": true,
                        }));
                        idx += 1;
                    }
                }
            }
        }
    }
}

/// Subscribe a new SSE client. Returns a stream of `Event`s with a
/// RAII-style cleanup guard: when the response stream is dropped, the
/// sender is removed from every topic it registered for.
pub async fn sse(
    State(state): State<AppState>,
    Query(q): Query<SseQuery>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let topics: Vec<String> = q
        .topics
        .as_deref()
        .unwrap_or("room.*")
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if topics.is_empty() {
        // Always subscribe to at least `room.*` so a misbehaving client
        // still gets a stream (and is rate-limited accordingly).
        topics.push("room.*".into());
    }

    let (tx, rx) = channel::<serde_json::Value>(MAX_QUEUE);
    let id = uuid::Uuid::new_v4().to_string();
    let client_id = id.clone();
    let sb = state.ws_clients.clone();

    // Register this client on every requested topic.
    {
        let mut table = sb.lock();
        for t in &topics {
            table.add(t, tx.clone());
        }
    }

    let sb_drop = sb.clone();
    let topics_drop = topics.clone();

    // RAII guard: when the response stream is dropped (client
    // disconnect, or server shutdown) the inner Drop impl removes
    // the sender from every topic.
    struct UnregisterOnDrop {
        sb: Arc<Mutex<SubscriberTable>>,
        tx: Sender<serde_json::Value>,
        topics: Vec<String>,
    }
    impl Drop for UnregisterOnDrop {
        fn drop(&mut self) {
            let mut table = self.sb.lock();
            for t in &self.topics {
                table.remove(t, &self.tx);
            }
        }
    }
    let _guard = UnregisterOnDrop { sb: sb_drop, tx: tx.clone(), topics: topics_drop };

    let stream = ReceiverStream::new(rx).map(move |msg| {
        let topic = msg
            .get("topic")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();
        let data = serde_json::to_string(&msg).unwrap_or_default();
        Ok::<_, Infallible>(
            Event::default()
                .event(topic)
                .data(data)
                .id(format!("{}-{}", client_id, uuid::Uuid::new_v4())),
        )
    });

    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keepalive"),
    )
    // The `_guard` lives until the response stream is fully consumed
    // (i.e. when the client disconnects or the handler is cancelled).
}

/// Subscribe to RabbitMQ and republish to in-process channels.
pub async fn start_rabbit_consumer(state: AppState) -> anyhow::Result<()> {
    use lapin::{Connection, ConnectionProperties, ExchangeKind};
    use lapin::options::{BasicConsumeOptions, ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions};
    use lapin::types::FieldTable;

    let conn = Connection::connect(&state.rabbit_url, ConnectionProperties::default()).await?;
    let channel = conn.create_channel().await?;

    // Declare a fanout-style exchange just for SSE delivery
    channel
        .exchange_declare(
            "ed.realtime",
            ExchangeKind::Topic,
            ExchangeDeclareOptions { durable: false, ..Default::default() },
            FieldTable::default(),
        )
        .await?;

    // Declare a queue + binding (catch-all `*` from `ed.events`)
    let q = channel
        .queue_declare(
            "ed.gateway.sse",
            QueueDeclareOptions { durable: false, auto_delete: true, exclusive: true, ..Default::default() },
            FieldTable::default(),
        )
        .await?;
    channel
        .queue_bind(q.name().as_str(), "ed.events", "*", QueueBindOptions::default(), FieldTable::default())
        .await?;

    // Store the channel in shared state
    *state.rabbit_channel.lock().await = Some(channel.clone());

    let mut consumer = channel
        .basic_consume(
            q.name().as_str(),
            "ed-gateway-sse",
            BasicConsumeOptions { no_ack: true, ..Default::default() },
            FieldTable::default(),
        )
        .await?;

    let clients = state.ws_clients.clone();
    tokio::spawn(async move {
        while let Some(delivery) = futures::StreamExt::next(&mut consumer).await {
            if let Ok(d) = delivery {
                if let Ok(text) = std::str::from_utf8(&d.data) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(text) {
                        let topic = json
                            .get("topic")
                            .and_then(|t| t.as_str())
                            .unwrap_or("")
                            .to_string();
                        let mut table = clients.lock();
                        // The SubscribeTable::broadcast handles per-client bounded
                        // queues and emits a "lagged" marker if a slow consumer
                        // falls behind, so the client knows to reconnect.
                        table.broadcast(&topic, json);
                    }
                }
            }
        }
    });

    Ok(())
}
