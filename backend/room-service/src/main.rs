use ed_observability::init_tracing;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing("room-service", true);
    tracing::info!("room-service starting");
    backend_room_service::app::run().await
}
