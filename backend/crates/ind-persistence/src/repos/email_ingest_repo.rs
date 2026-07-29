use chrono::Utc;
use sqlx::PgPool;

use ind_application::AppError;
use ind_application::repos::email_ingest::{
    ClaimAndEnqueueInput, EmailIngestLogRepository, EmailIngestLogRow,
};
use ind_domain::{JobOutboxId, UserId, parse_from_header};

pub struct PgEmailIngestLogRepository {
    pool: PgPool,
}

impl PgEmailIngestLogRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl EmailIngestLogRepository for PgEmailIngestLogRepository {
    async fn claim_and_enqueue(
        &self,
        input: ClaimAndEnqueueInput<'_>,
    ) -> Result<Option<EmailIngestLogRow>, AppError> {
        let provider = input.provider;
        let provider_email_id = input.provider_email_id;
        let user_id = input.user_id;
        let destination = input.destination;
        let job_type = input.job_type;
        let job_payload = input.job_payload;
        let raw_payload = input.raw_payload;
        let list_id = input.list_id;

        let (canonical, display_name) = parse_from_header(input.from_address);
        let canonical_value = canonical.as_str().to_string();
        let display_name_ref = display_name.as_deref();

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Repository(Box::new(e)))?;

        let now = Utc::now();

        // Sender upsert: own the row before we decide whether the delivery is blocked.
        // delivery_count is left untouched — the worker bumps it after successful processing.
        let sender_row = sqlx::query!(
            r#"
            INSERT INTO email_senders (
                user_id, canonical_addr, list_id, display_name,
                first_seen_at, last_seen_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $5, $5, $5)
            ON CONFLICT (user_id, canonical_addr) DO UPDATE SET
                list_id      = COALESCE(email_senders.list_id, EXCLUDED.list_id),
                display_name = COALESCE(EXCLUDED.display_name, email_senders.display_name),
                last_seen_at = EXCLUDED.last_seen_at,
                updated_at   = EXCLUDED.updated_at
            RETURNING id, blocked_at, list_id, routing_default
            "#,
            user_id.into_uuid(),
            canonical_value,
            list_id,
            display_name_ref,
            now,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::Repository(Box::new(e)))?;

        // Per-sender routing override wins over the alias-derived destination.
        // Mutates both the email_ingest_log row and the enqueued job payload so
        // the worker dispatches to the sender's preferred destination.
        let effective_destination = sender_row.routing_default.as_deref().unwrap_or(destination);

        let sender_blocked = sender_row.blocked_at.is_some();
        let list_id_blocked = if !sender_blocked {
            if let Some(effective_list_id) = sender_row.list_id.as_deref() {
                sqlx::query_scalar!(
                    r#"
                    SELECT EXISTS (
                        SELECT 1 FROM email_senders
                        WHERE user_id = $1 AND list_id = $2 AND blocked_at IS NOT NULL
                    ) AS "exists!"
                    "#,
                    user_id.into_uuid(),
                    effective_list_id,
                )
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| AppError::Repository(Box::new(e)))?
            } else {
                false
            }
        } else {
            false
        };

        let is_blocked = sender_blocked || list_id_blocked;
        let log_id = uuid::Uuid::now_v7();
        let initial_status = if is_blocked { "blocked" } else { "pending" };

        let processed_at = if is_blocked { Some(now) } else { None };

        let row = sqlx::query_as!(
            DbEmailIngestLogRow,
            r#"
            INSERT INTO email_ingest_log (id, provider, provider_email_id, user_id, destination, status, raw_payload, created_at, processed_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT ON CONSTRAINT uq_email_ingest_delivery DO NOTHING
            RETURNING id, provider, provider_email_id, user_id, destination, status, error, created_at, processed_at
            "#,
            log_id,
            provider,
            provider_email_id,
            user_id.into_uuid(),
            effective_destination,
            initial_status,
            raw_payload,
            now,
            processed_at,
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| AppError::Repository(Box::new(e)))?;

        let Some(row) = row else {
            tx.rollback()
                .await
                .map_err(|e| AppError::Repository(Box::new(e)))?;
            return Ok(None);
        };

        if !is_blocked {
            let outbox_id = JobOutboxId::new();
            let mut enriched_payload = job_payload;
            if let serde_json::Value::Object(ref mut map) = enriched_payload {
                map.insert(
                    "ingest_log_id".to_string(),
                    serde_json::Value::String(log_id.to_string()),
                );
                map.insert(
                    "destination".to_string(),
                    serde_json::Value::String(effective_destination.to_string()),
                );
            }
            sqlx::query!(
                r#"
                INSERT INTO job_outbox (id, job_type, payload, dedupe_key, available_at, created_at)
                VALUES ($1, $2, $3, NULL, $4, $5)
                "#,
                outbox_id.as_uuid(),
                job_type,
                enriched_payload,
                now,
                now,
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Repository(Box::new(e)))?;
        }

        tx.commit()
            .await
            .map_err(|e| AppError::Repository(Box::new(e)))?;

        Ok(Some(EmailIngestLogRow {
            id: row.id,
            provider: row.provider,
            provider_email_id: row.provider_email_id,
            user_id: UserId::from_uuid(row.user_id),
            destination: row.destination,
            status: row.status,
            error: row.error,
            created_at: row.created_at,
            processed_at: row.processed_at,
        }))
    }

    async fn mark_processed(&self, id: uuid::Uuid) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE email_ingest_log SET status = 'processed', processed_at = now() WHERE id = $1",
            id,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Repository(Box::new(e)))?;

        Ok(())
    }

    async fn mark_failed(&self, id: uuid::Uuid, error: &str) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE email_ingest_log SET status = 'failed', error = $2, processed_at = now() WHERE id = $1",
            id,
            error,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Repository(Box::new(e)))?;

        Ok(())
    }

    async fn mark_gmail_confirmation(&self, id: uuid::Uuid) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE email_ingest_log SET status = 'gmail_confirmation', processed_at = now() WHERE id = $1",
            id,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Repository(Box::new(e)))?;

        Ok(())
    }
}

struct DbEmailIngestLogRow {
    id: uuid::Uuid,
    provider: String,
    provider_email_id: String,
    user_id: uuid::Uuid,
    destination: String,
    status: String,
    error: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    processed_at: Option<chrono::DateTime<chrono::Utc>>,
}
