use chrono::{DateTime, Utc};
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::ai_output::AiOutputRepository;
use ind_application::repos::event::MutationSideEffects;
use ind_domain::{AiOutput, AiOutputId, AiOutputType, AiRunId, DocumentId};
use sqlx::PgPool;

pub struct PgAiOutputRepository {
    pool: PgPool,
}

impl PgAiOutputRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

struct AiOutputRow {
    id: Uuid,
    document_id: Uuid,
    output_type: String,
    content: serde_json::Value,
    ai_run_id: Option<Uuid>,
    created_at: DateTime<Utc>,
}

impl TryFrom<AiOutputRow> for AiOutput {
    type Error = AppError;

    fn try_from(row: AiOutputRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: AiOutputId::from_uuid(row.id),
            document_id: Some(DocumentId::from_uuid(row.document_id)),
            output_type: parse_output_type(&row.output_type)?,
            content: row.content,
            ai_run_id: row.ai_run_id.map(AiRunId::from_uuid),
            created_at: row.created_at,
        })
    }
}

#[async_trait::async_trait]
impl AiOutputRepository for PgAiOutputRepository {
    async fn upsert(
        &self,
        output: &AiOutput,
        effects: MutationSideEffects,
    ) -> Result<AiOutput, AppError> {
        let document_id = output.document_id.ok_or_else(|| {
            AppError::Domain(ind_domain::DomainError::InvariantViolation {
                message: "ai output requires document_id".to_string(),
            })
        })?;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;
        let row = sqlx::query_as!(
            AiOutputRow,
            r#"
            INSERT INTO ai_outputs (
                id, document_id, output_type, content, ai_run_id, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (document_id, output_type) WHERE document_id IS NOT NULL
            DO UPDATE SET
                content = EXCLUDED.content,
                ai_run_id = EXCLUDED.ai_run_id,
                created_at = EXCLUDED.created_at
            RETURNING id, document_id AS "document_id!", output_type, content, ai_run_id, created_at
            "#,
            output.id.into_uuid(),
            document_id.into_uuid(),
            format_output_type(output.output_type),
            output.content,
            output.ai_run_id.map(|id| id.into_uuid()),
            output.created_at,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;
        crate::repos::write_helpers::apply_mutation_side_effects_tx(&mut tx, effects).await?;
        tx.commit()
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;
        row.try_into()
    }

    async fn list_for_document(
        &self,
        document_id: DocumentId,
        output_type: Option<AiOutputType>,
    ) -> Result<Vec<AiOutput>, AppError> {
        let rows = sqlx::query_as!(
            AiOutputRow,
            r#"
            SELECT id, document_id AS "document_id!", output_type, content, ai_run_id, created_at
            FROM ai_outputs
            WHERE document_id = $1
              AND ($2::text IS NULL OR output_type = $2)
            ORDER BY created_at DESC, id DESC
            "#,
            document_id.into_uuid(),
            output_type.map(format_output_type) as Option<&str>,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;

        rows.into_iter().map(TryInto::try_into).collect()
    }

    async fn list_for_documents(
        &self,
        document_ids: &[DocumentId],
        output_type: Option<AiOutputType>,
    ) -> Result<Vec<AiOutput>, AppError> {
        if document_ids.is_empty() {
            return Ok(Vec::new());
        }

        let ids: Vec<_> = document_ids.iter().map(|id| (*id).into_uuid()).collect();
        let rows = sqlx::query_as!(
            AiOutputRow,
            r#"
            SELECT id, document_id AS "document_id!", output_type, content, ai_run_id, created_at
            FROM ai_outputs
            WHERE document_id = ANY($1)
              AND ($2::text IS NULL OR output_type = $2)
            ORDER BY document_id ASC, created_at DESC, id DESC
            "#,
            &ids,
            output_type.map(format_output_type) as Option<&str>,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;

        rows.into_iter().map(TryInto::try_into).collect()
    }

    async fn delete_by_document_and_type(
        &self,
        document_id: DocumentId,
        output_type: AiOutputType,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "DELETE FROM ai_outputs WHERE document_id = $1 AND output_type = $2",
            document_id.into_uuid(),
            format_output_type(output_type),
        )
        .execute(&self.pool)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;
        Ok(())
    }
}

fn format_output_type(value: AiOutputType) -> &'static str {
    match value {
        AiOutputType::Summary => "summary",
        AiOutputType::Tags => "tags",
        AiOutputType::Entities => "entities",
    }
}

fn parse_output_type(value: &str) -> Result<AiOutputType, AppError> {
    match value {
        "summary" => Ok(AiOutputType::Summary),
        "tags" => Ok(AiOutputType::Tags),
        "entities" => Ok(AiOutputType::Entities),
        other => Err(AppError::Domain(
            ind_domain::DomainError::InvariantViolation {
                message: format!("unknown ai output type: {other}"),
            },
        )),
    }
}
