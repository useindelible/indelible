use secrecy::ExposeSecret;

use crate::config::ServerConfig;
use crate::{router, services};

pub async fn run() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = ServerConfig::load()?;

    ind_observability::init_tracing(&config.server.environment, &config.log_level);

    tracing::info!(
        environment = %config.server.environment,
        host = %config.server.host,
        port = %config.server.port,
        "starting ind-api"
    );

    let pool = ind_persistence::create_pool(config.database_url.expose_secret()).await?;
    tracing::info!("database pool created");

    ind_persistence::run_migrations(&pool).await?;
    tracing::info!("database migrations applied");

    let services = services::build(&config, pool).await?;
    let app = router::build(services.state, &config, services.rate_limit_config);

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("listening on {addr}");

    // Connect-info exposes the direct peer address, which the trusted-proxy
    // logic needs to decide whether to honor forwarded headers.
    let serve_result = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await;
    services.realtime_listener_handle.abort();
    serve_result?;

    tracing::info!("server shut down");
    Ok(())
}

#[expect(
    clippy::expect_used,
    reason = "signal-handler installation is an unavoidable boot precondition; failure is fatal at startup"
)]
async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("shutdown signal received, starting graceful shutdown");
}
