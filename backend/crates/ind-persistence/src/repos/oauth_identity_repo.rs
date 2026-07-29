use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::oauth_identity::OAuthIdentityRepository;
use ind_domain::{DomainError, OAuthIdentity, OAuthIdentityId, OAuthProvider, UserId};

pub struct PgOAuthIdentityRepository {
    pool: PgPool,
}

impl PgOAuthIdentityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct OAuthIdentityRow {
    id: Uuid,
    user_id: Uuid,
    provider: String,
    provider_user_id: String,
    provider_email: Option<String>,
    access_token_enc: Option<Vec<u8>>,
    refresh_token_enc: Option<Vec<u8>>,
    created_at: DateTime<Utc>,
}

impl TryFrom<OAuthIdentityRow> for OAuthIdentity {
    type Error = AppError;

    fn try_from(row: OAuthIdentityRow) -> Result<Self, Self::Error> {
        let provider = parse_oauth_provider(&row.provider)?;

        Ok(OAuthIdentity {
            id: OAuthIdentityId::from_uuid(row.id),
            user_id: UserId::from_uuid(row.user_id),
            provider,
            provider_user_id: row.provider_user_id,
            provider_email: row.provider_email,
            access_token_enc: row.access_token_enc,
            refresh_token_enc: row.refresh_token_enc,
            created_at: row.created_at,
        })
    }
}

fn parse_oauth_provider(s: &str) -> Result<OAuthProvider, AppError> {
    match s {
        "google" => Ok(OAuthProvider::Google),
        "apple" => Ok(OAuthProvider::Apple),
        "oidc" => Ok(OAuthProvider::Oidc),
        other => Err(AppError::Domain(DomainError::InvariantViolation {
            message: format!("invalid OAuth provider: {other}"),
        })),
    }
}

fn oauth_provider_to_str(provider: OAuthProvider) -> &'static str {
    match provider {
        OAuthProvider::Google => "google",
        OAuthProvider::Apple => "apple",
        OAuthProvider::Oidc => "oidc",
    }
}

fn map_sqlx_error(err: sqlx::Error) -> AppError {
    super::map_sqlx_error("oauth_identity", "OAuth identity already linked", err)
}

#[async_trait::async_trait]
impl OAuthIdentityRepository for PgOAuthIdentityRepository {
    async fn create(&self, identity: OAuthIdentity) -> Result<OAuthIdentity, AppError> {
        let row = sqlx::query_as!(
            OAuthIdentityRow,
            "INSERT INTO oauth_identities (id, user_id, provider, provider_user_id, \
             provider_email, access_token_enc, refresh_token_enc, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
             RETURNING id, user_id, provider, provider_user_id, \
             provider_email, access_token_enc, refresh_token_enc, created_at",
            identity.id.into_uuid(),
            identity.user_id.into_uuid(),
            oauth_provider_to_str(identity.provider),
            identity.provider_user_id,
            identity.provider_email.as_deref(),
            identity.access_token_enc.as_deref(),
            identity.refresh_token_enc.as_deref(),
            identity.created_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        OAuthIdentity::try_from(row)
    }

    async fn find_by_provider_user_id(
        &self,
        provider: OAuthProvider,
        provider_user_id: &str,
    ) -> Result<Option<OAuthIdentity>, AppError> {
        let row = sqlx::query_as!(
            OAuthIdentityRow,
            "SELECT id, user_id, provider, provider_user_id, \
             provider_email, access_token_enc, refresh_token_enc, created_at \
             FROM oauth_identities \
             WHERE provider = $1 AND provider_user_id = $2",
            oauth_provider_to_str(provider),
            provider_user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.map(OAuthIdentity::try_from).transpose()
    }

    async fn find_by_id(&self, id: OAuthIdentityId) -> Result<Option<OAuthIdentity>, AppError> {
        let row = sqlx::query_as!(
            OAuthIdentityRow,
            "SELECT id, user_id, provider, provider_user_id, \
             provider_email, access_token_enc, refresh_token_enc, created_at \
             FROM oauth_identities WHERE id = $1",
            id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.map(OAuthIdentity::try_from).transpose()
    }

    async fn find_by_user_and_provider(
        &self,
        user_id: UserId,
        provider: OAuthProvider,
    ) -> Result<Option<OAuthIdentity>, AppError> {
        let row = sqlx::query_as!(
            OAuthIdentityRow,
            "SELECT id, user_id, provider, provider_user_id, \
             provider_email, access_token_enc, refresh_token_enc, created_at \
             FROM oauth_identities \
             WHERE user_id = $1 AND provider = $2",
            user_id.into_uuid(),
            oauth_provider_to_str(provider),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.map(OAuthIdentity::try_from).transpose()
    }

    async fn list_by_user(&self, user_id: UserId) -> Result<Vec<OAuthIdentity>, AppError> {
        let rows = sqlx::query_as!(
            OAuthIdentityRow,
            "SELECT id, user_id, provider, provider_user_id, \
             provider_email, access_token_enc, refresh_token_enc, created_at \
             FROM oauth_identities WHERE user_id = $1 \
             ORDER BY created_at ASC",
            user_id.into_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(OAuthIdentity::try_from).collect()
    }

    async fn count_by_user_id(&self, user_id: UserId) -> Result<u64, AppError> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) AS \"count!\" FROM oauth_identities WHERE user_id = $1",
            user_id.into_uuid(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(count as u64)
    }

    async fn update_tokens(
        &self,
        id: OAuthIdentityId,
        access_token: Option<Vec<u8>>,
        refresh_token: Option<Vec<u8>>,
    ) -> Result<(), AppError> {
        let result = sqlx::query!(
            "UPDATE oauth_identities \
             SET access_token_enc = $2, refresh_token_enc = $3 \
             WHERE id = $1",
            id.into_uuid(),
            access_token.as_deref(),
            refresh_token.as_deref(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(AppError::Domain(DomainError::NotFound {
                entity: "oauth_identity",
                id: id.to_string(),
            }));
        }

        Ok(())
    }

    async fn delete(&self, id: OAuthIdentityId) -> Result<(), AppError> {
        let result = sqlx::query!("DELETE FROM oauth_identities WHERE id = $1", id.into_uuid())
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(AppError::Domain(DomainError::NotFound {
                entity: "oauth_identity",
                id: id.to_string(),
            }));
        }

        Ok(())
    }
}
