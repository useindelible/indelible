use chrono::{DateTime, Utc};
use sqlx::PgPool;

use ind_application::error::AppError;
use ind_application::repos::email_unsubscribe_commit::{
    EmailUnsubscribeCommit, EmailUnsubscribeCommitOutcome,
};
use ind_domain::{DomainError, EmailSender, EmailSenderId, JobOutbox, JobOutboxId, UserId};

use super::email_sender_repo::EmailSenderRow;

pub struct PgEmailUnsubscribeCommit {
    pool: PgPool,
}

impl PgEmailUnsubscribeCommit {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl EmailUnsubscribeCommit for PgEmailUnsubscribeCommit {
    async fn commit_unsubscribe(
        &self,
        user_id: UserId,
        sender_id: EmailSenderId,
        payload: serde_json::Value,
        dedupe_key: String,
        available_at: DateTime<Utc>,
    ) -> Result<EmailUnsubscribeCommitOutcome, AppError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;

        let sender_row: Option<EmailSenderRow> = sqlx::query_as!(
            EmailSenderRow,
            r#"
            UPDATE email_senders
            SET blocked_at = COALESCE(blocked_at, now()),
                updated_at = now()
            WHERE id = $1 AND user_id = $2
            RETURNING id, user_id, canonical_addr, list_id, display_name,
                      render_default, routing_default, blocked_at,
                      first_seen_at, last_seen_at, delivery_count,
                      created_at, updated_at
            "#,
            sender_id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;

        let sender = sender_row
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "EmailSender",
                    id: sender_id.to_string(),
                })
            })
            .and_then(EmailSender::try_from)?;

        let now = Utc::now();
        let new_outbox_id = JobOutboxId::new();
        let outbox_row = sqlx::query_as!(
            OutboxInsertRow,
            r#"
            INSERT INTO job_outbox (id, job_type, payload, dedupe_key, available_at, created_at)
            VALUES ($1, 'email.unsubscribe', $2, $3, $4, $5)
            ON CONFLICT (dedupe_key) WHERE dedupe_key IS NOT NULL DO UPDATE
                SET payload = EXCLUDED.payload,
                    available_at = CASE
                        WHEN job_outbox.dispatched_at IS NULL
                            THEN LEAST(job_outbox.available_at, EXCLUDED.available_at)
                        ELSE EXCLUDED.available_at
                    END,
                    dispatched_at = NULL
            RETURNING id, job_type, payload, dedupe_key, available_at, dispatched_at, created_at
            "#,
            new_outbox_id.as_uuid(),
            payload,
            dedupe_key,
            available_at,
            now,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;

        tx.commit()
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;

        let outbox = JobOutbox {
            id: JobOutboxId::from(outbox_row.id),
            job_type: outbox_row.job_type,
            payload: outbox_row.payload,
            dedupe_key: outbox_row.dedupe_key,
            available_at: outbox_row.available_at,
            dispatched_at: outbox_row.dispatched_at,
            created_at: outbox_row.created_at,
        };

        Ok(EmailUnsubscribeCommitOutcome { sender, outbox })
    }
}

struct OutboxInsertRow {
    id: uuid::Uuid,
    job_type: String,
    payload: serde_json::Value,
    dedupe_key: Option<String>,
    available_at: DateTime<Utc>,
    dispatched_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}
