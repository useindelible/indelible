use crate::error::AppError;
use ind_domain::UserId;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct EmailIngestLogRow {
    pub id: Uuid,
    pub provider: String,
    pub provider_email_id: String,
    pub user_id: UserId,
    pub destination: String,
    pub status: String,
    pub error: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub processed_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct ClaimAndEnqueueInput<'a> {
    pub provider: &'a str,
    pub provider_email_id: &'a str,
    pub user_id: UserId,
    pub destination: &'a str,
    pub job_type: &'a str,
    pub job_payload: serde_json::Value,
    pub raw_payload: Option<&'a [u8]>,
    /// `From:` header verbatim (e.g. `"Display Name <addr@host>"` or `"addr@host"`).
    /// Canonicalized and used for sender upsert + block enforcement.
    pub from_address: &'a str,
    /// `List-ID:` header verbatim (with brackets) if present, used for list-scoped blocking.
    pub list_id: Option<&'a str>,
}

#[async_trait::async_trait]
pub trait EmailIngestLogRepository: Send + Sync {
    /// Atomically insert a log row (status='pending') and enqueue a job to the outbox.
    /// Returns None if the `(provider, provider_email_id, user_id, destination)` combination
    /// already exists (idempotent — no duplicate job enqueued).
    async fn claim_and_enqueue(
        &self,
        input: ClaimAndEnqueueInput<'_>,
    ) -> Result<Option<EmailIngestLogRow>, AppError>;

    async fn mark_processed(&self, id: Uuid) -> Result<(), AppError>;

    async fn mark_failed(&self, id: Uuid, error: &str) -> Result<(), AppError>;

    /// Marks the row as a Gmail forwarding-confirmation email (auto-handled
    /// out-of-band). Sets `processed_at` so it leaves the pending backlog.
    async fn mark_gmail_confirmation(&self, id: Uuid) -> Result<(), AppError>;
}
