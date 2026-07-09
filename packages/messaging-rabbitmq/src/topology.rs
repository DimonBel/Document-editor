use serde::{Deserialize, Serialize};
use crate::error::BrokerError;
use lapin::{Channel, ExchangeKind, options::*, types::FieldTable};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeSpec { pub name: String, pub kind: String, pub durable: bool }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueSpec { pub name: String, pub durable: bool, #[serde(default)] pub auto_delete: bool, #[serde(default)] pub dead_letter_exchange: Option<String> }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindingSpec { pub source: String, pub destination: String, pub routing_key: String }
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Topology { #[serde(default)] pub exchanges: Vec<ExchangeSpec>, #[serde(default)] pub queues: Vec<QueueSpec>, #[serde(default)] pub bindings: Vec<BindingSpec> }
#[async_trait::async_trait]
pub trait TopologyDeclaration: Send + Sync { async fn declare(&self, ch: &Channel) -> Result<(), BrokerError>; }
#[async_trait::async_trait]
impl TopologyDeclaration for Topology {
    async fn declare(&self, ch: &Channel) -> Result<(), BrokerError> {
        for ex in &self.exchanges {
            let kind = match ex.kind.as_str() { "topic" => ExchangeKind::Topic, "fanout" => ExchangeKind::Fanout, "direct" => ExchangeKind::Direct, _ => ExchangeKind::Headers };
            ch.exchange_declare(&ex.name, kind, ExchangeDeclareOptions { durable: ex.durable, ..Default::default() }, FieldTable::default()).await?;
        }
        for q in &self.queues {
            let mut args = FieldTable::default();
            if let Some(dlx) = &q.dead_letter_exchange { args.insert("x-dead-letter-exchange".into(), lapin::types::AMQPValue::LongString(dlx.clone().into())); }
            ch.queue_declare(&q.name, QueueDeclareOptions { durable: q.durable, auto_delete: q.auto_delete, ..Default::default() }, args).await?;
        }
        for b in &self.bindings {
            ch.queue_bind(&b.destination, &b.source, &b.routing_key, QueueBindOptions::default(), FieldTable::default()).await?;
        }
        Ok(())
    }
}
