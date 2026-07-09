use lapin::{Channel, options::BasicPublishOptions, BasicProperties};
use serde::Serialize;
use ed_contracts::EventMessage;
use crate::error::BrokerError;
pub struct HubProducer { pub channel: Channel }
impl HubProducer {
    pub async fn send<T: Serialize>(&self, exchange: &str, topic: &str, evt: &EventMessage<T>) -> Result<(), BrokerError> {
        let payload = serde_json::to_vec(evt)?;
        self.channel.basic_publish(exchange, topic, BasicPublishOptions { mandatory: true, ..Default::default() }, &payload,
            BasicProperties::default().with_content_type("application/json".into())
                .with_correlation_id(evt.correlation_id().to_string().into())
                .with_message_id(evt.id().to_string().into())).await?.await?;
        Ok(())
    }
}
