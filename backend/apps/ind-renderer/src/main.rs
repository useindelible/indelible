#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

mod browser;
mod browser_identity;
mod config;
mod render;
mod routes;
mod storage;
mod types;

use std::sync::Arc;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};

use crate::browser::BrowserManager;
use crate::config::RendererConfig;
use crate::routes::AppState;
use crate::storage::S3Storage;

fn build_app_state(
    browser: Arc<BrowserManager>,
    capture: config::CaptureSettings,
    storage: Arc<S3Storage>,
    egress_policy: ind_egress::EgressPolicy,
) -> Arc<AppState> {
    Arc::new(AppState {
        browser,
        capture,
        storage,
        egress_policy,
    })
}

fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(routes::health))
        .route("/render/url", post(routes::render_url))
        .route("/render/monolith", post(routes::render_monolith))
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
        .with_state(state)
}

#[tokio::main(worker_threads = 2)]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let config = RendererConfig::load()?;
    ind_observability::init_tracing(&config.server.environment, &config.server.log_level);

    tracing::info!(
        chromium = %config.chromium.path.display(),
        host = %config.server.host,
        port = config.server.port,
        "starting ind-renderer"
    );

    let s3 = Arc::new(S3Storage::from_config(&config).await?);

    let browser_manager = Arc::new(BrowserManager::new(
        config.chromium.path.clone(),
        config.chromium.single_process,
        config.chromium.virtual_time_budget,
        config.chromium.idle_timeout_secs,
        config.capture.max_concurrency,
    ));

    browser_manager.spawn_idle_watchdog();

    let state = build_app_state(
        browser_manager.clone(),
        config.capture.clone(),
        s3,
        config.egress_policy(),
    );

    let app = build_router(state);

    let bind = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!(bind = %bind, "listening");

    let serve_result = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await;

    browser_manager.shutdown().await;
    serve_result?;

    tracing::info!("ind-renderer shut down");
    Ok(())
}

async fn shutdown_signal() {
    #[expect(
        clippy::expect_used,
        reason = "installing the Ctrl+C handler is an unavoidable boot precondition for graceful shutdown"
    )]
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    #[expect(
        clippy::expect_used,
        reason = "installing the SIGTERM handler is an unavoidable boot precondition for graceful shutdown"
    )]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}

#[cfg(test)]
mod journey_tests;
