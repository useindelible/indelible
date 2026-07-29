use chrono::{DateTime, Utc};
use sqlx::PgPool;

use ind_application::AppError;
use ind_application::repos::email_unsubscribe_target::{
    EmailUnsubscribeTarget, EmailUnsubscribeTargetRepository, UnsubscribeTargetUpsert,
};
use ind_domain::EmailSenderId;

pub struct PgEmailUnsubscribeTargetRepository {
    pool: PgPool,
}

impl PgEmailUnsubscribeTargetRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl EmailUnsubscribeTargetRepository for PgEmailUnsubscribeTargetRepository {
    async fn upsert(
        &self,
        sender_id: EmailSenderId,
        targets: UnsubscribeTargetUpsert,
    ) -> Result<EmailUnsubscribeTarget, AppError> {
        let row = sqlx::query_as!(
            EmailUnsubscribeTargetRow,
            r#"
            INSERT INTO email_unsubscribe_targets (
                sender_id, one_click_post_url, mailto_addr, web_url, last_seen_at
            )
            VALUES ($1, $2, $3, $4, now())
            ON CONFLICT (sender_id) DO UPDATE
            SET one_click_post_url = EXCLUDED.one_click_post_url,
                mailto_addr        = EXCLUDED.mailto_addr,
                web_url            = EXCLUDED.web_url,
                last_seen_at       = now()
            RETURNING sender_id, one_click_post_url, mailto_addr, web_url, last_seen_at
            "#,
            sender_id.into_uuid(),
            targets.one_click_post_url,
            targets.mailto_addr,
            targets.web_url,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(row.into())
    }

    async fn find_by_sender(
        &self,
        sender_id: EmailSenderId,
    ) -> Result<Option<EmailUnsubscribeTarget>, AppError> {
        let row = sqlx::query_as!(
            EmailUnsubscribeTargetRow,
            r#"
            SELECT sender_id, one_click_post_url, mailto_addr, web_url, last_seen_at
            FROM email_unsubscribe_targets
            WHERE sender_id = $1
            "#,
            sender_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(row.map(Into::into))
    }
}

struct EmailUnsubscribeTargetRow {
    sender_id: uuid::Uuid,
    one_click_post_url: Option<String>,
    mailto_addr: Option<String>,
    web_url: Option<String>,
    last_seen_at: DateTime<Utc>,
}

impl From<EmailUnsubscribeTargetRow> for EmailUnsubscribeTarget {
    fn from(row: EmailUnsubscribeTargetRow) -> Self {
        Self {
            sender_id: EmailSenderId::from_uuid(row.sender_id),
            one_click_post_url: row.one_click_post_url,
            mailto_addr: row.mailto_addr,
            web_url: row.web_url,
            last_seen_at: row.last_seen_at,
        }
    }
}

fn map_err(err: sqlx::Error) -> AppError {
    AppError::Repository(Box::new(err))
}
