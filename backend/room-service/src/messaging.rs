use ed_messaging_rabbitmq::ConsumerHandler;
use ed_contracts::EventMessage;
use ed_contracts::events::document::DocumentCommitRecordedEvent;
use async_trait::async_trait;
use lapin::{message::Delivery, Channel};
use ed_messaging_rabbitmq::BrokerError;

pub struct DocumentCommitConsumer;
#[async_trait]
impl ConsumerHandler for DocumentCommitConsumer {
    type Event = DocumentCommitRecordedEvent;
    async fn handle(&self, evt: EventMessage<Self::Event>, _raw: &Delivery, _ch: &Channel) -> Result<(), BrokerError> {
        tracing::info!(document_id = %evt.data.unwrap().document_id, "consumed document.commit.recorded");
        Ok(())
    }
}
