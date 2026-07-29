use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::email_sender::EmailSenderRepository;
use ind_domain::{
    CanonicalAddress, DomainError, EmailDestination, EmailSender, EmailSenderId,
    EmailSenderRenderDefault, UserId,
};

pub struct PgEmailSenderRepository {
    pool: PgPool,
}

impl PgEmailSenderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl EmailSenderRepository for PgEmailSenderRepository {
    async fn upsert_for_user(
        &self,
        user_id: UserId,
        canonical_addr: &CanonicalAddress,
        list_id: Option<&str>,
        display_name: Option<&str>,
    ) -> Result<EmailSender, AppError> {
        let new_id = Uuid::now_v7();
        let row = sqlx::query_as!(
            EmailSenderRow,
            r#"
            INSERT INTO email_senders (
                id, user_id, canonical_addr, list_id, display_name,
                first_seen_at, last_seen_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, now(), now(), now(), now())
            ON CONFLICT (user_id, canonical_addr) DO UPDATE
            SET list_id = COALESCE(email_senders.list_id, EXCLUDED.list_id),
                display_name = COALESCE(EXCLUDED.display_name, email_senders.display_name),
                last_seen_at = now(),
                updated_at = now()
            RETURNING id, user_id, canonical_addr, list_id, display_name,
                      render_default, routing_default, blocked_at,
                      first_seen_at, last_seen_at, delivery_count,
                      created_at, updated_at
            "#,
            new_id,
            user_id.into_uuid(),
            canonical_addr.as_str(),
            list_id,
            display_name,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_err)?;

        row.try_into()
    }

    async fn find_by_user_and_canonical(
        &self,
        user_id: UserId,
        canonical_addr: &CanonicalAddress,
    ) -> Result<Option<EmailSender>, AppError> {
        let row = sqlx::query_as!(
            EmailSenderRow,
            r#"
            SELECT id, user_id, canonical_addr, list_id, display_name,
                   render_default, routing_default, blocked_at,
                   first_seen_at, last_seen_at, delivery_count,
                   created_at, updated_at
            FROM email_senders
            WHERE user_id = $1 AND canonical_addr = $2
            "#,
            user_id.into_uuid(),
            canonical_addr.as_str(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_err)?;

        row.map(EmailSender::try_from).transpose()
    }

    async fn find_by_id_and_user(
        &self,
        user_id: UserId,
        sender_id: EmailSenderId,
    ) -> Result<Option<EmailSender>, AppError> {
        let row = sqlx::query_as!(
            EmailSenderRow,
            r#"
            SELECT id, user_id, canonical_addr, list_id, display_name,
                   render_default, routing_default, blocked_at,
                   first_seen_at, last_seen_at, delivery_count,
                   created_at, updated_at
            FROM email_senders
            WHERE id = $1 AND user_id = $2
            "#,
            sender_id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_err)?;

        row.map(EmailSender::try_from).transpose()
    }

    async fn list_for_user(
        &self,
        user_id: UserId,
        offset: i64,
        limit: i64,
    ) -> Result<(Vec<EmailSender>, i64), AppError> {
        let total: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) AS \"count!\" FROM email_senders WHERE user_id = $1",
            user_id.into_uuid(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_err)?;

        let rows = sqlx::query_as!(
            EmailSenderRow,
            r#"
            SELECT id, user_id, canonical_addr, list_id, display_name,
                   render_default, routing_default, blocked_at,
                   first_seen_at, last_seen_at, delivery_count,
                   created_at, updated_at
            FROM email_senders
            WHERE user_id = $1
            ORDER BY last_seen_at DESC, id DESC
            OFFSET $2
            LIMIT $3
            "#,
            user_id.into_uuid(),
            offset,
            limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_err)?;

        let senders = rows
            .into_iter()
            .map(EmailSender::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok((senders, total))
    }

    async fn list_by_ids_for_user(
        &self,
        user_id: UserId,
        ids: &[EmailSenderId],
    ) -> Result<Vec<EmailSender>, AppError> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let uuids: Vec<uuid::Uuid> = ids.iter().map(|id| id.into_uuid()).collect();
        let rows = sqlx::query_as!(
            EmailSenderRow,
            r#"
            SELECT id, user_id, canonical_addr, list_id, display_name,
                   render_default, routing_default, blocked_at,
                   first_seen_at, last_seen_at, delivery_count,
                   created_at, updated_at
            FROM email_senders
            WHERE user_id = $1 AND id = ANY($2::uuid[])
            "#,
            user_id.into_uuid(),
            &uuids,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_err)?;

        rows.into_iter().map(EmailSender::try_from).collect()
    }

    async fn block(&self, sender_id: EmailSenderId) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE email_senders \
             SET blocked_at = COALESCE(blocked_at, now()), updated_at = now() \
             WHERE id = $1",
            sender_id.into_uuid(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn unblock(&self, sender_id: EmailSenderId) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE email_senders SET blocked_at = NULL, updated_at = now() WHERE id = $1",
            sender_id.into_uuid(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn set_render_default(
        &self,
        sender_id: EmailSenderId,
        value: EmailSenderRenderDefault,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE email_senders SET render_default = $2, updated_at = now() WHERE id = $1",
            sender_id.into_uuid(),
            value.as_str(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn set_routing_default(
        &self,
        sender_id: EmailSenderId,
        value: Option<EmailDestination>,
    ) -> Result<(), AppError> {
        let value_str = value.map(|v| v.as_str());
        sqlx::query!(
            "UPDATE email_senders SET routing_default = $2, updated_at = now() WHERE id = $1",
            sender_id.into_uuid(),
            value_str,
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn increment_delivery(&self, sender_id: EmailSenderId) -> Result<(), AppError> {
        sqlx::query!(
            r#"UPDATE email_senders
               SET delivery_count = delivery_count + 1,
                   last_seen_at = now(),
                   updated_at = now()
               WHERE id = $1"#,
            sender_id.into_uuid(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn block_for_user(
        &self,
        user_id: UserId,
        sender_id: EmailSenderId,
    ) -> Result<(), AppError> {
        // First-block-wins: COALESCE preserves the original blocked_at so a
        // re-click on Unsubscribe doesn't push the timestamp forward.
        let rows = sqlx::query!(
            "UPDATE email_senders \
             SET blocked_at = COALESCE(blocked_at, now()), updated_at = now() \
             WHERE id = $1 AND user_id = $2",
            sender_id.into_uuid(),
            user_id.into_uuid(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;
        if rows.rows_affected() == 0 {
            return Err(not_found_for_sender(sender_id));
        }
        Ok(())
    }

    async fn unblock_for_user(
        &self,
        user_id: UserId,
        sender_id: EmailSenderId,
    ) -> Result<(), AppError> {
        let rows = sqlx::query!(
            "UPDATE email_senders SET blocked_at = NULL, updated_at = now() \
             WHERE id = $1 AND user_id = $2",
            sender_id.into_uuid(),
            user_id.into_uuid(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;
        if rows.rows_affected() == 0 {
            return Err(not_found_for_sender(sender_id));
        }
        Ok(())
    }

    async fn set_render_default_for_user(
        &self,
        user_id: UserId,
        sender_id: EmailSenderId,
        value: EmailSenderRenderDefault,
    ) -> Result<(), AppError> {
        let rows = sqlx::query!(
            "UPDATE email_senders SET render_default = $3, updated_at = now() \
             WHERE id = $1 AND user_id = $2",
            sender_id.into_uuid(),
            user_id.into_uuid(),
            value.as_str(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;
        if rows.rows_affected() == 0 {
            return Err(not_found_for_sender(sender_id));
        }
        Ok(())
    }

    async fn set_routing_default_for_user(
        &self,
        user_id: UserId,
        sender_id: EmailSenderId,
        value: Option<EmailDestination>,
    ) -> Result<(), AppError> {
        let value_str = value.map(|v| v.as_str());
        let rows = sqlx::query!(
            "UPDATE email_senders SET routing_default = $3, updated_at = now() \
             WHERE id = $1 AND user_id = $2",
            sender_id.into_uuid(),
            user_id.into_uuid(),
            value_str,
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;
        if rows.rows_affected() == 0 {
            return Err(not_found_for_sender(sender_id));
        }
        Ok(())
    }
}

fn not_found_for_sender(sender_id: EmailSenderId) -> AppError {
    AppError::Domain(DomainError::NotFound {
        entity: "EmailSender",
        id: sender_id.to_string(),
    })
}

pub(super) struct EmailSenderRow {
    pub(super) id: Uuid,
    pub(super) user_id: Uuid,
    pub(super) canonical_addr: String,
    pub(super) list_id: Option<String>,
    pub(super) display_name: Option<String>,
    pub(super) render_default: String,
    pub(super) routing_default: Option<String>,
    pub(super) blocked_at: Option<DateTime<Utc>>,
    pub(super) first_seen_at: DateTime<Utc>,
    pub(super) last_seen_at: DateTime<Utc>,
    pub(super) delivery_count: i32,
    pub(super) created_at: DateTime<Utc>,
    pub(super) updated_at: DateTime<Utc>,
}

impl TryFrom<EmailSenderRow> for EmailSender {
    type Error = AppError;

    fn try_from(row: EmailSenderRow) -> Result<Self, Self::Error> {
        Ok(EmailSender {
            id: EmailSenderId::from_uuid(row.id),
            user_id: UserId::from_uuid(row.user_id),
            canonical_addr: row.canonical_addr,
            list_id: row.list_id,
            display_name: row.display_name,
            render_default: parse_render_default(&row.render_default)?,
            routing_default: row
                .routing_default
                .as_deref()
                .map(parse_email_destination)
                .transpose()?,
            blocked_at: row.blocked_at,
            first_seen_at: row.first_seen_at,
            last_seen_at: row.last_seen_at,
            delivery_count: row.delivery_count,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

fn parse_render_default(s: &str) -> Result<EmailSenderRenderDefault, AppError> {
    match s {
        "reader" => Ok(EmailSenderRenderDefault::Reader),
        "original" => Ok(EmailSenderRenderDefault::Original),
        other => Err(AppError::Domain(DomainError::InvariantViolation {
            message: format!("invalid email_senders.render_default: {other}"),
        })),
    }
}

fn parse_email_destination(s: &str) -> Result<EmailDestination, AppError> {
    match s {
        "feed" => Ok(EmailDestination::Feed),
        "library" => Ok(EmailDestination::Library),
        other => Err(AppError::Domain(DomainError::InvariantViolation {
            message: format!("invalid email destination: {other}"),
        })),
    }
}

fn map_err(err: sqlx::Error) -> AppError {
    AppError::Repository(Box::new(err))
}
