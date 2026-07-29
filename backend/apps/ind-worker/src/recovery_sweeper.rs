//! Universal background job recovery sweeper.
//!
//! Claims `waiting` recovery rows that are due, decides whether to:
//!
//! - **Terminalize + write DLQ** when `recovery_attempts` already reached
//!   the configured cap (one-shot DLQ insert is gated inside the repo).
//!   `patient` rows are exempt — a dependency outage waits indefinitely.
//! - **Enqueue a replay** through `job_outbox` and flip the row back to
//!   `waiting` with `recovery_attempts += 1` and a pushed-out
//!   `next_retry_at` — both writes in one repository-managed transaction
//!   (`replay_via_outbox`) so a crash between them can neither strand the
//!   ledger nor double-replay.
//!
//! The sweeper never propagates errors to its caller — failures are logged
//! and the row remains leased (re-claimable when the lease expires) so a
//! transient repo or outbox blip does not lose work.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use ind_application::recovery_keys::last_backoff_for;
use ind_application::repos::background_job_recovery::{
    BackgroundJobRecoveryRepository, DeadLetterInsert, RecoveryReplay,
};
use ind_domain::{BackgroundJobFailureClass, BackgroundJobRecovery, patient_backoff};

/// Fallback when `last_backoff_for(job_type)` returns a duration that overflows
/// `chrono::Duration::from_std`. The default matches the `unknown` arm in
/// `last_backoff_for`, so jobs without an explicit policy still defer by 15
/// minutes instead of immediately re-claiming.
const FALLBACK_NEXT_RETRY_SECONDS: i64 = 900;

/// A replayed job may still be executing when its schedule-based `next_retry_at`
/// comes due (AI requests time out at 300s), and re-claiming an in-flight replay
/// dispatches a duplicate. The post-replay deferral therefore never drops below
/// this window; failure-driven re-parks still own the real pacing.
const REPLAY_IN_FLIGHT_FLOOR_SECONDS: i64 = 900;

fn next_retry_at_after(now: DateTime<Utc>, job_type: &str) -> DateTime<Utc> {
    let backoff = last_backoff_for(job_type);
    now + chrono::Duration::from_std(backoff)
        .unwrap_or_else(|_| chrono::Duration::seconds(FALLBACK_NEXT_RETRY_SECONDS))
}

pub async fn sweep_background_recoveries(
    background_recovery_repo: &Arc<dyn BackgroundJobRecoveryRepository>,
    worker_id: &str,
    job_recovery_max_attempts: i32,
    job_recovery_batch_size: i64,
    lease_seconds: i64,
    now: DateTime<Utc>,
) {
    let lease_until = now + chrono::Duration::seconds(lease_seconds);

    let claimed = match background_recovery_repo
        .claim_due(now, worker_id, lease_until, job_recovery_batch_size)
        .await
    {
        Ok(rows) => rows,
        Err(error) => {
            tracing::error!(%error, "background recovery sweeper failed to claim due rows");
            return;
        }
    };

    if claimed.is_empty() {
        return;
    }

    tracing::info!(
        count = claimed.len(),
        worker = worker_id,
        "background recovery sweeper claimed due rows",
    );

    for row in claimed {
        process_claimed_row(
            background_recovery_repo,
            row,
            worker_id,
            job_recovery_max_attempts,
            now,
        )
        .await;
    }
}

async fn process_claimed_row(
    background_recovery_repo: &Arc<dyn BackgroundJobRecoveryRepository>,
    row: BackgroundJobRecovery,
    worker_id: &str,
    job_recovery_max_attempts: i32,
    now: DateTime<Utc>,
) {
    let patient = row.failure_class == BackgroundJobFailureClass::Patient;
    if !patient && row.recovery_attempts >= job_recovery_max_attempts {
        terminalize(background_recovery_repo, &row, worker_id, now).await;
        return;
    }

    enqueue_replay(background_recovery_repo, &row, worker_id, now).await;
}

async fn terminalize(
    background_recovery_repo: &Arc<dyn BackgroundJobRecoveryRepository>,
    row: &BackgroundJobRecovery,
    worker_id: &str,
    now: DateTime<Utc>,
) {
    let dlq_insert = DeadLetterInsert {
        job_type: &row.job_type,
        payload: row.payload.clone(),
        dedupe_key: row.dedupe_key.as_deref(),
        failure_reason_code: Some("recovery_attempts_exhausted"),
        error_message: &row.error_message,
        attempts: row.apalis_attempts,
        failed_at: now,
    };

    if let Err(error) = background_recovery_repo
        .mark_recovery_terminal(
            row.id,
            "recovery_attempts_exhausted",
            &row.error_message,
            worker_id,
            dlq_insert,
            now,
        )
        .await
    {
        tracing::error!(
            %error,
            recovery_id = %row.id,
            job_type = %row.job_type,
            "background recovery sweeper failed to terminalize row",
        );
    } else {
        tracing::warn!(
            recovery_id = %row.id,
            job_type = %row.job_type,
            "background recovery sweeper terminalized row at cap; DLQ written",
        );
    }
}

async fn enqueue_replay(
    background_recovery_repo: &Arc<dyn BackgroundJobRecoveryRepository>,
    row: &BackgroundJobRecovery,
    worker_id: &str,
    now: DateTime<Utc>,
) {
    // This deferral only guards re-claiming a replay that never reported back;
    // a re-failing job re-parks itself with the authoritative schedule.
    let scheduled = if row.failure_class == BackgroundJobFailureClass::Patient {
        now + chrono::Duration::from_std(patient_backoff(row.recovery_attempts))
            .unwrap_or_else(|_| chrono::Duration::seconds(FALLBACK_NEXT_RETRY_SECONDS))
    } else {
        next_retry_at_after(now, &row.job_type)
    };
    let next_retry_at =
        scheduled.max(now + chrono::Duration::seconds(REPLAY_IN_FLIGHT_FLOOR_SECONDS));

    match background_recovery_repo
        .replay_via_outbox(RecoveryReplay {
            id: row.id,
            job_type: &row.job_type,
            payload: row.payload.clone(),
            dedupe_key: row.dedupe_key.as_deref(),
            lease_owner: worker_id,
            next_retry_at,
            now,
        })
        .await
    {
        Ok(outbox_id) => {
            tracing::info!(
                recovery_id = %row.id,
                job_type = %row.job_type,
                outbox_id = %outbox_id,
                next_retry_at = %next_retry_at,
                "background recovery sweeper enqueued replay",
            );
        }
        Err(error) => {
            tracing::error!(
                %error,
                recovery_id = %row.id,
                job_type = %row.job_type,
                "background recovery sweeper failed to enqueue replay",
            );
        }
    }
}
