use async_trait::async_trait;
use ed_contracts::EventMessage;
use crate::error::BrokerError;
use lapin::{message::Delivery, Channel};
#[async_trait]
pub trait ConsumerHandler: Send + Sync {
    type Event: serde::de::DeserializeOwned + Send + Sync;
    async fn handle(&self, evt: EventMessage<Self::Event>, raw: &Delivery, ch: &Channel) -> Result<(), BrokerError>;
}
