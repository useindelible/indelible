use chrono::{DateTime, Utc};
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::ai_run::AiRunRepository;
use ind_application::repos::event::MutationSideEffects;
use ind_domain::{AiPromptAction, AiRun, AiRunId, DocumentId, UserId};
use sqlx::PgPool;

pub struct PgAiRunRepository {
    pool: PgPool,
}

impl PgAiRunRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct AiRunRow {
    id: Uuid,
    user_id: Uuid,
    document_id: Option<Uuid>,
    action: String,
    provider: String,
    model: String,
    input_tokens: Option<i32>,
    output_tokens: Option<i32>,
    is_byok: bool,
    status: String,
    error_message: Option<String>,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
}

impl TryFrom<AiRunRow> for AiRun {
    type Error = AppError;

    fn try_from(row: AiRunRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: AiRunId::from_uuid(row.id),
            user_id: UserId::from_uuid(row.user_id),
            document_id: row.document_id.map(DocumentId::from_uuid),
            action: parse_prompt_action(&row.action)?,
            provider: row.provider,
            model: row.model,
            input_tokens: row.input_tokens,
            output_tokens: row.output_tokens,
            is_byok: row.is_byok,
            status: row.status,
            error_message: row.error_message,
            started_at: row.started_at,
            completed_at: row.completed_at,
        })
    }
}

#[async_trait::async_trait]
impl AiRunRepository for PgAiRunRepository {
    async fn create(&self, run: &AiRun) -> Result<AiRun, AppError> {
        let document_id = run.document_id;
        let row = sqlx::query_as!(
            AiRunRow,
            r#"
            INSERT INTO ai_runs (
                id, user_id, document_id, action, provider, model, input_tokens,
                output_tokens, is_byok, status, error_message, started_at, completed_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING
                id, user_id, document_id, action, provider, model, input_tokens,
                output_tokens, is_byok, status, error_message, started_at, completed_at
            "#,
            run.id.into_uuid(),
            run.user_id.into_uuid(),
            document_id.map(|id| id.into_uuid()),
            format_prompt_action(run.action),
            &run.provider,
            &run.model,
            run.input_tokens,
            run.output_tokens,
            run.is_byok,
            &run.status,
            run.error_message.as_deref(),
            run.started_at,
            run.completed_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;

        row.try_into()
    }

    async fn mark_completed(
        &self,
        run_id: AiRunId,
        input_tokens: Option<i32>,
        output_tokens: Option<i32>,
        completed_at: DateTime<Utc>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE ai_runs
            SET
                input_tokens = $2,
                output_tokens = $3,
                status = 'completed',
                error_message = NULL,
                completed_at = $4
            WHERE id = $1
            "#,
            run_id.into_uuid(),
            input_tokens,
            output_tokens,
            completed_at,
        )
        .execute(&self.pool)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;
        Ok(())
    }

    async fn mark_failed(
        &self,
        run_id: AiRunId,
        error_message: String,
        effects: MutationSideEffects,
        completed_at: DateTime<Utc>,
    ) -> Result<(), AppError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;
        sqlx::query!(
            r#"
            UPDATE ai_runs
            SET
                status = 'failed',
                error_message = $2,
                completed_at = $3
            WHERE id = $1
            "#,
            run_id.into_uuid(),
            error_message,
            completed_at,
        )
        .execute(&mut *tx)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;
        crate::repos::write_helpers::apply_mutation_side_effects_tx(&mut tx, effects).await?;
        tx.commit()
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;
        Ok(())
    }
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
        other => Err(AppError::Domain(
            ind_domain::DomainError::InvariantViolation {
                message: format!("unknown ai prompt action: {other}"),
            },
        )),
    }
}
