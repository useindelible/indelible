#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

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

fn main() -> anyhow::Result<()> {
    ind_observability::process::disable_transparent_huge_pages();
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(bootstrap::run())
}
