use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ind_domain::{
    BackgroundJobFailureClass, BackgroundJobRecovery, BackgroundJobRecoveryId,
    BackgroundJobRecoveryStatus, BackgroundJobSubjectKind, JobOutboxId,
};

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct RecoveryFailureInput<'a> {
    pub recovery_key: &'a str,
    pub job_type: &'a str,
    pub payload: serde_json::Value,
    pub dedupe_key: Option<&'a str>,
    pub outbox_id: Option<JobOutboxId>,
    pub subject_kind: Option<BackgroundJobSubjectKind>,
    pub subject_id: Option<&'a str>,
    pub failure_class: BackgroundJobFailureClass,
    pub failure_reason_code: &'a str,
    pub error_message: &'a str,
    pub apalis_attempts: i32,
    /// Only set on the waiting path; ignored on terminal upserts.
    pub next_retry_at: Option<DateTime<Utc>>,
    pub now: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct DeadLetterInsert<'a> {
    pub job_type: &'a str,
    pub payload: serde_json::Value,
    pub dedupe_key: Option<&'a str>,
    pub failure_reason_code: Option<&'a str>,
    pub error_message: &'a str,
    pub attempts: i32,
    pub failed_at: DateTime<Utc>,
}

/// Inputs for [`BackgroundJobRecoveryRepository::replay_via_outbox`].
#[derive(Debug, Clone)]
pub struct RecoveryReplay<'a> {
    pub id: BackgroundJobRecoveryId,
    pub job_type: &'a str,
    pub payload: serde_json::Value,
    pub dedupe_key: Option<&'a str>,
    pub lease_owner: &'a str,
    pub next_retry_at: DateTime<Utc>,
    pub now: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
pub struct ActiveRecoveryFilter {
    pub status: Option<BackgroundJobRecoveryStatus>,
    pub job_type: Option<String>,
    pub subject_kind: Option<BackgroundJobSubjectKind>,
}

#[async_trait]
pub trait BackgroundJobRecoveryRepository: Send + Sync {
    /// Records (or updates) a recovery row in `waiting` status. Used when an
    /// Apalis-retryable job exhausts its in-Apalis attempts. Preserves
    /// `first_failed_at` and `recovery_attempts` if the row already exists.
    async fn upsert_waiting_failure(
        &self,
        input: RecoveryFailureInput<'_>,
    ) -> Result<BackgroundJobRecovery, AppError>;

    /// Records (or updates) a recovery row in `terminal` status and writes a
    /// `dead_letter_jobs` row in the same transaction — but only if the
    /// recovery row was not already `terminal`. Repeated terminal failures for
    /// the same `recovery_key` will not spam DLQ.
    async fn upsert_terminal_failure(
        &self,
        input: RecoveryFailureInput<'_>,
        dlq_insert: DeadLetterInsert<'_>,
    ) -> Result<BackgroundJobRecovery, AppError>;

    /// Marks the row for `recovery_key` as `resolved` (no-op if no row).
    async fn mark_resolved(
        &self,
        recovery_key: &str,
        resolved_at: DateTime<Utc>,
    ) -> Result<Option<BackgroundJobRecovery>, AppError>;

    /// Claims `waiting` rows whose `next_retry_at <= now` and whose lease
    /// is absent or expired. Sets `status = 'leased'`, `lease_owner`, and
    /// `lease_expires_at`. Returns the claimed rows.
    async fn claim_due(
        &self,
        now: DateTime<Utc>,
        lease_owner: &str,
        lease_until: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<BackgroundJobRecovery>, AppError>;

    /// Enqueues a replay through `job_outbox` and flips the ledger row back to
    /// `waiting` — new outbox id recorded, `recovery_attempts` incremented,
    /// lease cleared, `next_retry_at` pushed forward — in one repository-managed
    /// transaction, so a crash can neither lose the ledger update after the
    /// outbox insert nor double-replay the row on the next sweep. Fenced by
    /// claim ownership: fails (and rolls back the outbox insert) unless the row
    /// is still `leased` by `lease_owner` with an unexpired lease, so a stale
    /// claimant cannot resurrect a resolved row or double-enqueue after a
    /// takeover.
    async fn replay_via_outbox(&self, replay: RecoveryReplay<'_>) -> Result<JobOutboxId, AppError>;

    /// Moves a `waiting` row's `next_retry_at` without touching its attempt
    /// budget. Used to pace patient re-parks by their accumulated
    /// `recovery_attempts` after the upsert has preserved the counter.
    async fn reschedule_waiting(
        &self,
        id: BackgroundJobRecoveryId,
        next_retry_at: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> Result<(), AppError>;

    /// Sweeper-cap path: the row hit the recovery_attempts cap. Marks it
    /// `terminal` and writes one `dead_letter_jobs` row in the same
    /// transaction. Fenced by claim ownership: fails unless the row is still
    /// `leased` by `lease_owner` with an unexpired lease, so a stale claimant
    /// cannot dead-letter a row that resolved or re-classified (e.g. turned
    /// `patient`) after its claim lapsed; the fence also makes the DLQ write
    /// at-most-once per terminalization.
    async fn mark_recovery_terminal(
        &self,
        id: BackgroundJobRecoveryId,
        reason: &str,
        error: &str,
        lease_owner: &str,
        dlq_insert: DeadLetterInsert<'_>,
        now: DateTime<Utc>,
    ) -> Result<(), AppError>;

    /// Sweeper carve-out path: drop the lease, push `next_retry_at` forward,
    /// and increment `recovery_attempts`. Used when ownership of the replay
    /// belongs to a different sweeper (e.g. the item-pipeline auto-heal owns
    /// item.* jobs). Counting the deferral keeps these rows bounded by the
    /// same recovery cap as exact-replay rows.
    async fn defer_recovery(
        &self,
        id: BackgroundJobRecoveryId,
        next_retry_at: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> Result<(), AppError>;

    /// Returns recovery rows matching the filter. When `filter.status` is
    /// `None`, rows in `resolved` status are excluded so operators inspecting
    /// active work do not have to filter them out manually. Pass an explicit
    /// `Some(BackgroundJobRecoveryStatus::Resolved)` to inspect resolved rows.
    async fn list_active(
        &self,
        filter: ActiveRecoveryFilter,
        limit: i64,
    ) -> Result<Vec<BackgroundJobRecovery>, AppError>;
}
