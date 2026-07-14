use ed_persistence_postgres::OutboxStore;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};
use crate::event_bus::IEventBus;
use crate::error::BrokerError;

pub struct OutboxRelayService {
    pub store: Arc<dyn OutboxStore>,
    pub bus: Arc<dyn IEventBus>,
    pub poll_interval: Duration,
    pub batch_size: i64,
    pub max_attempts: i32,
    pub backoff_base_ms: i64,
    pub backoff_max_ms: i64,
    /// Per-replica identifier so the `leased_to` column on the
    /// outbox row records which instance is processing it.
    pub relay_id: String,
}

impl OutboxRelayService {
    pub async fn run(self: Arc<Self>) {
        loop {
            if let Err(e) = self.tick().await {
                error!(error = %e, "outbox relay tick failed");
            }
            tokio::time::sleep(self.poll_interval).await;
        }
    }

    pub async fn tick(&self) -> Result<(), BrokerError> {
        // Reserve up to `batch_size` rows, advancing them to `InFlight`
        // with `leased_to = relay_id` and a fresh lease.
        let claimed = self
            .store
            .claim_pending(self.batch_size, self.backoff_base_ms, &self.relay_id)
            .await?;
        if claimed.is_empty() {
            return Ok(());
        }
        info!(count = claimed.len(), "claimed outbox rows");
        for row in claimed {
            // The payload is already a serialized envelope; forward
            // it through the IEventBus (which uses a non-generic
            // `publish_value` to stay dyn-compatible).
            match self.bus.publish_value(&row.topic, row.payload.clone()).await {
                Ok(()) => {
                    self.store.mark_sent(row.id).await?;
                }
                Err(e) => {
                    let next_attempt = row.attempt_count + 1;
                    if next_attempt >= self.max_attempts {
                        self.store
                            .mark_dead_lettered(row.id, &e.to_string())
                            .await?;
                        warn!(id = %row.id, "outbox row dead-lettered after max attempts");
                    } else {
                        let backoff = (self.backoff_base_ms
                            * (1 << next_attempt.min(8)))
                        .min(self.backoff_max_ms);
                        self.store
                            .mark_failed(row.id, &e.to_string(), backoff / 1000)
                            .await?;
                    }
                }
            }
        }
        Ok(())
    }
}
