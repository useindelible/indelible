use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::authorization_code::AuthorizationCodeRepository;
use ind_domain::{AuthorizationCode, AuthorizationCodeId, ClientType, DomainError, UserId};

pub struct PgAuthorizationCodeRepository {
    pool: PgPool,
}

impl PgAuthorizationCodeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct AuthorizationCodeRow {
    id: Uuid,
    user_id: Uuid,
    code_hash: String,
    code_challenge: String,
    code_challenge_method: String,
    client_type: String,
    redirect_uri: String,
    scopes: Vec<String>,
    used_at: Option<DateTime<Utc>>,
    expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

impl TryFrom<AuthorizationCodeRow> for AuthorizationCode {
    type Error = AppError;

    fn try_from(row: AuthorizationCodeRow) -> Result<Self, Self::Error> {
        let client_type = parse_client_type(&row.client_type)?;

        Ok(AuthorizationCode {
            id: AuthorizationCodeId::from_uuid(row.id),
            user_id: UserId::from_uuid(row.user_id),
            code_hash: row.code_hash,
            code_challenge: row.code_challenge,
            code_challenge_method: row.code_challenge_method,
            client_type,
            redirect_uri: row.redirect_uri,
            scopes: row.scopes,
            used_at: row.used_at,
            expires_at: row.expires_at,
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
    super::map_sqlx_error("authorization_code", "duplicate authorization code", err)
}

#[async_trait::async_trait]
impl AuthorizationCodeRepository for PgAuthorizationCodeRepository {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn create(&self, code: AuthorizationCode) -> Result<AuthorizationCode, AppError> {
        let row = sqlx::query_as!(
            AuthorizationCodeRow,
            "INSERT INTO authorization_codes (id, user_id, code_hash, code_challenge, \
             code_challenge_method, client_type, redirect_uri, scopes, used_at, \
             expires_at, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) \
             RETURNING id, user_id, code_hash, code_challenge, code_challenge_method, \
             client_type, redirect_uri, scopes, used_at, expires_at, created_at",
            code.id.into_uuid(),
            code.user_id.into_uuid(),
            code.code_hash,
            code.code_challenge,
            code.code_challenge_method,
            client_type_to_str(code.client_type),
            code.redirect_uri,
            &code.scopes,
            code.used_at,
            code.expires_at,
            code.created_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        AuthorizationCode::try_from(row)
    }

    async fn find_by_code_hash(
        &self,
        code_hash: &str,
    ) -> Result<Option<AuthorizationCode>, AppError> {
        let row = sqlx::query_as!(
            AuthorizationCodeRow,
            "SELECT id, user_id, code_hash, code_challenge, code_challenge_method, \
             client_type, redirect_uri, scopes, used_at, expires_at, created_at \
             FROM authorization_codes WHERE code_hash = $1",
            code_hash
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.map(AuthorizationCode::try_from).transpose()
    }

    async fn consume_by_code_hash(
        &self,
        code_hash: &str,
    ) -> Result<Option<AuthorizationCode>, AppError> {
        let row = sqlx::query_as!(
            AuthorizationCodeRow,
            "UPDATE authorization_codes SET used_at = now() \
             WHERE code_hash = $1 AND used_at IS NULL \
             RETURNING id, user_id, code_hash, code_challenge, code_challenge_method, \
             client_type, redirect_uri, scopes, used_at, expires_at, created_at",
            code_hash,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.map(AuthorizationCode::try_from).transpose()
    }

    async fn mark_used(&self, id: AuthorizationCodeId) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE authorization_codes SET used_at = now() WHERE id = $1",
            id.into_uuid()
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn delete_expired(&self) -> Result<u64, AppError> {
        let result = sqlx::query!("DELETE FROM authorization_codes WHERE expires_at < now()")
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

        Ok(result.rows_affected())
    }
}
