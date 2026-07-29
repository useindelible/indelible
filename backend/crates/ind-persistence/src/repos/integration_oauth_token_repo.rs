use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::integration_oauth_token::IntegrationOAuthTokenRepository;
use ind_domain::{DomainError, IntegrationOAuthProvider, IntegrationOAuthToken, UserId};

pub struct PgIntegrationOAuthTokenRepository {
    pool: PgPool,
}

impl PgIntegrationOAuthTokenRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct TokenRow {
    id: Uuid,
    user_id: Uuid,
    provider: String,
    access_token_enc: Vec<u8>,
    refresh_token_enc: Option<Vec<u8>>,
    token_expires_at: Option<DateTime<Utc>>,
    extra: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<TokenRow> for IntegrationOAuthToken {
    type Error = AppError;

    fn try_from(row: TokenRow) -> Result<Self, Self::Error> {
        Ok(IntegrationOAuthToken {
            id: row.id,
            user_id: UserId::from_uuid(row.user_id),
            provider: parse_provider(&row.provider)?,
            access_token_enc: row.access_token_enc,
            refresh_token_enc: row.refresh_token_enc,
            token_expires_at: row.token_expires_at,
            extra: row.extra,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

fn parse_provider(s: &str) -> Result<IntegrationOAuthProvider, AppError> {
    match s {
        "notion" => Ok(IntegrationOAuthProvider::Notion),
        other => Err(AppError::Domain(DomainError::InvariantViolation {
            message: format!("invalid integration oauth provider: {other}"),
        })),
    }
}

fn map_err(err: sqlx::Error) -> AppError {
    super::map_sqlx_error(
        "integration_oauth_token",
        "integration oauth token already exists",
        err,
    )
}

#[async_trait::async_trait]
impl IntegrationOAuthTokenRepository for PgIntegrationOAuthTokenRepository {
    async fn upsert(
        &self,
        user_id: UserId,
        provider: IntegrationOAuthProvider,
        access_token_enc: Vec<u8>,
        refresh_token_enc: Option<Vec<u8>>,
        token_expires_at: Option<DateTime<Utc>>,
        extra: serde_json::Value,
    ) -> Result<IntegrationOAuthToken, AppError> {
        let now = Utc::now();
        let id = Uuid::now_v7();
        let row = sqlx::query_as!(
            TokenRow,
            r#"INSERT INTO integration_oauth_tokens
                (id, user_id, provider, access_token_enc, refresh_token_enc, token_expires_at, extra, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8)
               ON CONFLICT (user_id, provider) DO UPDATE
                 SET access_token_enc = EXCLUDED.access_token_enc,
                     refresh_token_enc = EXCLUDED.refresh_token_enc,
                     token_expires_at = EXCLUDED.token_expires_at,
                     extra = EXCLUDED.extra,
                     updated_at = EXCLUDED.updated_at
               RETURNING id, user_id, provider, access_token_enc, refresh_token_enc, token_expires_at, extra, created_at, updated_at"#,
            id,
            user_id.into_uuid(),
            provider.as_str(),
            access_token_enc,
            refresh_token_enc,
            token_expires_at,
            extra,
            now,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_err)?;

        IntegrationOAuthToken::try_from(row)
    }

    async fn find_by_user_provider(
        &self,
        user_id: UserId,
        provider: IntegrationOAuthProvider,
    ) -> Result<Option<IntegrationOAuthToken>, AppError> {
        let row = sqlx::query_as!(
            TokenRow,
            r#"SELECT id, user_id, provider, access_token_enc, refresh_token_enc, token_expires_at, extra, created_at, updated_at
               FROM integration_oauth_tokens
               WHERE user_id = $1 AND provider = $2"#,
            user_id.into_uuid(),
            provider.as_str(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_err)?;

        row.map(IntegrationOAuthToken::try_from).transpose()
    }

    async fn set_tokens(
        &self,
        id: Uuid,
        access_token_enc: Vec<u8>,
        refresh_token_enc: Option<Vec<u8>>,
        token_expires_at: Option<DateTime<Utc>>,
    ) -> Result<(), AppError> {
        let result = sqlx::query!(
            r#"UPDATE integration_oauth_tokens
               SET access_token_enc = $2,
                   refresh_token_enc = $3,
                   token_expires_at = $4,
                   updated_at = now()
               WHERE id = $1"#,
            id,
            access_token_enc,
            refresh_token_enc,
            token_expires_at,
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;

        if result.rows_affected() == 0 {
            return Err(AppError::Domain(DomainError::NotFound {
                entity: "integration_oauth_token",
                id: id.to_string(),
            }));
        }
        Ok(())
    }

    async fn delete_by_user_provider(
        &self,
        user_id: UserId,
        provider: IntegrationOAuthProvider,
    ) -> Result<u64, AppError> {
        let result = sqlx::query!(
            r#"DELETE FROM integration_oauth_tokens
               WHERE user_id = $1 AND provider = $2"#,
            user_id.into_uuid(),
            provider.as_str(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;
        Ok(result.rows_affected())
    }
}
