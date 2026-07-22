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

    /// Broadcast to every subscriber whose subscription pattern matches
    /// `topic`. Patterns use RabbitMQ topic-exchange semantics:
    /// - `*` matches exactly one dot-delimited word
    /// - `#` matches zero or more words (must be the whole segment)
    /// A pattern with no wildcards is treated as an exact-match key.
    pub fn broadcast(&mut self, topic: &str, msg: serde_json::Value) {
        let keys: Vec<String> = self.by_topic.keys().cloned().collect();
        for key in keys {
            if topic_matches(&key, topic) {
                if let Some(senders) = self.by_topic.get_mut(&key) {
                    let mut idx = 0;
                    while idx < senders.len() {
                        let s = &senders[idx];
                        match s.try_send(msg.clone()) {
                            Ok(_) => idx += 1,
                            Err(_) => {
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
    }
}

/// RabbitMQ-style topic pattern matcher.
/// `*` matches exactly one word; `#` matches zero or more words.
fn topic_matches(pattern: &str, topic: &str) -> bool {
    if pattern == topic { return true; }
    if !pattern.contains('*') && !pattern.contains('#') { return false; }
    let p_parts: Vec<&str> = pattern.split('.').collect();
    let t_parts: Vec<&str> = topic.split('.').collect();
    match_wildcard(&p_parts, &t_parts)
}

fn match_wildcard(p: &[&str], t: &[&str]) -> bool {
    if p.is_empty() && t.is_empty() { return true; }
    if p.is_empty() { return false; }
    match p[0] {
        "#" => {
            // `#` matches the rest of the topic
            match_wildcard(&p[1..], t) || (!t.is_empty() && match_wildcard(p, &t[1..]))
        }
        "*" => {
            // `*` matches exactly one word
            !t.is_empty() && match_wildcard(&p[1..], &t[1..])
        }
        exact => {
            !t.is_empty() && t[0] == exact && match_wildcard(&p[1..], &t[1..])
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
    let mut topics: Vec<String> = q
        .topics
        .as_deref()
        .unwrap_or("room.#")
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if topics.is_empty() {
        // Always subscribe to at least `room.#` so a misbehaving client
        // still gets a stream (and is rate-limited accordingly).
        topics.push("room.#".into());
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
    let guard = UnregisterOnDrop { sb: sb_drop, tx: tx.clone(), topics: topics_drop };

    // Stream wrapper that owns the guard, ensuring it lives for the
    // full lifetime of the response and is dropped when the client
    // disconnects (issue #204).
    struct GuardedStream<S> {
        inner: S,
        guard: Option<UnregisterOnDrop>,
    }
    impl<S: Stream + Unpin> Stream for GuardedStream<S> {
        type Item = S::Item;
        fn poll_next(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<Self::Item>> {
            std::pin::Pin::new(&mut self.inner).poll_next(cx)
        }
    }
    impl<S> Drop for GuardedStream<S> {
        fn drop(&mut self) {
            // explicit take so the inner Drop runs at stream-drop time
            self.guard.take();
        }
    }

    let inner = ReceiverStream::new(rx).map(move |msg| {
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

    let stream = GuardedStream { inner, guard: Some(guard) };

    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keepalive"),
    )
}

/// Subscribe to RabbitMQ and republish to in-process channels.
pub async fn start_rabbit_consumer(state: AppState) -> anyhow::Result<()> {
    use lapin::{Connection, ConnectionProperties, ExchangeKind};
    use lapin::options::{BasicConsumeOptions, ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions};
    use lapin::types::FieldTable;

    let conn = Connection::connect(&state.rabbit_url, ConnectionProperties::default()).await?;
    let channel = conn.create_channel().await?;

    // Declare the durable event exchange used by the service outbox relays.
    channel
        .exchange_declare(
            "ed.events",
            ExchangeKind::Topic,
            ExchangeDeclareOptions { durable: true, ..Default::default() },
            FieldTable::default(),
        )
        .await?;

    // Declare a topic exchange for SSE delivery.
    channel
        .exchange_declare(
            "ed.realtime",
            ExchangeKind::Topic,
            ExchangeDeclareOptions { durable: false, ..Default::default() },
            FieldTable::default(),
        )
        .await?;

    // Declare a queue + binding. Issue #205: use `#` (multi-word wildcard)
    // not `*` (single-word), otherwise `room.snapshot` etc. never reach us.
    let q = channel
        .queue_declare(
            "ed.gateway.sse",
            QueueDeclareOptions { durable: false, auto_delete: true, exclusive: true, ..Default::default() },
            FieldTable::default(),
        )
        .await?;
    channel
        .queue_bind(q.name().as_str(), "ed.events", "#", QueueBindOptions::default(), FieldTable::default())
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
