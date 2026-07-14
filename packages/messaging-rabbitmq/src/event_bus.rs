//! Async-agnostic event-bus trait + the lapin-based implementation.
//!
//! Per the build errors during the docker compose build, the trait
//! must be **dyn-compatible** to live behind `Arc<dyn IEventBus>`,
//! which means it cannot have generic methods (`async fn publish<T: ...>`).
//! The fix is to make `publish` non-generic: it accepts a
//! pre-serialized `serde_json::Value` (which is what we send over
//! AMQP anyway), and concrete types are converted to it via
//! `serde_json::to_value` at the call site.

use async_trait::async_trait;
use lapin::{Channel, Connection, ConnectionProperties, options::BasicPublishOptions, BasicProperties};
use std::sync::Arc;
use tokio::sync::Mutex;
use ed_contracts::{EventMessage, IEventMessage};
use serde::Serialize;
use crate::error::BrokerError;
use crate::topology::{Topology, TopologyDeclaration};

#[async_trait]
pub trait IEventBus: Send + Sync {
    /// Publish a pre-serialized payload. The signature is generic-free
    /// so the trait is dyn-compatible (can live behind `Arc<dyn IEventBus>`).
    /// Callers that have a `T: Serialize` value use `publish_value`,
    /// which serializes + delegates.
    async fn publish_value(&self, topic: &str, payload: serde_json::Value) -> Result<(), BrokerError>;
    /// Convenience wrapper for `T: Serialize` callers: serializes
    /// `value` to JSON, then forwards to `publish_value`. Not part of
    /// the trait object (would break dyn-compat), so a default impl
    /// is provided for the concrete `RabbitEventBus`.
    fn channel(&self) -> Channel;
}

pub struct RabbitEventBus {
    pub conn: Arc<Mutex<Connection>>,
    pub channel: Channel,
    pub topology: Topology,
}

impl RabbitEventBus {
    pub async fn connect(url: &str, topology: Topology) -> Result<Self, BrokerError> {
        let conn = Connection::connect(url, ConnectionProperties::default()).await?;
        let channel = conn.create_channel().await?;
        topology.declare(&channel).await?;
        Ok(Self { conn: Arc::new(Mutex::new(conn)), channel, topology })
    }

    /// Concrete-class helper: serialize a typed message, then call
    /// `publish_value`. This is the path used by the gateway
    /// and the room/doc services -- they call `bus.publish(&msg)`
    /// (with a typed `T`), which lands here.
    pub async fn publish<T: Serialize + Send + Sync>(
        &self, topic: &str, evt: &EventMessage<T>,
    ) -> Result<(), BrokerError> {
        let payload = serde_json::to_value(evt)
            .map_err(|e| BrokerError::Internal(format!("serialize: {e}")))?;
        self.publish_value(topic, payload).await
    }
}

#[async_trait]
impl IEventBus for RabbitEventBus {
    async fn publish_value(
        &self, topic: &str, payload: serde_json::Value,
    ) -> Result<(), BrokerError> {
        let correlation = payload.get("correlation_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let id = payload.get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let bytes = serde_json::to_vec(&payload)?;
        let confirm = self.channel
            .basic_publish(
                "ed.events", topic,
                BasicPublishOptions { mandatory: true, ..Default::default() },
                &bytes,
                BasicProperties::default()
                    .with_content_type("application/json".into())
                    .with_correlation_id(correlation.into())
                    .with_message_id(id.into()),
            )
            .await?
            .await?;
        if confirm.is_nack() { return Err(BrokerError::NotConnected); }
        Ok(())
    }
    fn channel(&self) -> Channel { self.channel.clone() }
}
