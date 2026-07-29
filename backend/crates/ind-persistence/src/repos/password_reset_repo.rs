use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::password_reset::PasswordResetTokenRepository;
use ind_domain::{DomainError, PasswordResetToken, UserId};

pub struct PgPasswordResetRepository {
    pool: PgPool,
}

impl PgPasswordResetRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct PasswordResetRow {
    id: Uuid,
    user_id: Uuid,
    token_hash: String,
    expires_at: DateTime<Utc>,
    used_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl From<PasswordResetRow> for PasswordResetToken {
    fn from(row: PasswordResetRow) -> Self {
        PasswordResetToken {
            id: row.id,
            user_id: UserId::from_uuid(row.user_id),
            token_hash: row.token_hash,
            expires_at: row.expires_at,
            used_at: row.used_at,
            created_at: row.created_at,
        }
    }
}

fn map_sqlx_error(err: sqlx::Error) -> AppError {
    super::map_sqlx_error("password_reset_token", "duplicate reset token", err)
}

#[async_trait::async_trait]
impl PasswordResetTokenRepository for PgPasswordResetRepository {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn create(&self, token: PasswordResetToken) -> Result<PasswordResetToken, AppError> {
        let row = sqlx::query_as!(
            PasswordResetRow,
            "INSERT INTO password_reset_tokens (id, user_id, token_hash, expires_at, used_at, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             RETURNING id, user_id, token_hash, expires_at, used_at, created_at",
            token.id,
            token.user_id.into_uuid(),
            token.token_hash,
            token.expires_at,
            token.used_at,
            token.created_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(PasswordResetToken::from(row))
    }

    async fn find_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<PasswordResetToken>, AppError> {
        let row = sqlx::query_as!(
            PasswordResetRow,
            "SELECT id, user_id, token_hash, expires_at, used_at, created_at \
             FROM password_reset_tokens WHERE token_hash = $1",
            token_hash
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(PasswordResetToken::from))
    }

    async fn mark_used(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query!(
            "UPDATE password_reset_tokens SET used_at = now() WHERE id = $1 AND used_at IS NULL",
            id,
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            let exists = sqlx::query_scalar!(
                "SELECT used_at FROM password_reset_tokens WHERE id = $1",
                id,
            )
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

            return match exists {
                Some(Some(_)) => Err(AppError::Domain(DomainError::Conflict {
                    entity: "password_reset_token",
                    message: "token already used".to_string(),
                })),
                _ => Err(AppError::Domain(DomainError::NotFound {
                    entity: "password_reset_token",
                    id: id.to_string(),
                })),
            };
        }

        Ok(())
    }

    async fn delete_all_for_user(&self, user_id: UserId) -> Result<u64, AppError> {
        let result = sqlx::query!(
            "DELETE FROM password_reset_tokens WHERE user_id = $1",
            user_id.into_uuid()
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(result.rows_affected())
    }

    async fn delete_expired(&self) -> Result<u64, AppError> {
        let result = sqlx::query!("DELETE FROM password_reset_tokens WHERE expires_at < now()")
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

        Ok(result.rows_affected())
    }
}
