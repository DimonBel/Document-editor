use ed_messaging_rabbitmq::ConsumerHandler;
use ed_contracts::EventMessage;
use ed_contracts::events::latex::LatexCompileSucceededEvent;
use async_trait::async_trait;
use lapin::{message::Delivery, Channel};
use ed_messaging_rabbitmq::BrokerError;
pub struct LatexSucceededConsumer;
#[async_trait]
impl ConsumerHandler for LatexSucceededConsumer {
    type Event = LatexCompileSucceededEvent;
    async fn handle(&self, evt: EventMessage<Self::Event>, _raw: &Delivery, _ch: &Channel) -> Result<(), BrokerError> {
        if let Some(d) = evt.data {
            tracing::info!(document_id = ?d.document_id, "latex compile succeeded for document");
        }
        Ok(())
    }
}
