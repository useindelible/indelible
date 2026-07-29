use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::email_alias::{CreateEmailAlias, EmailAliasRepository};
use ind_domain::{
    DomainError, EmailAlias, EmailAliasId, EmailAliasStatus, EmailDestination, UserId,
};

use super::map_sqlx_error;

pub struct PgEmailAliasRepository {
    pool: PgPool,
}

impl PgEmailAliasRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl EmailAliasRepository for PgEmailAliasRepository {
    async fn create(&self, input: CreateEmailAlias<'_>) -> Result<EmailAlias, AppError> {
        let new_id = Uuid::now_v7();
        let row = sqlx::query_as!(
            EmailAliasRow,
            r#"
            INSERT INTO email_aliases (id, user_id, destination, local_part, status, is_default, created_at)
            VALUES ($1, $2, $3, $4, 'active', $5, now())
            RETURNING id, user_id, destination, local_part, status, is_default,
                      created_at, retire_at, retired_at
            "#,
            new_id,
            input.user_id.into_uuid(),
            input.destination.as_str(),
            input.local_part,
            input.is_default,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| map_sqlx_error("email_alias", "alias already exists", err))?;

        row.try_into()
    }

    async fn create_with_default_rotation(
        &self,
        input: CreateEmailAlias<'_>,
        retire_grace_days: i64,
    ) -> Result<EmailAlias, AppError> {
        let mut tx = self.pool.begin().await.map_err(map_err)?;

        if input.is_default {
            let days = i32::try_from(retire_grace_days.max(1)).unwrap_or(i32::MAX);
            sqlx::query!(
                r#"
                UPDATE email_aliases
                SET retire_at = now() + make_interval(days => $3::int),
                    is_default = false
                WHERE user_id = $1
                  AND destination = $2
                  AND status = 'active'
                  AND is_default = true
                  AND retire_at IS NULL
                "#,
                input.user_id.into_uuid(),
                input.destination.as_str(),
                days,
            )
            .execute(&mut *tx)
            .await
            .map_err(map_err)?;
        }

        let new_id = Uuid::now_v7();
        let row = sqlx::query_as!(
            EmailAliasRow,
            r#"
            INSERT INTO email_aliases (id, user_id, destination, local_part, status, is_default, created_at)
            VALUES ($1, $2, $3, $4, 'active', $5, now())
            RETURNING id, user_id, destination, local_part, status, is_default,
                      created_at, retire_at, retired_at
            "#,
            new_id,
            input.user_id.into_uuid(),
            input.destination.as_str(),
            input.local_part,
            input.is_default,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|err| map_sqlx_error("email_alias", "alias already exists", err))?;

        tx.commit().await.map_err(map_err)?;
        row.try_into()
    }

    async fn list_for_user(&self, user_id: UserId) -> Result<Vec<EmailAlias>, AppError> {
        let rows = sqlx::query_as!(
            EmailAliasRow,
            r#"
            SELECT id, user_id, destination, local_part, status, is_default,
                   created_at, retire_at, retired_at
            FROM email_aliases
            WHERE user_id = $1
            ORDER BY created_at ASC
            "#,
            user_id.into_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_err)?;

        rows.into_iter().map(EmailAlias::try_from).collect()
    }

    async fn find_active(
        &self,
        destination: EmailDestination,
        local_part: &str,
    ) -> Result<Option<EmailAlias>, AppError> {
        // `retire_at` filter implements the 28-day grace window: an alias
        // stays receiving while retire_at > now(), and becomes invisible
        // once that timestamp passes. No background sweep needed — the
        // resolver simply ignores expired rows.
        let row = sqlx::query_as!(
            EmailAliasRow,
            r#"
            SELECT id, user_id, destination, local_part, status, is_default,
                   created_at, retire_at, retired_at
            FROM email_aliases
            WHERE destination = $1
              AND local_part = $2
              AND status = 'active'
              AND (retire_at IS NULL OR retire_at > now())
            "#,
            destination.as_str(),
            local_part,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_err)?;

        row.map(EmailAlias::try_from).transpose()
    }

    async fn find_by_id_and_user(
        &self,
        user_id: UserId,
        alias_id: EmailAliasId,
    ) -> Result<Option<EmailAlias>, AppError> {
        let row = sqlx::query_as!(
            EmailAliasRow,
            r#"
            SELECT id, user_id, destination, local_part, status, is_default,
                   created_at, retire_at, retired_at
            FROM email_aliases
            WHERE id = $1 AND user_id = $2
            "#,
            alias_id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_err)?;

        row.map(EmailAlias::try_from).transpose()
    }

    async fn find_active_default(
        &self,
        user_id: UserId,
        destination: EmailDestination,
    ) -> Result<Option<EmailAlias>, AppError> {
        let row = sqlx::query_as!(
            EmailAliasRow,
            r#"
            SELECT id, user_id, destination, local_part, status, is_default,
                   created_at, retire_at, retired_at
            FROM email_aliases
            WHERE user_id = $1
              AND destination = $2
              AND status = 'active'
              AND is_default = true
              AND retire_at IS NULL
            "#,
            user_id.into_uuid(),
            destination.as_str(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_err)?;

        row.map(EmailAlias::try_from).transpose()
    }

    async fn retire(&self, alias_id: EmailAliasId) -> Result<(), AppError> {
        sqlx::query!(
            r#"UPDATE email_aliases
               SET status = 'retired',
                   retired_at = now()
               WHERE id = $1"#,
            alias_id.into_uuid(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn mark_for_retire(
        &self,
        alias_id: EmailAliasId,
        grace_days: i64,
    ) -> Result<(), AppError> {
        // Clamp to >= 1 so a 0/negative caller cannot place retire_at in the
        // past, which `find_active` would already treat as expired. Also clear
        // is_default so the partial unique index permits a new default for the
        // same (user_id, destination) while this alias stays active and
        // receiving during the grace window.
        let days = i32::try_from(grace_days.max(1)).unwrap_or(i32::MAX);
        sqlx::query!(
            r#"UPDATE email_aliases
               SET retire_at = now() + make_interval(days => $2::int),
                   is_default = false
               WHERE id = $1
                 AND status = 'active'"#,
            alias_id.into_uuid(),
            days,
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;
        Ok(())
    }
}

struct EmailAliasRow {
    id: Uuid,
    user_id: Uuid,
    destination: String,
    local_part: String,
    status: String,
    is_default: bool,
    created_at: DateTime<Utc>,
    retire_at: Option<DateTime<Utc>>,
    retired_at: Option<DateTime<Utc>>,
}

impl TryFrom<EmailAliasRow> for EmailAlias {
    type Error = AppError;

    fn try_from(row: EmailAliasRow) -> Result<Self, Self::Error> {
        Ok(EmailAlias {
            id: EmailAliasId::from_uuid(row.id),
            user_id: UserId::from_uuid(row.user_id),
            destination: parse_destination(&row.destination)?,
            local_part: row.local_part,
            status: parse_status(&row.status)?,
            is_default: row.is_default,
            created_at: row.created_at,
            retire_at: row.retire_at,
            retired_at: row.retired_at,
        })
    }
}

fn parse_destination(s: &str) -> Result<EmailDestination, AppError> {
    match s {
        "feed" => Ok(EmailDestination::Feed),
        "library" => Ok(EmailDestination::Library),
        other => Err(AppError::Domain(DomainError::InvariantViolation {
            message: format!("invalid email_aliases.destination: {other}"),
        })),
    }
}

fn parse_status(s: &str) -> Result<EmailAliasStatus, AppError> {
    match s {
        "active" => Ok(EmailAliasStatus::Active),
        "retired" => Ok(EmailAliasStatus::Retired),
        other => Err(AppError::Domain(DomainError::InvariantViolation {
            message: format!("invalid email_aliases.status: {other}"),
        })),
    }
}

fn map_err(err: sqlx::Error) -> AppError {
    AppError::Repository(Box::new(err))
}
