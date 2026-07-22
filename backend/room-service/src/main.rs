use ed_observability::init_tracing;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing("room-service", true);
    tracing::info!("room-service starting");
    room_service::app::run().await
}
