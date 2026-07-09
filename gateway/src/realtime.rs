//! Realtime push: RabbitMQ -> Server-Sent Events fanout at `/api/realtime/sse`.
//!
//! On gateway startup, a single background consumer subscribes to
//! `ed.events` (routing key `*`) and republishes messages to per-user
//! channels. Connected SSE clients get events for topics they subscribed
//! to (via `?topics=room.*`).

use axum::{
    extract::{Query, State},
    response::sse::{Event, KeepAlive, Sse},
};
use futures::stream::Stream;
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt as _};

use crate::state::AppState;

#[derive(Debug, serde::Deserialize)]
pub struct SseQuery {
    #[serde(default)]
    pub topics: Option<String>,
}

/// Subscribe a new SSE client. Returns a stream of `Event`s.
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

    let (tx, rx) = unbounded_channel::<serde_json::Value>();
    let id = uuid::Uuid::new_v4().to_string();

    // Register this client
    {
        let mut clients = state.ws_clients.write();
        for t in &topics {
            clients.entry(t.clone()).or_default().push(tx.clone());
        }
    }

    let clients = state.ws_clients.clone();
    let topics_for_cleanup = topics.clone();
    let cleanup_id = id.clone();
    let rx_for_cleanup = tx.clone();

    let stream = UnboundedReceiverStream::new(rx).map(move |msg| {
        let topic = msg.get("topic").and_then(|t| t.as_str()).unwrap_or("").to_string();
        Ok::<_, Infallible>(Event::default().event("event").data(serde_json::to_string(&msg).unwrap_or_default()).id(format!("{}-{}", cleanup_id, uuid::Uuid::new_v4())))
    });

    let sse = Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keepalive"),
    );

    // Spawn cleanup: when the SSE stream is dropped, unregister
    tokio::spawn(async move {
        // We don't have a direct way to know when the stream ends;
        // the receiver will be dropped when the client disconnects.
        // The UnboundedSender will be dropped at the same time.
        drop(rx_for_cleanup);
        let mut clients = clients.write();
        for t in &topics_for_cleanup {
            if let Some(v) = clients.get_mut(t) {
                v.retain(|s| !s.same_channel(&tx));
                if v.is_empty() { clients.remove(t); }
            }
        }
    });

    sse
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
                        // Fan out to all matching subscribers
                        let snapshot: Vec<(String, Vec<UnboundedSender<serde_json::Value>>)> = {
                            let c = clients.read();
                            c.iter()
                                .filter(|(pattern, _)| topic_matches(pattern, &topic))
                                .map(|(k, v)| (k.clone(), v.clone()))
                                .collect()
                        };
                        for (_pattern, senders) in snapshot {
                            for s in senders {
                                let _ = s.send(json.clone());
                            }
                        }
                    }
                }
            }
        }
    });

    Ok(())
}

fn topic_matches(pattern: &str, topic: &str) -> bool {
    if pattern == "*" { return true; }
    if pattern == topic { return true; }
    if let Some(prefix) = pattern.strip_suffix(".*") {
        return topic == prefix || topic.starts_with(&format!("{prefix}."));
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return topic.starts_with(prefix);
    }
    false
}
