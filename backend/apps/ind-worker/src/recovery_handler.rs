//! Universal background job recovery — worker-side helpers.
//!
//! These helpers translate the four worker outcomes into ledger writes:
//!
//! | Outcome | Helper | Effect |
//! |---|---|---|
//! | Success | [`record_success`] | Best-effort `mark_resolved` for the recovery key. |
//! | Retryable, attempt < max | _none_ | Apalis owns the retry; we do not touch the ledger. |
//! | Retryable, attempt >= max | [`record_retryable_exhausted`] | Upserts a `waiting` row. |
//! | Terminal | [`record_terminal_failure`] | Upserts a `terminal` row + writes one DLQ entry in the same transaction. |
//!
//! Success resolution is best-effort, but failure recording is part of the
//! durable handoff. Once Apalis retries are exhausted, the worker must not ack
//! the job unless the recovery ledger/DLQ write succeeds.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use ind_application::AppError;
use ind_application::repos::background_job_recovery::{
    BackgroundJobRecoveryRepository, DeadLetterInsert, RecoveryFailureInput,
};
use ind_domain::{BackgroundJobFailureClass, BackgroundJobSubjectKind, JobOutboxId};

/// Inputs shared by both failure-recording helpers. Borrowed strings keep
/// the call site free of pointless `.to_string()` allocations.
pub struct RecordedFailure<'a> {
    pub recovery_key: &'a str,
    pub job_type: &'a str,
    pub payload: serde_json::Value,
    pub dedupe_key: Option<&'a str>,
    pub outbox_id: Option<JobOutboxId>,
    pub subject_kind: Option<BackgroundJobSubjectKind>,
    pub subject_id: Option<&'a str>,
    pub failure_reason_code: &'a str,
    pub error_message: &'a str,
    pub attempt: i32,
    pub now: DateTime<Utc>,
}

pub async fn record_success(
    repo: &Arc<dyn BackgroundJobRecoveryRepository>,
    recovery_key: &str,
    now: DateTime<Utc>,
) {
    if let Err(error) = repo.mark_resolved(recovery_key, now).await {
        tracing::warn!(
            %error,
            recovery_key,
            "failed to mark background job recovery row as resolved",
        );
    }
}

pub async fn record_retryable_exhausted(
    repo: &Arc<dyn BackgroundJobRecoveryRepository>,
    failure: RecordedFailure<'_>,
    next_retry_at: DateTime<Utc>,
) -> Result<(), AppError> {
    let input = RecoveryFailureInput {
        recovery_key: failure.recovery_key,
        job_type: failure.job_type,
        payload: failure.payload,
        dedupe_key: failure.dedupe_key,
        outbox_id: failure.outbox_id,
        subject_kind: failure.subject_kind,
        subject_id: failure.subject_id,
        failure_class: BackgroundJobFailureClass::Retryable,
        failure_reason_code: failure.failure_reason_code,
        error_message: failure.error_message,
        apalis_attempts: failure.attempt,
        next_retry_at: Some(next_retry_at),
        now: failure.now,
    };

    repo.upsert_waiting_failure(input).await?;
    Ok(())
}

/// Parks a dependency-outage failure as a `patient` waiting row. Unlike
/// [`record_retryable_exhausted`] this runs on the FIRST failure — in-process
/// retries against an offline dependency only burn attempts.
///
/// Pacing lives here, not at the call site: the upsert preserves the row's
/// accumulated `recovery_attempts` across re-failures, and that counter —
/// only known after the upsert returns — indexes the sparse backoff. The
/// follow-up reschedule is a separate targeted write; a crash between the
/// two leaves the row at the one-minute floor, which merely replays it
/// early once.
pub async fn record_patient_failure(
    repo: &Arc<dyn BackgroundJobRecoveryRepository>,
    failure: RecordedFailure<'_>,
) -> Result<(), AppError> {
    let now = failure.now;
    let provisional_next = now + patient_backoff_chrono(0);
    let input = RecoveryFailureInput {
        recovery_key: failure.recovery_key,
        job_type: failure.job_type,
        payload: failure.payload,
        dedupe_key: failure.dedupe_key,
        outbox_id: failure.outbox_id,
        subject_kind: failure.subject_kind,
        subject_id: failure.subject_id,
        failure_class: BackgroundJobFailureClass::Patient,
        failure_reason_code: failure.failure_reason_code,
        error_message: failure.error_message,
        apalis_attempts: failure.attempt,
        next_retry_at: Some(provisional_next),
        now,
    };

    let row = repo.upsert_waiting_failure(input).await?;
    if row.recovery_attempts > 0 {
        repo.reschedule_waiting(
            row.id,
            now + patient_backoff_chrono(row.recovery_attempts),
            now,
        )
        .await?;
    }
    Ok(())
}

fn patient_backoff_chrono(recovery_attempts: i32) -> chrono::Duration {
    chrono::Duration::from_std(ind_domain::patient_backoff(recovery_attempts))
        .unwrap_or_else(|_| chrono::Duration::seconds(60))
}

pub async fn record_terminal_failure(
    repo: &Arc<dyn BackgroundJobRecoveryRepository>,
    failure: RecordedFailure<'_>,
) -> Result<(), AppError> {
    let dlq_insert = DeadLetterInsert {
        job_type: failure.job_type,
        payload: failure.payload.clone(),
        dedupe_key: failure.dedupe_key,
        failure_reason_code: Some(failure.failure_reason_code),
        error_message: failure.error_message,
        attempts: failure.attempt,
        failed_at: failure.now,
    };

    let input = RecoveryFailureInput {
        recovery_key: failure.recovery_key,
        job_type: failure.job_type,
        payload: failure.payload,
        dedupe_key: failure.dedupe_key,
        outbox_id: failure.outbox_id,
        subject_kind: failure.subject_kind,
        subject_id: failure.subject_id,
        failure_class: BackgroundJobFailureClass::Terminal,
        failure_reason_code: failure.failure_reason_code,
        error_message: failure.error_message,
        apalis_attempts: failure.attempt,
        next_retry_at: None,
        now: failure.now,
    };

    repo.upsert_terminal_failure(input, dlq_insert).await?;
    Ok(())
}
