use ed_persistence_postgres::OutboxStore;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};
use crate::event_bus::IEventBus;
use crate::error::BrokerError;
use ed_contracts::EventMessage;
use serde_json::Value;
pub struct OutboxRelayService {
    pub store: Arc<dyn OutboxStore>,
    pub bus: Arc<dyn IEventBus>,
    pub poll_interval: Duration, pub batch_size: i64,
    pub max_attempts: i32, pub backoff_base_ms: i64, pub backoff_max_ms: i64,
}
impl OutboxRelayService {
    pub async fn run(self: Arc<Self>) {
        loop {
            if let Err(e) = self.tick().await { error!(error = %e, "outbox relay tick failed"); }
            tokio::time::sleep(self.poll_interval).await;
        }
    }
    pub async fn tick(&self) -> Result<(), BrokerError> {
        let claimed = self.store.claim_pending(self.batch_size).await?;
        if claimed.is_empty() { return Ok(()); }
        info!(count = claimed.len(), "claimed outbox rows");
        for row in claimed {
            let envelope: Result<EventMessage<Value>, _> = serde_json::from_value(row.payload.clone());
            let evt = match envelope { Ok(e) => e, Err(e) => { self.store.mark_dead_lettered(row.id, &format!("decode: {e}")).await?; continue; } };
            match self.bus.publish(&row.topic, &evt).await {
                Ok(()) => { self.store.mark_sent(row.id).await?; }
                Err(e) => {
                    let next_attempt = row.attempt_count + 1;
                    if next_attempt >= self.max_attempts {
                        self.store.mark_dead_lettered(row.id, &e.to_string()).await?;
                        warn!(id = %row.id, "outbox row dead-lettered after max attempts");
                    } else {
                        let backoff = (self.backoff_base_ms * (1 << next_attempt.min(8))).min(self.backoff_max_ms);
                        self.store.mark_failed(row.id, &e.to_string(), backoff / 1000).await?;
                    }
                }
            }
        }
        Ok(())
    }
}
