#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() -> anyhow::Result<()> {
    ind_observability::process::disable_transparent_huge_pages();
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(ind_api::bootstrap::run())
}
