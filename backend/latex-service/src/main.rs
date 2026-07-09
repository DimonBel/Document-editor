use ed_observability::init_tracing;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing("latex-service", true);
    backend_latex_service::app::run().await
}
