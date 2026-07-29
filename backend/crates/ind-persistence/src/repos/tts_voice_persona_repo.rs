use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::tts_voice_persona::TtsVoicePersonaRepository;
use ind_domain::{
    DomainError, TtsPersonaStatus, TtsProvider, TtsVoicePersona, TtsVoicePersonaId, UserId,
};

pub struct PgTtsVoicePersonaRepository {
    pool: PgPool,
}

impl PgTtsVoicePersonaRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

struct TtsVoicePersonaRow {
    id: Uuid,
    user_id: Option<Uuid>,
    display_name: String,
    description: Option<String>,
    provider: String,
    provider_voice_id: Option<String>,
    provider_model: Option<String>,
    design_prompt: Option<String>,
    style_prompt: Option<String>,
    pace: Option<String>,
    energy: Option<String>,
    warmth: Option<String>,
    formality: Option<String>,
    pronunciation_prefs: serde_json::Value,
    status: String,
    is_builtin: bool,
    prompt_hash: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<TtsVoicePersonaRow> for TtsVoicePersona {
    type Error = AppError;

    fn try_from(row: TtsVoicePersonaRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: TtsVoicePersonaId::from_uuid(row.id),
            user_id: row.user_id.map(UserId::from_uuid),
            display_name: row.display_name,
            description: row.description,
            provider: TtsProvider::parse(&row.provider).ok_or_else(|| {
                AppError::Repository(format!("unknown tts provider: {}", row.provider).into())
            })?,
            provider_voice_id: row.provider_voice_id,
            provider_model: row.provider_model,
            design_prompt: row.design_prompt,
            style_prompt: row.style_prompt,
            pace: row.pace,
            energy: row.energy,
            warmth: row.warmth,
            formality: row.formality,
            pronunciation_prefs: row.pronunciation_prefs,
            status: TtsPersonaStatus::parse(&row.status).ok_or_else(|| {
                AppError::Repository(format!("unknown persona status: {}", row.status).into())
            })?,
            is_builtin: row.is_builtin,
            prompt_hash: row.prompt_hash,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

fn map_err(err: sqlx::Error) -> AppError {
    super::map_sqlx_error("tts_voice_persona", "voice persona already exists", err)
}

#[async_trait::async_trait]
impl TtsVoicePersonaRepository for PgTtsVoicePersonaRepository {
    async fn list_for_user(&self, user_id: UserId) -> Result<Vec<TtsVoicePersona>, AppError> {
        let rows = sqlx::query_as!(
            TtsVoicePersonaRow,
            r#"
            SELECT
                id,
                user_id,
                display_name,
                description,
                provider,
                provider_voice_id,
                provider_model,
                design_prompt,
                style_prompt,
                pace,
                energy,
                warmth,
                formality,
                pronunciation_prefs,
                status,
                is_builtin,
                prompt_hash,
                created_at,
                updated_at
            FROM tts_voice_personas
            WHERE user_id = $1 OR (is_builtin = true AND user_id IS NULL)
            ORDER BY is_builtin DESC, display_name ASC
            "#,
            user_id.into_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_err)?;

        rows.into_iter().map(TtsVoicePersona::try_from).collect()
    }

    async fn get(
        &self,
        id: TtsVoicePersonaId,
        user_id: UserId,
    ) -> Result<Option<TtsVoicePersona>, AppError> {
        let row = sqlx::query_as!(
            TtsVoicePersonaRow,
            r#"
            SELECT
                id,
                user_id,
                display_name,
                description,
                provider,
                provider_voice_id,
                provider_model,
                design_prompt,
                style_prompt,
                pace,
                energy,
                warmth,
                formality,
                pronunciation_prefs,
                status,
                is_builtin,
                prompt_hash,
                created_at,
                updated_at
            FROM tts_voice_personas
            WHERE id = $1 AND (user_id = $2 OR (is_builtin = true AND user_id IS NULL))
            "#,
            id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_err)?;

        row.map(TtsVoicePersona::try_from).transpose()
    }

    async fn insert(&self, persona: &TtsVoicePersona) -> Result<TtsVoicePersona, AppError> {
        let row = sqlx::query_as!(
            TtsVoicePersonaRow,
            r#"
            INSERT INTO tts_voice_personas (
                id,
                user_id,
                display_name,
                description,
                provider,
                provider_voice_id,
                provider_model,
                design_prompt,
                style_prompt,
                pace,
                energy,
                warmth,
                formality,
                pronunciation_prefs,
                status,
                is_builtin,
                prompt_hash,
                created_at,
                updated_at
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15, $16, $17, $18, $19
            )
            RETURNING
                id,
                user_id,
                display_name,
                description,
                provider,
                provider_voice_id,
                provider_model,
                design_prompt,
                style_prompt,
                pace,
                energy,
                warmth,
                formality,
                pronunciation_prefs,
                status,
                is_builtin,
                prompt_hash,
                created_at,
                updated_at
            "#,
            persona.id.into_uuid(),
            persona.user_id.map(|u| u.into_uuid()),
            persona.display_name,
            persona.description.as_deref(),
            persona.provider.as_str(),
            persona.provider_voice_id.as_deref(),
            persona.provider_model.as_deref(),
            persona.design_prompt.as_deref(),
            persona.style_prompt.as_deref(),
            persona.pace.as_deref(),
            persona.energy.as_deref(),
            persona.warmth.as_deref(),
            persona.formality.as_deref(),
            persona.pronunciation_prefs,
            persona.status.as_str(),
            persona.is_builtin,
            persona.prompt_hash,
            persona.created_at,
            persona.updated_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_err)?;

        TtsVoicePersona::try_from(row)
    }

    async fn update_fields(&self, persona: &TtsVoicePersona) -> Result<TtsVoicePersona, AppError> {
        let row = sqlx::query_as!(
            TtsVoicePersonaRow,
            r#"
            UPDATE tts_voice_personas SET
                display_name = $2,
                description = $3,
                provider_voice_id = $4,
                provider_model = $5,
                design_prompt = $6,
                style_prompt = $7,
                pace = $8,
                energy = $9,
                warmth = $10,
                formality = $11,
                pronunciation_prefs = $12,
                status = $13,
                prompt_hash = $14,
                updated_at = $15
            WHERE id = $1 AND user_id = $16
            RETURNING
                id,
                user_id,
                display_name,
                description,
                provider,
                provider_voice_id,
                provider_model,
                design_prompt,
                style_prompt,
                pace,
                energy,
                warmth,
                formality,
                pronunciation_prefs,
                status,
                is_builtin,
                prompt_hash,
                created_at,
                updated_at
            "#,
            persona.id.into_uuid(),
            persona.display_name,
            persona.description.as_deref(),
            persona.provider_voice_id.as_deref(),
            persona.provider_model.as_deref(),
            persona.design_prompt.as_deref(),
            persona.style_prompt.as_deref(),
            persona.pace.as_deref(),
            persona.energy.as_deref(),
            persona.warmth.as_deref(),
            persona.formality.as_deref(),
            persona.pronunciation_prefs,
            persona.status.as_str(),
            persona.prompt_hash,
            persona.updated_at,
            persona.user_id.map(|u| u.into_uuid()),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_err)?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "tts_voice_persona",
                id: persona.id.to_string(),
            })
        })?;

        TtsVoicePersona::try_from(row)
    }

    async fn delete(&self, id: TtsVoicePersonaId, user_id: UserId) -> Result<bool, AppError> {
        let result = sqlx::query!(
            "DELETE FROM tts_voice_personas WHERE id = $1 AND user_id = $2",
            id.into_uuid(),
            user_id.into_uuid(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(result.rows_affected() > 0)
    }
}
