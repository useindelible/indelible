use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use ind_application::AppError;
use ind_application::repos::embedding_backfill::EmbeddingBackfillRepository;
use ind_application::repos::integrity::{IntegrityStats, IntegrityStatsRepository};
use ind_application::repos::maintenance::MaintenanceTaskLease;
use ind_application::services::tts::synthesis::{TtsOrphanSweepReport, TtsOrphanSweeper};

use crate::context::RecoveryJobDeps;

const EMBEDDING_REPAIR_TASK: &str = "embedding.repair";
const INTEGRITY_TASK: &str = "integrity.check";
const TTS_ORPHAN_TASK: &str = "tts.orphan_cleanup";
const MAINTENANCE_FAILURE_RETRY_SECS: u64 = 60;
const TTS_PAGE_CONTINUATION_DELAY_SECS: u64 = 1;

pub async fn run_auto_heal_loop(ctx: Arc<RecoveryJobDeps>) {
    run_auto_heal_once(&ctx).await;

    let mut interval = tokio::time::interval(Duration::from_secs(ctx.auto_heal_interval_secs));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    interval.tick().await;

    loop {
        interval.tick().await;
        run_auto_heal_once(&ctx).await;
    }
}

pub async fn run_auto_heal_once(ctx: &RecoveryJobDeps) {
    let now = Utc::now();

    crate::recovery_sweeper::sweep_background_recoveries(
        &ctx.background_recovery_repo,
        &ctx.worker_id,
        ctx.job_recovery_max_attempts,
        ctx.job_recovery_batch_size,
        ctx.auto_heal_lease_secs,
        now,
    )
    .await;

    run_embedding_repair_if_due(ctx).await;
    run_integrity_check_if_due(ctx).await;
    run_tts_orphan_cleanup_if_due(ctx).await;
}

pub async fn repair_missing_vector_embeddings(
    repo: &dyn EmbeddingBackfillRepository,
    limit: i64,
) -> Result<i64, AppError> {
    repo.enqueue_missing_vector_repairs(limit).await
}

pub async fn sweep_integrity_stats(
    repo: &dyn IntegrityStatsRepository,
) -> Result<IntegrityStats, AppError> {
    repo.stats().await
}

pub async fn sweep_tts_orphan_objects(
    sweeper: &TtsOrphanSweeper,
    continuation_cursor: Option<&str>,
    max_objects: i32,
) -> Result<TtsOrphanSweepReport, AppError> {
    sweeper.sweep_page(continuation_cursor, max_objects).await
}

async fn run_embedding_repair_if_due(ctx: &RecoveryJobDeps) {
    let Some(_) = acquire_maintenance(ctx, EMBEDDING_REPAIR_TASK, Utc::now()).await else {
        return;
    };
    match repair_missing_vector_embeddings(
        ctx.embedding_backfill_repo.as_ref(),
        ctx.auto_heal_batch_size,
    )
    .await
    {
        Ok(repaired) => {
            tracing::info!(repaired, "embedding missing-vector repair finished");
            let completed_at = Utc::now();
            complete_maintenance(
                ctx,
                EMBEDDING_REPAIR_TASK,
                schedule_after(completed_at, ctx.embedding_repair_interval_secs),
                None,
                completed_at,
            )
            .await;
        }
        Err(error) => {
            tracing::warn!(%error, "embedding missing-vector repair failed");
            fail_maintenance(ctx, EMBEDDING_REPAIR_TASK, &error).await;
        }
    }
}

async fn run_integrity_check_if_due(ctx: &RecoveryJobDeps) {
    let Some(_) = acquire_maintenance(ctx, INTEGRITY_TASK, Utc::now()).await else {
        return;
    };
    match sweep_integrity_stats(ctx.integrity_stats_repo.as_ref()).await {
        Ok(stats) => {
            log_integrity_stats(&stats);
            let completed_at = Utc::now();
            complete_maintenance(
                ctx,
                INTEGRITY_TASK,
                schedule_after(completed_at, ctx.integrity_interval_secs),
                None,
                completed_at,
            )
            .await;
        }
        Err(error) => {
            tracing::warn!(%error, "integrity stats sweep failed");
            fail_maintenance(ctx, INTEGRITY_TASK, &error).await;
        }
    }
}

async fn run_tts_orphan_cleanup_if_due(ctx: &RecoveryJobDeps) {
    let Some(sweeper) = ctx.tts_orphan_sweeper.as_ref() else {
        return;
    };
    let Some(lease) = acquire_maintenance(ctx, TTS_ORPHAN_TASK, Utc::now()).await else {
        return;
    };
    match sweep_tts_orphan_objects(
        sweeper,
        lease.continuation_cursor.as_deref(),
        ctx.tts_orphan_page_size,
    )
    .await
    {
        Ok(report) => {
            log_tts_orphan_sweep(&report);
            let completed_at = Utc::now();
            let has_more = report.next_continuation_cursor.is_some();
            let next_run_at = schedule_after(
                completed_at,
                if has_more {
                    TTS_PAGE_CONTINUATION_DELAY_SECS
                } else {
                    ctx.tts_orphan_interval_secs
                },
            );
            complete_maintenance(
                ctx,
                TTS_ORPHAN_TASK,
                next_run_at,
                report.next_continuation_cursor.as_deref(),
                completed_at,
            )
            .await;
        }
        Err(error) => {
            tracing::warn!(%error, "TTS orphan object sweep failed");
            fail_maintenance(ctx, TTS_ORPHAN_TASK, &error).await;
        }
    }
}

async fn acquire_maintenance(
    ctx: &RecoveryJobDeps,
    task_name: &str,
    now: chrono::DateTime<Utc>,
) -> Option<MaintenanceTaskLease> {
    match ctx
        .maintenance_task_repo
        .try_acquire(
            task_name,
            &ctx.worker_id,
            now,
            now + chrono::Duration::seconds(ctx.maintenance_lease_secs.max(1)),
        )
        .await
    {
        Ok(lease) => lease,
        Err(error) => {
            tracing::warn!(%error, task_name, "maintenance task lease acquisition failed");
            None
        }
    }
}

async fn complete_maintenance(
    ctx: &RecoveryJobDeps,
    task_name: &str,
    next_run_at: chrono::DateTime<Utc>,
    continuation_cursor: Option<&str>,
    now: chrono::DateTime<Utc>,
) {
    if let Err(error) = ctx
        .maintenance_task_repo
        .complete(
            task_name,
            &ctx.worker_id,
            next_run_at,
            continuation_cursor,
            now,
        )
        .await
    {
        tracing::warn!(%error, task_name, "maintenance task completion failed");
    }
}

async fn fail_maintenance(ctx: &RecoveryJobDeps, task_name: &str, task_error: &AppError) {
    let failed_at = Utc::now();
    if let Err(error) = ctx
        .maintenance_task_repo
        .fail(
            task_name,
            &ctx.worker_id,
            schedule_after(failed_at, MAINTENANCE_FAILURE_RETRY_SECS),
            &task_error.to_string(),
            failed_at,
        )
        .await
    {
        tracing::warn!(%error, task_name, "maintenance task failure release failed");
    }
}

fn schedule_after(now: chrono::DateTime<Utc>, seconds: u64) -> chrono::DateTime<Utc> {
    now + chrono::Duration::seconds(i64::try_from(seconds).unwrap_or(i64::MAX))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrityStatsLogSeverity {
    Info,
    Warn,
}

pub fn integrity_stats_log_severity(stats: &IntegrityStats) -> IntegrityStatsLogSeverity {
    if stats.documents_missing_search_rows > 0
        || stats.documents_missing_vectors > 0
        || stats.failed_derived_assets > 0
        || stats.dead_letter_jobs > 0
    {
        IntegrityStatsLogSeverity::Warn
    } else {
        IntegrityStatsLogSeverity::Info
    }
}

fn log_integrity_stats(stats: &IntegrityStats) {
    match integrity_stats_log_severity(stats) {
        IntegrityStatsLogSeverity::Info => tracing::info!(
            documents_missing_search_rows = stats.documents_missing_search_rows,
            documents_missing_vectors = stats.documents_missing_vectors,
            failed_derived_assets = stats.failed_derived_assets,
            dead_letter_jobs = stats.dead_letter_jobs,
            "integrity stats sweep finished"
        ),
        IntegrityStatsLogSeverity::Warn => tracing::warn!(
            documents_missing_search_rows = stats.documents_missing_search_rows,
            documents_missing_vectors = stats.documents_missing_vectors,
            failed_derived_assets = stats.failed_derived_assets,
            dead_letter_jobs = stats.dead_letter_jobs,
            "integrity stats sweep found issues"
        ),
    }
}

fn log_tts_orphan_sweep(report: &TtsOrphanSweepReport) {
    if report.failed_deletes > 0 {
        tracing::warn!(
            scanned_objects = report.scanned_objects,
            referenced_objects = report.referenced_objects,
            deleted_objects = report.deleted_objects,
            failed_deletes = report.failed_deletes,
            "TTS orphan object sweep finished with delete failures"
        );
    } else {
        tracing::info!(
            scanned_objects = report.scanned_objects,
            referenced_objects = report.referenced_objects,
            deleted_objects = report.deleted_objects,
            failed_deletes = report.failed_deletes,
            "TTS orphan object sweep finished"
        );
    }
}
