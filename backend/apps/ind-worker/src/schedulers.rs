use std::sync::Arc;
use std::time::Duration;

use ind_application::handlers::feed::FeedPollScheduleConfig;
use ind_domain::{FeedPollJob, NotionSyncConnectionJob};

use crate::config::WorkerConfig;
use crate::context::{FeedJobDeps, NotionJobDeps, WebhookJobDeps};
use crate::jobs;

pub async fn run_feed_scheduler_loop(ctx: Arc<FeedJobDeps>, config: WorkerConfig) {
    let mut interval = tokio::time::interval(Duration::from_secs(
        config.feed.scheduler_interval_secs.max(1),
    ));

    loop {
        interval.tick().await;

        let claimed = match ctx
            .feed_repo
            .claim_due_sources(
                chrono::Utc::now(),
                &ctx.worker_id,
                config.feed.batch_size,
                chrono::Duration::seconds(config.feed.lease_secs),
            )
            .await
        {
            Ok(claimed) => claimed,
            Err(err) => {
                tracing::error!(error = %err, "feed scheduler claim failed");
                continue;
            }
        };

        for source in claimed {
            let payload = match serde_json::to_value(FeedPollJob {
                source_id: source.id,
            }) {
                Ok(payload) => payload,
                Err(err) => {
                    tracing::error!(error = %err, source_id = %source.id, "failed to serialize feed poll job");
                    let _ = ctx.feed_repo.clear_source_lease(source.id).await;
                    continue;
                }
            };

            if let Err(err) = ctx
                .outbox_repo
                .enqueue(
                    "feed.poll",
                    payload,
                    Some(format!("feed.poll:{}", source.id)),
                    chrono::Utc::now(),
                )
                .await
            {
                tracing::error!(error = %err, source_id = %source.id, "failed to enqueue feed poll job");
                let _ = ctx.feed_repo.clear_source_lease(source.id).await;
            }
        }
    }
}

pub async fn run_notion_catch_up_loop(
    notion_job_deps: Option<Arc<NotionJobDeps>>,
    config: WorkerConfig,
) {
    let Some(deps) = notion_job_deps else {
        tracing::info!("notion catch-up scheduler disabled: notion dependencies unavailable");
        return;
    };
    let mut interval = tokio::time::interval(Duration::from_secs(
        config.integrations.notion.catch_up_interval_secs.max(1),
    ));

    loop {
        interval.tick().await;

        let connections = match deps.connection_repo.list_active_notion_auto_export().await {
            Ok(connections) => connections,
            Err(err) => {
                tracing::error!(error = %err, "notion catch-up connection scan failed");
                continue;
            }
        };

        for connection in connections {
            let payload = match serde_json::to_value(NotionSyncConnectionJob {
                connection_id: connection.id,
                user_id: connection.user_id,
                requested_by_user: false,
            }) {
                Ok(payload) => payload,
                Err(err) => {
                    tracing::error!(
                        error = %err,
                        connection_id = %connection.id,
                        "failed to serialize notion catch-up job"
                    );
                    continue;
                }
            };

            if let Err(err) = deps
                .outbox_repo
                .enqueue(
                    "integration.notion.sync_connection",
                    payload,
                    Some(format!("notion_catch_up:{}", connection.id.into_uuid())),
                    chrono::Utc::now(),
                )
                .await
            {
                tracing::error!(
                    error = %err,
                    connection_id = %connection.id,
                    "failed to enqueue notion catch-up job"
                );
            }
        }
    }
}

pub async fn run_webhook_projector_loop(webhooks: WebhookJobDeps) {
    let mut interval = tokio::time::interval(Duration::from_secs(5));

    loop {
        interval.tick().await;
        match jobs::webhooks::project_due_webhooks(&webhooks, 100).await {
            Ok(count) if count > 0 => {
                tracing::info!(count, "webhook projector enqueued dispatches");
            }
            Ok(_) => {}
            Err(err) => {
                tracing::error!(error = %err, "webhook projector failed");
            }
        }
    }
}

pub fn feed_poll_schedule(config: &WorkerConfig) -> FeedPollScheduleConfig {
    FeedPollScheduleConfig {
        default_public_poll_interval_minutes: config.feed.default_poll_interval_minutes,
        min_public_poll_interval_minutes: config.feed.min_poll_interval_minutes,
    }
    .normalized()
}
