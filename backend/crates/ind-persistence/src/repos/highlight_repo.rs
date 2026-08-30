use std::collections::HashMap;

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::event::MutationSideEffects;
use ind_application::repos::highlight::{HighlightRepository, HighlightWrite};
use ind_domain::{
    DocumentId, DomainError, Highlight, HighlightId, HighlightNote, NewHighlight, Tag, TagId,
    UserId,
};

use super::write_helpers::apply_mutation_side_effects_tx;
use rows::{HighlightNoteRow, HighlightRow, HighlightTagRow, TagRow};

mod document;
mod rows;

pub struct PgHighlightRepository {
    pool: PgPool,
}

impl PgHighlightRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

pub(super) fn map_sqlx_error(err: sqlx::Error) -> AppError {
    super::map_sqlx_error("highlight", "duplicate highlight", err)
}

#[async_trait::async_trait]
impl HighlightRepository for PgHighlightRepository {
    async fn create(
        &self,
        highlight: &NewHighlight,
        effects: MutationSideEffects,
    ) -> Result<Highlight, AppError> {
        let now = Utc::now();
        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;
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

    async fn create_for_document(
        &self,
        highlight: &NewHighlight,
        effects: MutationSideEffects,
    ) -> Result<HighlightWrite, AppError> {
        self.create_for_document_impl(highlight, effects).await
    }

    async fn list_by_document(
        &self,
        document_id: DocumentId,
        user_id: UserId,
    ) -> Result<Vec<Highlight>, AppError> {
        self.list_by_document_impl(document_id, user_id).await
    }

    async fn count_by_document(
        &self,
        document_id: DocumentId,
        user_id: UserId,
    ) -> Result<i64, AppError> {
        self.count_by_document_impl(document_id, user_id).await
    }

    async fn get_by_id(
        &self,
        id: HighlightId,
        user_id: UserId,
    ) -> Result<Option<Highlight>, AppError> {
        let row = sqlx::query_as!(
            HighlightRow,
            "SELECT id, document_id AS \"document_id!\", user_id, color, text_content, locator, source_locator, created_at, updated_at \
             FROM highlights \
             WHERE id = $1 AND user_id = $2",
            id.into_uuid(),
            user_id.into_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Highlight::try_from).transpose()
    }

    async fn list_recent_by_user(
        &self,
        user_id: UserId,
        limit: i64,
    ) -> Result<Vec<Highlight>, AppError> {
        let rows = sqlx::query_as!(
            HighlightRow,
            "SELECT id, document_id AS \"document_id!\", user_id, color, text_content, locator, source_locator, created_at, updated_at \
             FROM highlights \
             WHERE user_id = $1 \
             ORDER BY created_at DESC \
             LIMIT $2",
            user_id.into_uuid(),
            limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(Highlight::try_from).collect()
    }

    async fn update_color(
        &self,
        id: HighlightId,
        user_id: UserId,
        color: &str,
        effects: MutationSideEffects,
    ) -> Result<Highlight, AppError> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;
        let row = sqlx::query_as!(
            HighlightRow,
            "UPDATE highlights SET color = $3, updated_at = now() \
             WHERE id = $1 AND user_id = $2 \
             RETURNING id, document_id AS \"document_id!\", user_id, color, text_content, locator, source_locator, created_at, updated_at",
            id.into_uuid(),
            user_id.into_uuid(),
            color,
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_sqlx_error)?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "highlight",
                id: id.to_string(),
            })
        })?;

        apply_mutation_side_effects_tx(&mut tx, effects).await?;
        tx.commit().await.map_err(map_sqlx_error)?;
        Highlight::try_from(row)
    }

    async fn delete(
        &self,
        id: HighlightId,
        user_id: UserId,
        effects: MutationSideEffects,
    ) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;
        let result = sqlx::query!(
            "DELETE FROM highlights WHERE id = $1 AND user_id = $2",
            id.into_uuid(),
            user_id.into_uuid()
        )
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(AppError::Domain(DomainError::NotFound {
                entity: "highlight",
                id: id.to_string(),
            }));
        }

        apply_mutation_side_effects_tx(&mut tx, effects).await?;
        tx.commit().await.map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn upsert_note(
        &self,
        highlight_id: HighlightId,
        user_id: UserId,
        body: &str,
        effects: MutationSideEffects,
    ) -> Result<HighlightNote, AppError> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;
        // SELECT from highlights verifies ownership before the INSERT/UPDATE.
        // If highlight_id does not belong to user_id the SELECT returns no rows,
        // nothing is written, and fetch_optional returns None -> NotFound.
        let row = sqlx::query_as!(
            HighlightNoteRow,
            "INSERT INTO highlight_notes (id, highlight_id, body, created_at, updated_at) \
             SELECT $1, h.id, $3, $4, $4 FROM highlights h \
             WHERE h.id = $2 AND h.user_id = $5 \
             ON CONFLICT (highlight_id) DO UPDATE SET \
             body = EXCLUDED.body, \
             updated_at = EXCLUDED.updated_at \
             RETURNING id, highlight_id, body, created_at, updated_at",
            id,
            highlight_id.into_uuid(),
            body,
            now,
            user_id.into_uuid(),
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_sqlx_error)?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "highlight",
                id: highlight_id.to_string(),
            })
        })?;

        apply_mutation_side_effects_tx(&mut tx, effects).await?;
        tx.commit().await.map_err(map_sqlx_error)?;
        Ok(HighlightNote::from(row))
    }

    async fn delete_note(
        &self,
        highlight_id: HighlightId,
        user_id: UserId,
        effects: MutationSideEffects,
    ) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;
        sqlx::query!(
            "DELETE FROM highlight_notes \
             WHERE highlight_id = $1 \
             AND EXISTS (SELECT 1 FROM highlights WHERE id = $1 AND user_id = $2)",
            highlight_id.into_uuid(),
            user_id.into_uuid(),
        )
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        apply_mutation_side_effects_tx(&mut tx, effects).await?;
        tx.commit().await.map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn get_note(
        &self,
        highlight_id: HighlightId,
        user_id: UserId,
    ) -> Result<Option<HighlightNote>, AppError> {
        let row = sqlx::query_as!(
            HighlightNoteRow,
            "SELECT hn.id, hn.highlight_id, hn.body, hn.created_at, hn.updated_at \
             FROM highlight_notes hn \
             JOIN highlights h ON h.id = hn.highlight_id \
             WHERE hn.highlight_id = $1 AND h.user_id = $2",
            highlight_id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(HighlightNote::from))
    }

    async fn add_tag(
        &self,
        highlight_id: HighlightId,
        user_id: UserId,
        tag_id: TagId,
        effects: MutationSideEffects,
    ) -> Result<(), AppError> {
        let now = Utc::now();
        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;
        // Verify highlight ownership and that the tag belongs to the same user
        // before associating. ON CONFLICT DO NOTHING makes the call idempotent.
        sqlx::query!(
            "INSERT INTO highlight_tags (highlight_id, tag_id, added_at) \
             SELECT $1, $2, $3 \
             WHERE EXISTS (SELECT 1 FROM highlights WHERE id = $1 AND user_id = $4) \
             AND EXISTS (SELECT 1 FROM tags WHERE id = $2 AND user_id = $4) \
             ON CONFLICT DO NOTHING",
            highlight_id.into_uuid(),
            tag_id.into_uuid(),
            now,
            user_id.into_uuid(),
        )
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        apply_mutation_side_effects_tx(&mut tx, effects).await?;
        tx.commit().await.map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn remove_tag(
        &self,
        highlight_id: HighlightId,
        user_id: UserId,
        tag_id: TagId,
        effects: MutationSideEffects,
    ) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;
        sqlx::query!(
            "DELETE FROM highlight_tags \
             WHERE highlight_id = $1 AND tag_id = $2 \
             AND EXISTS (SELECT 1 FROM highlights WHERE id = $1 AND user_id = $3)",
            highlight_id.into_uuid(),
            tag_id.into_uuid(),
            user_id.into_uuid(),
        )
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        apply_mutation_side_effects_tx(&mut tx, effects).await?;
        tx.commit().await.map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn list_tags(
        &self,
        highlight_id: HighlightId,
        user_id: UserId,
    ) -> Result<Vec<Tag>, AppError> {
        let rows = sqlx::query_as!(
            TagRow,
            "SELECT t.id, t.user_id, t.name, t.color, t.parent_id, t.created_at \
             FROM tags t \
             JOIN highlight_tags ht ON ht.tag_id = t.id \
             JOIN highlights h ON h.id = ht.highlight_id \
             WHERE ht.highlight_id = $1 AND h.user_id = $2 \
             ORDER BY t.name ASC",
            highlight_id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(Tag::from).collect())
    }

    async fn list_tags_for_highlights(
        &self,
        highlight_ids: &[HighlightId],
        user_id: UserId,
    ) -> Result<HashMap<HighlightId, Vec<Tag>>, AppError> {
        if highlight_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let highlight_uuids: Vec<Uuid> = highlight_ids.iter().map(|id| id.into_uuid()).collect();
        let rows = sqlx::query_as!(
            HighlightTagRow,
            "SELECT ht.highlight_id, t.id, t.user_id, t.name, t.color, t.parent_id, t.created_at \
             FROM tags t \
             JOIN highlight_tags ht ON ht.tag_id = t.id \
             JOIN highlights h ON h.id = ht.highlight_id \
             WHERE ht.highlight_id = ANY($1) AND h.user_id = $2 \
             ORDER BY ht.highlight_id ASC, t.name ASC",
            &highlight_uuids,
            user_id.into_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        let mut tags_by_highlight: HashMap<HighlightId, Vec<Tag>> = HashMap::new();
        for row in rows {
            let highlight_id = HighlightId::from_uuid(row.highlight_id);
            tags_by_highlight
                .entry(highlight_id)
                .or_default()
                .push(Tag::from(TagRow {
                    id: row.id,
                    user_id: row.user_id,
                    name: row.name,
                    color: row.color,
                    parent_id: row.parent_id,
                    created_at: row.created_at,
                }));
        }

        Ok(tags_by_highlight)
    }

    async fn list_by_document_after_cursor(
        &self,
        document_id: ind_domain::DocumentId,
        user_id: UserId,
        after_created_at: Option<chrono::DateTime<chrono::Utc>>,
        after_id: Option<HighlightId>,
        limit: i64,
    ) -> Result<Vec<Highlight>, ind_application::AppError> {
        let rows = sqlx::query_as!(
            HighlightRow,
            r#"SELECT id, document_id AS "document_id!", user_id, color, text_content, locator, source_locator, created_at, updated_at
               FROM highlights
               WHERE document_id = $1
                 AND user_id = $2
                 AND (
                       $3::timestamptz IS NULL
                       OR created_at > $3
                       OR (created_at = $3 AND id > $4::uuid)
                     )
               ORDER BY created_at ASC, id ASC
               LIMIT $5"#,
            document_id.into_uuid(),
            user_id.into_uuid(),
            after_created_at as Option<chrono::DateTime<chrono::Utc>>,
            after_id.map(|id| id.into_uuid()) as Option<Uuid>,
            limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(Highlight::try_from).collect()
    }
}
