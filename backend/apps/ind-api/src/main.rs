#[tokio::main]
async fn main() -> anyhow::Result<()> {
    ind_api::bootstrap::run().await
}
