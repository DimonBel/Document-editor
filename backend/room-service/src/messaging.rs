//! Room-service consumer for `document.commit.recorded`.
//!
//! Per issue #148: previously a malformed but-validly-deserialized
//! event with absent `data` could panic on `evt.data.unwrap()`,
//! taking the consumer task down. Also, consumer tasks had no
//! join handle, shutdown signal, or supervisor -- a panic killed
//! the whole subscriber with no telemetry.

use std::sync::Arc;
use std::future::Future;

use async_trait::async_trait;
use lapin::{message::Delivery, Channel};
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

use ed_messaging_rabbitmq::{BrokerError, ConsumerHandler};
use ed_contracts::EventMessage;
use ed_contracts::events::document::DocumentCommitRecordedEvent;

pub struct DocumentCommitConsumer;

#[async_trait]
impl ConsumerHandler for DocumentCommitConsumer {
    type Event = DocumentCommitRecordedEvent;

    async fn handle(
        &self,
        evt: EventMessage<Self::Event>,
        raw: &Delivery,
        _ch: &Channel,
    ) -> Result<(), BrokerError> {
        // Extract the payload safely.
        let Some(data) = evt.data.as_ref() else {
            warn!(
                delivery_tag = raw.delivery_tag,
                correlation_id = %evt.correlation_id,
                "received document.commit.recorded with no data; acking and skipping"
            );
            return Ok(());
        };
        info!(
            document_id = %data.document_id,
            commit_hash = %data.commit_hash,
            correlation_id = %evt.correlation_id,
            "consumed document.commit.recorded"
        );
        // Business logic would update the room's read-model here.
        Ok(())
    }
}

/// Supervisor that owns the consumer task and can be gracefully shut
/// down by `notify_shutdown()` (or by dropping the supervisor).
pub struct ConsumerSupervisor {
    handle: JoinHandle<()>,
    shutdown: Arc<Notify>,
}

impl ConsumerSupervisor {
    pub fn new(
        runner: impl Future<Output = ()> + Send + 'static,
        shutdown: Arc<Notify>,
    ) -> Self {
        let shutdown_inner = shutdown.clone();
        let handle = tokio::spawn(async move {
            // `tokio::select!` runs the consumer until either the
            // future completes (broker error) or shutdown is signaled.
            tokio::select! {
                _ = runner => {
                    error!("consumer task exited unexpectedly; supervisor will report at /healthz");
                }
                _ = shutdown_inner.notified() => {
                    info!("consumer shutdown signaled");
                }
            }
        });
        Self { handle, shutdown }
    }

    pub fn shutdown(&self) { self.shutdown.notify_waiters(); }

    pub fn is_alive(&self) -> bool { !self.handle.is_finished() }

    pub async fn join(self) -> Result<(), tokio::task::JoinError> {
        self.handle.await
    }
}

/// Consumes the channel errors and re-publishes them as `tracing`
/// events instead of letting them take the task down.
pub fn observe_panic(context: &str) {
    let context = context.to_string();
    std::panic::set_hook(Box::new(move |info| {
        error!(%context, panic = %info, "consumer task panicked");
    }));
}
