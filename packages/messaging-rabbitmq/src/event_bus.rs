use async_trait::async_trait;
use lapin::{Channel, Connection, ConnectionProperties, options::BasicPublishOptions, BasicProperties};
use std::sync::Arc;
use tokio::sync::Mutex;
use ed_contracts::EventMessage;
use crate::error::BrokerError;
use crate::topology::{Topology, TopologyDeclaration};
#[async_trait]
pub trait IEventBus: Send + Sync {
    async fn publish<T: serde::Serialize + Send + Sync>(&self, topic: &str, evt: &EventMessage<T>) -> Result<(), BrokerError>;
    fn channel(&self) -> Channel;
}
pub struct RabbitEventBus { pub conn: Arc<Mutex<Connection>>, pub channel: Channel, pub topology: Topology }
impl RabbitEventBus {
    pub async fn connect(url: &str, topology: Topology) -> Result<Self, BrokerError> {
        let conn = Connection::connect(url, ConnectionProperties::default()).await?;
        let channel = conn.create_channel().await?;
        topology.declare(&channel).await?;
        Ok(Self { conn: Arc::new(Mutex::new(conn)), channel, topology })
    }
}
#[async_trait]
impl IEventBus for RabbitEventBus {
    async fn publish<T: serde::Serialize + Send + Sync>(&self, topic: &str, evt: &EventMessage<T>) -> Result<(), BrokerError> {
        let payload = serde_json::to_vec(evt)?;
        let confirm = self.channel.basic_publish("ed.events", topic, BasicPublishOptions { mandatory: true, ..Default::default() }, &payload,
            BasicProperties::default().with_content_type("application/json".into())
                .with_correlation_id(evt.correlation_id().to_string().into())
                .with_message_id(evt.id().to_string().into())
        ).await?.await?;
        if confirm.is_nack() { return Err(BrokerError::NotConnected); }
        Ok(())
    }
    fn channel(&self) -> Channel { self.channel.clone() }
}
