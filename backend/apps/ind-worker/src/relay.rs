use std::sync::Arc;
use std::time::Duration;

use ind_persistence::repos::PgOutboxHandoff;

use crate::config::WorkerConfig;

pub async fn run_relay(handoff: Arc<PgOutboxHandoff>, config: &WorkerConfig) {
    let mut interval = tokio::time::interval(Duration::from_millis(config.relay.poll_interval_ms));

    loop {
        interval.tick().await;

        match handoff.handoff_batch(config.relay.batch_size).await {
            Ok(stats) => {
                if stats.relayed > 0 || stats.deduped > 0 {
                    tracing::info!(
                        claimed = stats.claimed,
                        relayed = stats.relayed,
                        deduped = stats.deduped,
                        "outbox relay batch committed"
                    );
                }
            }
            Err(err) => {
                tracing::error!(error = %err, "outbox relay batch failed");
            }
        }
    }
}
