mod auto_heal;
mod bootstrap;
mod concurrency;
mod config;
pub mod context;
mod failure;
mod jobs;
mod providers;
mod recovery_handler;
mod recovery_sweeper;
mod relay;
mod renderer_client;
mod repositories;
mod schedulers;
mod shutdown;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    bootstrap::run().await
}
