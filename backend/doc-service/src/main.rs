use ed_observability::init_tracing;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing("doc-service", true);
    doc_service::app::run().await
}
