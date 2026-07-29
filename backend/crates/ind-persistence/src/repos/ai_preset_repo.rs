use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::ai_preset::{AiPromptPresetRepository, UpdateAiPromptPresetInput};
use ind_domain::{AiPromptAction, AiPromptPreset, AiPromptPresetId, DomainError, UserId};

pub struct PgAiPromptPresetRepository {
    pool: PgPool,
}

impl PgAiPromptPresetRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct AiPromptPresetRow {
    id: Uuid,
    user_id: Option<Uuid>,
    name: String,
    action: String,
    system_prompt: String,
    is_default: bool,
    is_system: bool,
    created_at: DateTime<Utc>,
}

impl TryFrom<AiPromptPresetRow> for AiPromptPreset {
    type Error = AppError;

    fn try_from(row: AiPromptPresetRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: AiPromptPresetId::from_uuid(row.id),
            user_id: row.user_id.map(UserId::from_uuid),
            name: row.name,
            action: parse_prompt_action(&row.action)?,
            system_prompt: row.system_prompt,
            is_default: row.is_default,
            is_system: row.is_system,
            created_at: row.created_at,
        })
    }
}

#[async_trait::async_trait]
impl AiPromptPresetRepository for PgAiPromptPresetRepository {
    async fn list_by_user(&self, user_id: UserId) -> Result<Vec<AiPromptPreset>, AppError> {
        let rows = sqlx::query_as!(
            AiPromptPresetRow,
            r#"
            SELECT id, user_id, name, action, system_prompt, is_default, is_system, created_at
            FROM ai_prompt_presets
            WHERE user_id = $1 OR is_system = true
            ORDER BY is_system DESC, action ASC, is_default DESC, created_at ASC, id ASC
            "#,
            user_id.into_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(repository_error)?;

        rows.into_iter().map(TryInto::try_into).collect()
    }

    async fn find_by_id_for_user(
        &self,
        preset_id: AiPromptPresetId,
        user_id: UserId,
    ) -> Result<Option<AiPromptPreset>, AppError> {
        let row = sqlx::query_as!(
            AiPromptPresetRow,
            r#"
            SELECT id, user_id, name, action, system_prompt, is_default, is_system, created_at
            FROM ai_prompt_presets
            WHERE id = $1
              AND user_id = $2
            "#,
            preset_id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(repository_error)?;

        row.map(TryInto::try_into).transpose()
    }

    async fn create(&self, preset: &AiPromptPreset) -> Result<AiPromptPreset, AppError> {
        let mut tx = self.pool.begin().await.map_err(repository_error)?;

        if preset.is_default
            && let Some(user_id) = preset.user_id
        {
            clear_default_for_action(&mut tx, user_id, preset.action).await?;
        }

        let row = sqlx::query_as!(
            AiPromptPresetRow,
            r#"
            INSERT INTO ai_prompt_presets (
                id,
                user_id,
                name,
                action,
                system_prompt,
                is_default,
                is_system,
                created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, user_id, name, action, system_prompt, is_default, is_system, created_at
            "#,
            preset.id.into_uuid(),
            preset.user_id.map(|id| id.into_uuid()),
            &preset.name,
            format_prompt_action(preset.action),
            &preset.system_prompt,
            preset.is_default,
            preset.is_system,
            preset.created_at,
        )
        .fetch_one(tx.as_mut())
        .await
        .map_err(repository_error)?;

        tx.commit().await.map_err(repository_error)?;
        AiPromptPreset::try_from(row)
    }

    async fn update(
        &self,
        preset_id: AiPromptPresetId,
        user_id: UserId,
        input: UpdateAiPromptPresetInput,
    ) -> Result<AiPromptPreset, AppError> {
        let existing = self
            .find_by_id_for_user(preset_id, user_id)
            .await?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "ai_prompt_preset",
                    id: preset_id.to_string(),
                })
            })?;

        let mut tx = self.pool.begin().await.map_err(repository_error)?;

        if input.is_default == Some(true) {
            clear_default_for_action(&mut tx, user_id, existing.action).await?;
        }

        let row = sqlx::query_as!(
            AiPromptPresetRow,
            r#"
            UPDATE ai_prompt_presets
            SET name = $3,
                system_prompt = $4,
                is_default = $5
            WHERE id = $1
              AND user_id = $2
            RETURNING id, user_id, name, action, system_prompt, is_default, is_system, created_at
            "#,
            preset_id.into_uuid(),
            user_id.into_uuid(),
            input.name.as_deref().unwrap_or(existing.name.as_str()),
            input
                .system_prompt
                .as_deref()
                .unwrap_or(existing.system_prompt.as_str()),
            input.is_default.unwrap_or(existing.is_default),
        )
        .fetch_one(tx.as_mut())
        .await
        .map_err(repository_error)?;

        tx.commit().await.map_err(repository_error)?;
        AiPromptPreset::try_from(row)
    }

    async fn delete(&self, preset_id: AiPromptPresetId, user_id: UserId) -> Result<(), AppError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM ai_prompt_presets
            WHERE id = $1
              AND user_id = $2
            "#,
            preset_id.into_uuid(),
            user_id.into_uuid(),
        )
        .execute(&self.pool)
        .await
        .map_err(repository_error)?;

        if result.rows_affected() == 0 {
            return Err(AppError::Domain(DomainError::NotFound {
                entity: "ai_prompt_preset",
                id: preset_id.to_string(),
            }));
        }

        Ok(())
    }

    async fn find_default_for_action(
        &self,
        user_id: UserId,
        action: AiPromptAction,
    ) -> Result<Option<AiPromptPreset>, AppError> {
        let row = sqlx::query_as!(
            AiPromptPresetRow,
            r#"
            SELECT id, user_id, name, action, system_prompt, is_default, is_system, created_at
            FROM ai_prompt_presets
            WHERE user_id = $1
              AND action = $2
              AND is_default = true
              AND is_system = false
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            user_id.into_uuid(),
            format_prompt_action(action),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(repository_error)?;

        row.map(TryInto::try_into).transpose()
    }

    async fn find_system_preset_for_action(
        &self,
        action: AiPromptAction,
    ) -> Result<Option<AiPromptPreset>, AppError> {
        let row = sqlx::query_as!(
            AiPromptPresetRow,
            r#"
            SELECT id, user_id, name, action, system_prompt, is_default, is_system, created_at
            FROM ai_prompt_presets
            WHERE is_system = true
              AND action = $1
            LIMIT 1
            "#,
            format_prompt_action(action),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(repository_error)?;

        row.map(TryInto::try_into).transpose()
    }
}

async fn clear_default_for_action(
    tx: &mut Transaction<'_, Postgres>,
    user_id: UserId,
    action: AiPromptAction,
) -> Result<(), AppError> {
    sqlx::query!(
        r#"
        UPDATE ai_prompt_presets
        SET is_default = false
        WHERE user_id = $1
          AND action = $2
          AND is_default = true
        "#,
        user_id.into_uuid(),
        format_prompt_action(action),
    )
    .execute(tx.as_mut())
    .await
    .map_err(repository_error)?;
    Ok(())
}

fn repository_error(err: sqlx::Error) -> AppError {
    AppError::Repository(Box::new(err))
}

fn format_prompt_action(value: AiPromptAction) -> &'static str {
    match value {
        AiPromptAction::Summary => "summary",
        AiPromptAction::Tags => "tags",
        AiPromptAction::Entities => "entities",
        AiPromptAction::Chat => "chat",
        AiPromptAction::Custom => "custom",
    }
}

fn parse_prompt_action(value: &str) -> Result<AiPromptAction, AppError> {
    match value {
        "summary" => Ok(AiPromptAction::Summary),
        "tags" => Ok(AiPromptAction::Tags),
        "entities" => Ok(AiPromptAction::Entities),
        "chat" => Ok(AiPromptAction::Chat),
        "custom" => Ok(AiPromptAction::Custom),
        other => Err(AppError::Domain(DomainError::InvariantViolation {
            message: format!("unknown ai prompt action: {other}"),
        })),
    }
}
