use std::sync::Arc;
use std::time::Duration;

use ind_application::repos::FeedDeliveryRetentionWindows;
use ind_application::repos::RetentionCleanupRepository;
use tokio::time::MissedTickBehavior;

use crate::config::FeedRetentionCleanupSettings;
pub async fn run_retention_cleanup_loop(
    retention_repo: Arc<dyn RetentionCleanupRepository>,
    settings: FeedRetentionCleanupSettings,
) {
    run_retention_cleanup_once(retention_repo.as_ref(), &settings).await;

    let mut interval = tokio::time::interval(Duration::from_secs(settings.interval_secs));
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
    interval.tick().await;

    loop {
        interval.tick().await;
        run_retention_cleanup_once(retention_repo.as_ref(), &settings).await;
    }
}

pub async fn run_retention_cleanup_once(
    retention_repo: &dyn RetentionCleanupRepository,
    settings: &FeedRetentionCleanupSettings,
) {
    let windows = FeedDeliveryRetentionWindows {
        unseen_days: settings.unseen_days,
        seen_days: settings.seen_days,
        dismissed_days: settings.dismissed_days,
    };

    match retention_repo.prune_feed_deliveries(windows).await {
        Ok(counts) => tracing::info!(
            unseen = counts.unseen,
            seen = counts.seen,
            dismissed = counts.dismissed,
            total = counts.total(),
            "feed retention cleanup: pruned old feed deliveries"
        ),
        Err(error) => {
            tracing::error!(%error, "feed retention cleanup: delivery pruning failed");
            return;
        }
    }

    if settings.compact_orphaned_source_entries {
        let source_entry_retention_days = settings
            .unseen_days
            .max(settings.seen_days)
            .max(settings.dismissed_days);
        match retention_repo
            .compact_orphaned_feed_source_entries(source_entry_retention_days)
            .await
        {
            Ok(count) => tracing::info!(
                count,
                retention_days = source_entry_retention_days,
                "feed retention cleanup: compacted orphaned source entries"
            ),
            Err(error) => {
                tracing::error!(%error, "feed retention cleanup: source-entry compaction failed");
                return;
            }
        }
    }

    match retention_repo
        .delete_disposable_documents(windows, settings.document_grace_days)
        .await
    {
        Ok(count) => tracing::info!(
            count,
            grace_days = settings.document_grace_days,
            "feed retention cleanup: deleted disposable documents"
        ),
        Err(error) => tracing::error!(%error, "feed retention cleanup: document GC failed"),
    }
}
