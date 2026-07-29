use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;

/// Initializes the tracing subscriber.
///
/// Filter priority: `RUST_LOG` env var → `default_filter` → `"info"`.
/// Production environments get JSON output; everything else gets pretty-printed logs.
pub fn init_tracing(environment: &str, default_filter: &str) {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter));

    let registry = tracing_subscriber::registry().with(filter);

    if environment == "production" {
        registry.with(fmt::layer().json()).init();
    } else {
        registry.with(fmt::layer().pretty()).init();
    }
}
