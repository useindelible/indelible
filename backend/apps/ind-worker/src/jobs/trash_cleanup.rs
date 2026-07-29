use std::sync::Arc;
use std::time::Duration;

use tokio::time::MissedTickBehavior;

use ind_application::repos::library::LibraryRepository;

use crate::config::TrashCleanupSettings;

pub async fn run_trash_cleanup_loop(
    library_repo: Arc<dyn LibraryRepository>,
    settings: TrashCleanupSettings,
) {
    run_trash_cleanup_once(library_repo.as_ref(), settings.retention_days).await;

    let mut interval = tokio::time::interval(Duration::from_secs(settings.interval_secs));
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
    interval.tick().await;

    loop {
        interval.tick().await;
        run_trash_cleanup_once(library_repo.as_ref(), settings.retention_days).await;
    }
}

async fn run_trash_cleanup_once(library_repo: &dyn LibraryRepository, retention_days: i64) {
    match library_repo.purge_expired_trash(retention_days).await {
        Ok(count) => tracing::info!(
            count,
            retention_days,
            "trash cleanup: purged expired library entries"
        ),
        Err(error) => tracing::error!(%error, "trash cleanup: purge failed"),
    }
}
