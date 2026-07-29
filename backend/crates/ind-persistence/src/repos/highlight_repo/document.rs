//! Document-keyed highlight reads/writes.

use chrono::Utc;

use ind_application::AppError;
use ind_application::repos::event::MutationSideEffects;
use ind_domain::{DocumentId, Highlight, NewHighlight, UserId};

use super::super::write_helpers::apply_mutation_side_effects_tx;
use super::{HighlightRow, PgHighlightRepository, map_sqlx_error};

impl PgHighlightRepository {
    /// Insert a document-keyed highlight and commit effects in the same transaction.
    pub(super) async fn create_for_document_impl(
        &self,
        highlight: &NewHighlight,
        effects: MutationSideEffects,
    ) -> Result<Highlight, AppError> {
        let now = Utc::now();
        let locator_json = highlight
            .locator
            .as_ref()
            .map(serde_json::to_value)
            .transpose()
            .map_err(|e| AppError::Repository(Box::new(e)))?;
        let source_locator_json = highlight
            .source_locator
            .as_ref()
            .map(serde_json::to_value)
            .transpose()
            .map_err(|e| AppError::Repository(Box::new(e)))?;

        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;

        let row = sqlx::query_as!(
            HighlightRow,
            "INSERT INTO highlights (id, document_id, user_id, color, text_content, locator, source_locator, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8) \
             RETURNING id, document_id AS \"document_id!\", user_id, color, text_content, locator, source_locator, created_at, updated_at",
            highlight.id.into_uuid(),
            highlight.document_id.into_uuid(),
            highlight.user_id.into_uuid(),
            &highlight.color,
            &highlight.text_content,
            locator_json,
            source_locator_json,
            now,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        apply_mutation_side_effects_tx(&mut tx, effects).await?;

        tx.commit().await.map_err(map_sqlx_error)?;
        Highlight::try_from(row)
    }

    pub(super) async fn list_by_document_impl(
        &self,
        document_id: DocumentId,
        user_id: UserId,
    ) -> Result<Vec<Highlight>, AppError> {
        let rows = sqlx::query_as!(
            HighlightRow,
            "SELECT id, document_id AS \"document_id!\", user_id, color, text_content, locator, source_locator, created_at, updated_at \
             FROM highlights \
             WHERE document_id = $1 AND user_id = $2 \
             ORDER BY created_at ASC",
            document_id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(Highlight::try_from).collect()
    }

    pub(super) async fn count_by_document_impl(
        &self,
        document_id: DocumentId,
        user_id: UserId,
    ) -> Result<i64, AppError> {
        let record = sqlx::query!(
            "SELECT COUNT(*) as count FROM highlights WHERE document_id = $1 AND user_id = $2",
            document_id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(record.count.unwrap_or(0))
    }
}
