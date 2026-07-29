use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::email_verification::EmailVerificationTokenRepository;
use ind_domain::{DomainError, EmailVerificationToken, UserId};

pub struct PgEmailVerificationRepository {
    pool: PgPool,
}

impl PgEmailVerificationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct EmailVerificationRow {
    id: Uuid,
    user_id: Uuid,
    token_hash: String,
    expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

impl From<EmailVerificationRow> for EmailVerificationToken {
    fn from(row: EmailVerificationRow) -> Self {
        EmailVerificationToken {
            id: row.id,
            user_id: UserId::from_uuid(row.user_id),
            token_hash: row.token_hash,
            expires_at: row.expires_at,
            created_at: row.created_at,
        }
    }
}

fn map_sqlx_error(err: sqlx::Error) -> AppError {
    super::map_sqlx_error(
        "email_verification_token",
        "duplicate verification token",
        err,
    )
}

#[async_trait::async_trait]
impl EmailVerificationTokenRepository for PgEmailVerificationRepository {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn create(
        &self,
        token: EmailVerificationToken,
    ) -> Result<EmailVerificationToken, AppError> {
        let row = sqlx::query_as!(
            EmailVerificationRow,
            "INSERT INTO email_verification_tokens (id, user_id, token_hash, expires_at, created_at) \
             VALUES ($1, $2, $3, $4, $5) \
             RETURNING id, user_id, token_hash, expires_at, created_at",
            token.id,
            token.user_id.into_uuid(),
            token.token_hash,
            token.expires_at,
            token.created_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(EmailVerificationToken::from(row))
    }

    async fn find_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<EmailVerificationToken>, AppError> {
        let row = sqlx::query_as!(
            EmailVerificationRow,
            "SELECT id, user_id, token_hash, expires_at, created_at \
             FROM email_verification_tokens WHERE token_hash = $1",
            token_hash
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(EmailVerificationToken::from))
    }

    async fn find_latest_for_user(
        &self,
        user_id: UserId,
    ) -> Result<Option<EmailVerificationToken>, AppError> {
        let row = sqlx::query_as!(
            EmailVerificationRow,
            "SELECT id, user_id, token_hash, expires_at, created_at \
             FROM email_verification_tokens WHERE user_id = $1 \
             ORDER BY created_at DESC LIMIT 1",
            user_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(EmailVerificationToken::from))
    }

    async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query!("DELETE FROM email_verification_tokens WHERE id = $1", id)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(AppError::Domain(DomainError::NotFound {
                entity: "email_verification_token",
                id: id.to_string(),
            }));
        }

        Ok(())
    }

    async fn delete_all_for_user(&self, user_id: UserId) -> Result<u64, AppError> {
        let result = sqlx::query!(
            "DELETE FROM email_verification_tokens WHERE user_id = $1",
            user_id.into_uuid()
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(result.rows_affected())
    }

    async fn delete_expired(&self) -> Result<u64, AppError> {
        let result = sqlx::query!("DELETE FROM email_verification_tokens WHERE expires_at < now()")
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

        Ok(result.rows_affected())
    }
}
