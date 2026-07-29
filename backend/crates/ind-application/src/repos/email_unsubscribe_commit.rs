use chrono::{DateTime, Utc};

use crate::error::AppError;
use ind_domain::{EmailSender, EmailSenderId, JobOutbox, UserId};

/// Outcome of an atomic block + outbox-enqueue for an Unsubscribe action.
pub struct EmailUnsubscribeCommitOutcome {
    pub sender: EmailSender,
    pub outbox: JobOutbox,
}

/// Atomically sets `email_senders.blocked_at` (first-block-wins, via COALESCE)
/// and enqueues an `email.unsubscribe` outbox row deduped on
/// `(user_id, sender_id)`. The block UPDATE and outbox INSERT share a single
/// Postgres transaction so a process kill mid-flight never leaves the user
/// locally blocked without an upstream notification ever queued.
#[async_trait::async_trait]
pub trait EmailUnsubscribeCommit: Send + Sync {
    async fn commit_unsubscribe(
        &self,
        user_id: UserId,
        sender_id: EmailSenderId,
        payload: serde_json::Value,
        dedupe_key: String,
        available_at: DateTime<Utc>,
    ) -> Result<EmailUnsubscribeCommitOutcome, AppError>;
}
