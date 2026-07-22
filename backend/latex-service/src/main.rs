use ed_observability::init_tracing;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing("latex-service", true);
    latex_service::app::run().await
}
