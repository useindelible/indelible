use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{BackgroundJobRecoveryId, DeadLetterJobId, DomainEventId, JobOutboxId, UserId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackgroundJobRecoveryStatus {
    Waiting,
    Leased,
    Terminal,
    Resolved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackgroundJobFailureClass {
    Retryable,
    Terminal,
    /// Dependency-outage failures that wait indefinitely on a sparse capped backoff
    /// instead of counting toward the dead-letter cap. The class is generic; each
    /// dependency opts in through its own worker-side classifier.
    Patient,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackgroundJobSubjectKind {
    Document,
    LibraryEntry,
    FeedDelivery,
    FeedSource,
    IntegrationConnection,
    ImportJob,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundJobRecovery {
    pub id: BackgroundJobRecoveryId,
    pub recovery_key: String,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub dedupe_key: Option<String>,
    pub outbox_id: Option<JobOutboxId>,
    pub subject_kind: Option<BackgroundJobSubjectKind>,
    pub subject_id: Option<String>,
    pub status: BackgroundJobRecoveryStatus,
    pub failure_class: BackgroundJobFailureClass,
    pub failure_reason_code: String,
    pub error_message: String,
    pub apalis_attempts: i32,
    pub recovery_attempts: i32,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub lease_owner: Option<String>,
    pub lease_expires_at: Option<DateTime<Utc>>,
    pub first_failed_at: DateTime<Utc>,
    pub last_failed_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobOutbox {
    pub id: JobOutboxId,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub dedupe_key: Option<String>,
    pub available_at: DateTime<Utc>,
    pub dispatched_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewDomainEvent {
    pub id: DomainEventId,
    pub event_type: String,
    pub aggregate_type: String,
    pub aggregate_id: Uuid,
    pub user_id: UserId,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    pub id: DomainEventId,
    pub event_type: String,
    pub aggregate_type: String,
    pub aggregate_id: Uuid,
    pub user_id: UserId,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterJob {
    pub id: DeadLetterJobId,
    pub original_job_type: String,
    pub original_payload: serde_json::Value,
    pub original_dedupe_key: Option<String>,
    pub failure_reason_code: Option<String>,
    pub error_message: String,
    pub attempts: i32,
    pub failed_at: DateTime<Utc>,
    pub replayed_at: Option<DateTime<Utc>>,
    pub replay_outbox_id: Option<JobOutboxId>,
}

pub fn build_domain_event(
    event_type: &str,
    aggregate_type: &str,
    aggregate_id: Uuid,
    user_id: UserId,
    payload: serde_json::Value,
) -> NewDomainEvent {
    NewDomainEvent {
        id: DomainEventId::new(),
        event_type: event_type.into(),
        aggregate_type: aggregate_type.into(),
        aggregate_id,
        user_id,
        payload,
        created_at: Utc::now(),
    }
}
