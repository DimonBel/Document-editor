use ed_messaging_rabbitmq::OutboxRelayService;
use std::sync::Arc;
use ed_persistence_postgres::OutboxStore;
use ed_messaging_rabbitmq::IEventBus;
pub fn start_relay(bus: Arc<dyn IEventBus>, store: Arc<dyn OutboxStore>) {
    let relay = Arc::new(OutboxRelayService {
        store, bus, poll_interval: std::time::Duration::from_millis(500),
        batch_size: 50, max_attempts: 5, backoff_base_ms: 500, backoff_max_ms: 60_000,
    });
    tokio::spawn(async move { relay.run().await; });
}
