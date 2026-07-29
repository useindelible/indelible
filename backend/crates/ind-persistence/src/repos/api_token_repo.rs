use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::api_token::ApiTokenRepository;
use ind_domain::{ApiToken, ApiTokenId, DomainError, UserId};

pub struct PgApiTokenRepository {
    pool: PgPool,
}

impl PgApiTokenRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct ApiTokenRow {
    id: Uuid,
    user_id: Uuid,
    name: String,
    token_hash: String,
    prefix: String,
    scopes: Vec<String>,
    last_used_at: Option<DateTime<Utc>>,
    expires_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl From<ApiTokenRow> for ApiToken {
    fn from(row: ApiTokenRow) -> Self {
        ApiToken {
            id: ApiTokenId::from_uuid(row.id),
            user_id: UserId::from_uuid(row.user_id),
            name: row.name,
            token_hash: row.token_hash,
            prefix: row.prefix,
            scopes: row.scopes,
            last_used_at: row.last_used_at,
            expires_at: row.expires_at,
            created_at: row.created_at,
        }
    }
}

fn map_sqlx_error(err: sqlx::Error) -> AppError {
    super::map_sqlx_error("api_token", "duplicate API token", err)
}

#[async_trait::async_trait]
impl ApiTokenRepository for PgApiTokenRepository {
    async fn create(&self, token: ApiToken) -> Result<ApiToken, AppError> {
        let row = sqlx::query_as!(
            ApiTokenRow,
            "INSERT INTO api_tokens (id, user_id, name, token_hash, prefix, \
             scopes, last_used_at, expires_at, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
             RETURNING id, user_id, name, token_hash, prefix, \
             scopes, last_used_at, expires_at, created_at",
            token.id.into_uuid(),
            token.user_id.into_uuid(),
            token.name,
            token.token_hash,
            token.prefix,
            &token.scopes,
            token.last_used_at,
            token.expires_at,
            token.created_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(ApiToken::from(row))
    }

    async fn find_by_id(
        &self,
        id: ApiTokenId,
        user_id: UserId,
    ) -> Result<Option<ApiToken>, AppError> {
        let row = sqlx::query_as!(
            ApiTokenRow,
            "SELECT id, user_id, name, token_hash, prefix, \
             scopes, last_used_at, expires_at, created_at \
             FROM api_tokens WHERE id = $1 AND user_id = $2",
            id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(ApiToken::from))
    }

    async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<ApiToken>, AppError> {
        let row = sqlx::query_as!(
            ApiTokenRow,
            "SELECT id, user_id, name, token_hash, prefix, \
             scopes, last_used_at, expires_at, created_at \
             FROM api_tokens WHERE token_hash = $1",
            token_hash,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(ApiToken::from))
    }

    async fn list_by_user(&self, user_id: UserId) -> Result<Vec<ApiToken>, AppError> {
        let rows = sqlx::query_as!(
            ApiTokenRow,
            "SELECT id, user_id, name, token_hash, prefix, \
             scopes, last_used_at, expires_at, created_at \
             FROM api_tokens WHERE user_id = $1 \
             ORDER BY created_at DESC",
            user_id.into_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(ApiToken::from).collect())
    }

    async fn update_last_used(&self, id: ApiTokenId) -> Result<(), AppError> {
        let result = sqlx::query!(
            "UPDATE api_tokens SET last_used_at = now() WHERE id = $1",
            id.into_uuid(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(AppError::Domain(DomainError::NotFound {
                entity: "api_token",
                id: id.to_string(),
            }));
        }

        Ok(())
    }

    async fn delete(&self, id: ApiTokenId, user_id: UserId) -> Result<(), AppError> {
        let result = sqlx::query!(
            "DELETE FROM api_tokens WHERE id = $1 AND user_id = $2",
            id.into_uuid(),
            user_id.into_uuid(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(AppError::Domain(DomainError::NotFound {
                entity: "api_token",
                id: id.to_string(),
            }));
        }

        Ok(())
    }
}
