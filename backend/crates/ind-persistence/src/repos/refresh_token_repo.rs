use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::refresh_token::RefreshTokenRepository;
use ind_domain::{ClientType, DomainError, RefreshToken, RefreshTokenId, UserId};

pub struct PgRefreshTokenRepository {
    pool: PgPool,
}

impl PgRefreshTokenRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct RefreshTokenRow {
    id: Uuid,
    family_id: Uuid,
    user_id: Uuid,
    token_hash: String,
    client_type: String,
    ip_address: Option<String>,
    user_agent: Option<String>,
    replaced_by: Option<Uuid>,
    revoked_at: Option<DateTime<Utc>>,
    expires_at: DateTime<Utc>,
    absolute_expires_at: DateTime<Utc>,
    last_used_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

impl TryFrom<RefreshTokenRow> for RefreshToken {
    type Error = AppError;

    fn try_from(row: RefreshTokenRow) -> Result<Self, Self::Error> {
        let client_type = parse_client_type(&row.client_type)?;

        let ip_address = row
            .ip_address
            .map(|ip| ip.strip_suffix("/32").unwrap_or(&ip).to_owned());

        Ok(RefreshToken {
            id: RefreshTokenId::from_uuid(row.id),
            family_id: row.family_id,
            user_id: UserId::from_uuid(row.user_id),
            token_hash: row.token_hash,
            client_type,
            ip_address,
            user_agent: row.user_agent,
            replaced_by: row.replaced_by.map(RefreshTokenId::from_uuid),
            revoked_at: row.revoked_at,
            expires_at: row.expires_at,
            absolute_expires_at: row.absolute_expires_at,
            last_used_at: row.last_used_at,
            created_at: row.created_at,
        })
    }
}

fn parse_client_type(s: &str) -> Result<ClientType, AppError> {
    match s {
        "web" => Ok(ClientType::Web),
        "ios" => Ok(ClientType::Ios),
        "android" => Ok(ClientType::Android),
        "desktop" => Ok(ClientType::Desktop),
        "extension" => Ok(ClientType::Extension),
        "cli" => Ok(ClientType::Cli),
        other => Err(AppError::Domain(DomainError::InvariantViolation {
            message: format!("invalid client type: {other}"),
        })),
    }
}

fn client_type_to_str(ct: ClientType) -> &'static str {
    match ct {
        ClientType::Web => "web",
        ClientType::Ios => "ios",
        ClientType::Android => "android",
        ClientType::Desktop => "desktop",
        ClientType::Extension => "extension",
        ClientType::Cli => "cli",
    }
}

fn map_sqlx_error(err: sqlx::Error) -> AppError {
    super::map_sqlx_error("refresh_token", "duplicate refresh token", err)
}

#[async_trait::async_trait]
impl RefreshTokenRepository for PgRefreshTokenRepository {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn create(&self, token: RefreshToken) -> Result<RefreshToken, AppError> {
        let row = sqlx::query_as!(
            RefreshTokenRow,
            "INSERT INTO refresh_tokens (id, family_id, user_id, token_hash, client_type, \
             ip_address, user_agent, replaced_by, revoked_at, expires_at, absolute_expires_at, \
             last_used_at, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6::text::inet, $7, $8, $9, $10, $11, $12, $13) \
             RETURNING id, family_id, user_id, token_hash, client_type, \
             ip_address::text, user_agent, replaced_by, revoked_at, expires_at, \
             absolute_expires_at, last_used_at, created_at",
            token.id.into_uuid(),
            token.family_id,
            token.user_id.into_uuid(),
            token.token_hash,
            client_type_to_str(token.client_type),
            token.ip_address.as_deref(),
            token.user_agent.as_deref(),
            token.replaced_by.map(|id| id.into_uuid()),
            token.revoked_at,
            token.expires_at,
            token.absolute_expires_at,
            token.last_used_at,
            token.created_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        RefreshToken::try_from(row)
    }

    async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<RefreshToken>, AppError> {
        let row = sqlx::query_as!(
            RefreshTokenRow,
            "SELECT id, family_id, user_id, token_hash, client_type, \
             ip_address::text, user_agent, replaced_by, revoked_at, expires_at, \
             absolute_expires_at, last_used_at, created_at \
             FROM refresh_tokens WHERE token_hash = $1",
            token_hash
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.map(RefreshToken::try_from).transpose()
    }

    async fn find_by_id(&self, id: RefreshTokenId) -> Result<Option<RefreshToken>, AppError> {
        let row = sqlx::query_as!(
            RefreshTokenRow,
            "SELECT id, family_id, user_id, token_hash, client_type, \
             ip_address::text, user_agent, replaced_by, revoked_at, expires_at, \
             absolute_expires_at, last_used_at, created_at \
             FROM refresh_tokens WHERE id = $1",
            id.into_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.map(RefreshToken::try_from).transpose()
    }

    async fn set_replaced_by(
        &self,
        id: RefreshTokenId,
        replaced_by: RefreshTokenId,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE refresh_tokens SET replaced_by = $2 WHERE id = $1",
            id.into_uuid(),
            replaced_by.into_uuid(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn revoke_family(&self, family_id: Uuid) -> Result<u64, AppError> {
        let result = sqlx::query!(
            "UPDATE refresh_tokens SET revoked_at = now() \
             WHERE family_id = $1 AND revoked_at IS NULL",
            family_id
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(result.rows_affected())
    }

    async fn revoke_all_for_user(&self, user_id: UserId) -> Result<u64, AppError> {
        let result = sqlx::query!(
            "UPDATE refresh_tokens SET revoked_at = now() \
             WHERE user_id = $1 AND revoked_at IS NULL",
            user_id.into_uuid()
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(result.rows_affected())
    }

    async fn list_active_families(&self, user_id: UserId) -> Result<Vec<RefreshToken>, AppError> {
        let rows = sqlx::query_as!(
            RefreshTokenRow,
            "SELECT id, family_id, user_id, token_hash, client_type, \
             ip_address::text, user_agent, replaced_by, revoked_at, expires_at, \
             absolute_expires_at, last_used_at, created_at \
             FROM refresh_tokens \
             WHERE user_id = $1 AND revoked_at IS NULL AND replaced_by IS NULL \
               AND expires_at > now() AND absolute_expires_at > now() \
             ORDER BY last_used_at DESC",
            user_id.into_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(RefreshToken::try_from).collect()
    }

    async fn update_last_used(&self, id: RefreshTokenId) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE refresh_tokens SET last_used_at = now() WHERE id = $1",
            id.into_uuid()
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn delete_expired(&self) -> Result<u64, AppError> {
        let result = sqlx::query!(
            "DELETE FROM refresh_tokens WHERE expires_at < now() OR absolute_expires_at < now()",
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(result.rows_affected())
    }
}
