use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::user::UserRepository;
use ind_domain::{DomainError, OAuthIdentity, OAuthProvider, Theme, User, UserId, UserStatus};

/// Transaction-scoped advisory lock key serializing first-run user creation ("IND_USR").
const BOOTSTRAP_USER_LOCK_KEY: i64 = 0x494E44_5F555352;

pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

struct UserRow {
    id: Uuid,
    email: String,
    password_hash: Option<String>,
    display_name: String,
    avatar_url: Option<String>,
    locale: Option<String>,
    timezone: String,
    theme: String,
    email_verified: bool,
    onboarding_completed: bool,
    onboarding_step: i16,
    email_token: String,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<UserRow> for User {
    type Error = AppError;

    fn try_from(row: UserRow) -> Result<Self, Self::Error> {
        let status = parse_user_status(&row.status)?;
        let theme = parse_theme(&row.theme)?;

        Ok(User {
            id: UserId::from_uuid(row.id),
            email: row.email,
            password_hash: row.password_hash,
            display_name: row.display_name,
            avatar_url: row.avatar_url,
            locale: row.locale,
            timezone: row.timezone,
            theme,
            email_verified: row.email_verified,
            onboarding_completed: row.onboarding_completed,
            onboarding_step: row.onboarding_step,
            email_token: row.email_token,
            status,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

fn parse_user_status(s: &str) -> Result<UserStatus, AppError> {
    match s {
        "active" => Ok(UserStatus::Active),
        "deactivated" => Ok(UserStatus::Deactivated),
        "deleted" => Ok(UserStatus::Deleted),
        other => Err(AppError::Domain(DomainError::InvariantViolation {
            message: format!("invalid user status: {other}"),
        })),
    }
}

fn parse_theme(s: &str) -> Result<Theme, AppError> {
    s.parse::<Theme>().map_err(|_| {
        AppError::Domain(DomainError::InvariantViolation {
            message: format!("invalid theme: {s}"),
        })
    })
}

fn status_to_str(status: UserStatus) -> &'static str {
    match status {
        UserStatus::Active => "active",
        UserStatus::Deactivated => "deactivated",
        UserStatus::Deleted => "deleted",
    }
}

fn theme_to_str(theme: Theme) -> &'static str {
    theme.as_str()
}

fn oauth_provider_to_str(provider: OAuthProvider) -> &'static str {
    match provider {
        OAuthProvider::Google => "google",
        OAuthProvider::Apple => "apple",
        OAuthProvider::Oidc => "oidc",
    }
}

fn map_sqlx_error(err: sqlx::Error) -> AppError {
    super::map_sqlx_error("user", "email already exists", err)
}

fn map_oauth_identity_sqlx_error(err: sqlx::Error) -> AppError {
    super::map_sqlx_error("oauth_identity", "OAuth identity already linked", err)
}

#[async_trait::async_trait]
impl UserRepository for PgUserRepository {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, AppError> {
        let row = sqlx::query_as!(
            UserRow,
            "SELECT id, email, password_hash, display_name, avatar_url, locale, timezone, theme, \
             email_verified, onboarding_completed, onboarding_step, email_token, status, \
             created_at, updated_at FROM users WHERE id = $1 AND status != 'deleted'",
            id.into_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.map(User::try_from).transpose()
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let normalized = User::normalize_email(email);
        let row = sqlx::query_as!(
            UserRow,
            "SELECT id, email, password_hash, display_name, avatar_url, locale, timezone, theme, \
             email_verified, onboarding_completed, onboarding_step, email_token, status, \
             created_at, updated_at FROM users WHERE email = $1 AND status != 'deleted'",
            normalized
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.map(User::try_from).transpose()
    }

    async fn find_by_email_token(&self, token: &str) -> Result<Option<User>, AppError> {
        let row = sqlx::query_as!(
            UserRow,
            "SELECT id, email, password_hash, display_name, avatar_url, locale, timezone, theme, \
             email_verified, onboarding_completed, onboarding_step, email_token, status, \
             created_at, updated_at FROM users WHERE email_token = $1 AND status != 'deleted'",
            token
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.map(User::try_from).transpose()
    }

    async fn create(&self, user: User) -> Result<User, AppError> {
        let normalized_email = User::normalize_email(&user.email);
        let row = sqlx::query_as!(
            UserRow,
            "INSERT INTO users (id, email, password_hash, display_name, avatar_url, \
             locale, timezone, theme, email_verified, onboarding_completed, \
             onboarding_step, email_token, status, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15) \
             RETURNING id, email, password_hash, display_name, avatar_url, locale, timezone, \
             theme, email_verified, onboarding_completed, onboarding_step, email_token, status, \
             created_at, updated_at",
            user.id.into_uuid(),
            normalized_email,
            user.password_hash.as_deref(),
            user.display_name,
            user.avatar_url.as_deref(),
            user.locale,
            user.timezone,
            theme_to_str(user.theme),
            user.email_verified,
            user.onboarding_completed,
            user.onboarding_step,
            user.email_token,
            status_to_str(user.status),
            user.created_at,
            user.updated_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        User::try_from(row)
    }

    async fn has_any_users(&self) -> Result<bool, AppError> {
        let exists =
            sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM users WHERE status != 'deleted')")
                .fetch_one(&self.pool)
                .await
                .map_err(map_sqlx_error)?;
        Ok(exists.unwrap_or(false))
    }

    async fn create_first_user(&self, user: User) -> Result<Option<User>, AppError> {
        let normalized_email = User::normalize_email(&user.email);
        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;

        sqlx::query!("SELECT pg_advisory_xact_lock($1)", BOOTSTRAP_USER_LOCK_KEY)
            .execute(&mut *tx)
            .await
            .map_err(map_sqlx_error)?;

        let exists =
            sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM users WHERE status != 'deleted')")
                .fetch_one(&mut *tx)
                .await
                .map_err(map_sqlx_error)?
                .unwrap_or(false);
        if exists {
            tx.rollback().await.map_err(map_sqlx_error)?;
            return Ok(None);
        }

        let row = sqlx::query_as!(
            UserRow,
            "INSERT INTO users (id, email, password_hash, display_name, avatar_url, \
             locale, timezone, theme, email_verified, onboarding_completed, \
             onboarding_step, email_token, status, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15) \
             RETURNING id, email, password_hash, display_name, avatar_url, locale, timezone, \
             theme, email_verified, onboarding_completed, onboarding_step, email_token, status, \
             created_at, updated_at",
            user.id.into_uuid(),
            normalized_email,
            user.password_hash.as_deref(),
            user.display_name,
            user.avatar_url.as_deref(),
            user.locale,
            user.timezone,
            theme_to_str(user.theme),
            user.email_verified,
            user.onboarding_completed,
            user.onboarding_step,
            user.email_token,
            status_to_str(user.status),
            user.created_at,
            user.updated_at,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        tx.commit().await.map_err(map_sqlx_error)?;

        Ok(Some(User::try_from(row)?))
    }

    async fn create_first_user_with_oauth_identity(
        &self,
        user: User,
        identity: OAuthIdentity,
    ) -> Result<Option<User>, AppError> {
        let normalized_email = User::normalize_email(&user.email);
        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;

        sqlx::query!("SELECT pg_advisory_xact_lock($1)", BOOTSTRAP_USER_LOCK_KEY)
            .execute(&mut *tx)
            .await
            .map_err(map_sqlx_error)?;

        let exists =
            sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM users WHERE status != 'deleted')")
                .fetch_one(&mut *tx)
                .await
                .map_err(map_sqlx_error)?
                .unwrap_or(false);
        if exists {
            tx.rollback().await.map_err(map_sqlx_error)?;
            return Ok(None);
        }

        let row = sqlx::query_as!(
            UserRow,
            "INSERT INTO users (id, email, password_hash, display_name, avatar_url, \
             locale, timezone, theme, email_verified, onboarding_completed, \
             onboarding_step, email_token, status, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15) \
             RETURNING id, email, password_hash, display_name, avatar_url, locale, timezone, \
             theme, email_verified, onboarding_completed, onboarding_step, email_token, status, \
             created_at, updated_at",
            user.id.into_uuid(),
            normalized_email,
            user.password_hash.as_deref(),
            user.display_name,
            user.avatar_url.as_deref(),
            user.locale,
            user.timezone,
            theme_to_str(user.theme),
            user.email_verified,
            user.onboarding_completed,
            user.onboarding_step,
            user.email_token,
            status_to_str(user.status),
            user.created_at,
            user.updated_at,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        sqlx::query!(
            "INSERT INTO oauth_identities (id, user_id, provider, provider_user_id, \
             provider_email, access_token_enc, refresh_token_enc, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            identity.id.into_uuid(),
            identity.user_id.into_uuid(),
            oauth_provider_to_str(identity.provider),
            identity.provider_user_id,
            identity.provider_email.as_deref(),
            identity.access_token_enc.as_deref(),
            identity.refresh_token_enc.as_deref(),
            identity.created_at,
        )
        .execute(&mut *tx)
        .await
        .map_err(map_oauth_identity_sqlx_error)?;

        tx.commit().await.map_err(map_sqlx_error)?;

        Ok(Some(User::try_from(row)?))
    }

    async fn update_profile_fields(
        &self,
        id: UserId,
        display_name: String,
        avatar_url: Option<String>,
        locale: Option<String>,
        timezone: String,
        theme: Theme,
    ) -> Result<User, AppError> {
        let row = sqlx::query_as!(
            UserRow,
            "UPDATE users SET display_name = $2, avatar_url = $3, locale = $4, \
             timezone = $5, theme = $6, updated_at = now() \
             WHERE id = $1 AND status != 'deleted' \
             RETURNING id, email, password_hash, display_name, avatar_url, locale, timezone, \
             theme, email_verified, onboarding_completed, onboarding_step, email_token, status, \
             created_at, updated_at",
            id.into_uuid(),
            display_name,
            avatar_url.as_deref(),
            locale.as_deref(),
            timezone,
            theme_to_str(theme),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "user",
                id: id.to_string(),
            })
        })?;

        User::try_from(row)
    }

    async fn update_onboarding(
        &self,
        id: UserId,
        onboarding_step: i16,
        onboarding_completed: bool,
    ) -> Result<User, AppError> {
        let row = sqlx::query_as!(
            UserRow,
            "UPDATE users SET onboarding_step = $2, onboarding_completed = $3, \
             updated_at = now() \
             WHERE id = $1 AND status != 'deleted' \
             RETURNING id, email, password_hash, display_name, avatar_url, locale, timezone, \
             theme, email_verified, onboarding_completed, onboarding_step, email_token, status, \
             created_at, updated_at",
            id.into_uuid(),
            onboarding_step,
            onboarding_completed,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "user",
                id: id.to_string(),
            })
        })?;

        User::try_from(row)
    }

    async fn update_password_hash(
        &self,
        id: UserId,
        password_hash: String,
    ) -> Result<User, AppError> {
        let row = sqlx::query_as!(
            UserRow,
            "UPDATE users SET password_hash = $2, updated_at = now() \
             WHERE id = $1 AND status != 'deleted' \
             RETURNING id, email, password_hash, display_name, avatar_url, locale, timezone, \
             theme, email_verified, onboarding_completed, onboarding_step, email_token, status, \
             created_at, updated_at",
            id.into_uuid(),
            password_hash,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "user",
                id: id.to_string(),
            })
        })?;

        User::try_from(row)
    }

    async fn update_email_verified(
        &self,
        id: UserId,
        email_verified: bool,
    ) -> Result<User, AppError> {
        let row = sqlx::query_as!(
            UserRow,
            "UPDATE users SET email_verified = $2, updated_at = now() \
             WHERE id = $1 AND status != 'deleted' \
             RETURNING id, email, password_hash, display_name, avatar_url, locale, timezone, \
             theme, email_verified, onboarding_completed, onboarding_step, email_token, status, \
             created_at, updated_at",
            id.into_uuid(),
            email_verified,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "user",
                id: id.to_string(),
            })
        })?;

        User::try_from(row)
    }

    async fn update_email_and_verification(
        &self,
        id: UserId,
        email: String,
        email_verified: bool,
    ) -> Result<User, AppError> {
        let normalized = User::normalize_email(&email);
        let row = sqlx::query_as!(
            UserRow,
            "UPDATE users SET email = $2, email_verified = $3, updated_at = now() \
             WHERE id = $1 AND status != 'deleted' \
             RETURNING id, email, password_hash, display_name, avatar_url, locale, timezone, \
             theme, email_verified, onboarding_completed, onboarding_step, email_token, status, \
             created_at, updated_at",
            id.into_uuid(),
            normalized,
            email_verified,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "user",
                id: id.to_string(),
            })
        })?;

        User::try_from(row)
    }

    async fn soft_delete(&self, id: UserId) -> Result<(), AppError> {
        let result = sqlx::query!(
            "UPDATE users SET status = 'deleted', updated_at = now() \
             WHERE id = $1 AND status != 'deleted'",
            id.into_uuid()
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(AppError::Domain(DomainError::NotFound {
                entity: "user",
                id: id.to_string(),
            }));
        }

        Ok(())
    }
}
