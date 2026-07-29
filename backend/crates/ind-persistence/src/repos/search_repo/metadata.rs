use super::types::{SearchIndexedHighlightRow, map_sqlx_error};
use super::*;

impl PgSearchRepository {
    pub(super) async fn get_document_note_text_impl(
        &self,
        document_id: DocumentId,
    ) -> Result<Option<String>, AppError> {
        sqlx::query_scalar!(
            "SELECT body FROM item_notes WHERE document_id = $1",
            document_id.into_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)
    }
    pub(super) async fn list_highlights_for_document_impl(
        &self,
        document_id: DocumentId,
    ) -> Result<Vec<SearchIndexedHighlight>, AppError> {
        let rows = sqlx::query_as!(
            SearchIndexedHighlightRow,
            r#"
            SELECT
                h.id AS highlight_id,
                h.text_content AS text,
                hn.body AS "note?",
                CASE
                    WHEN h.locator ->> 'type' = 'epub' THEN h.locator ->> 'chapter'
                    ELSE NULL
                END AS "section_key?"
            FROM highlights h
            LEFT JOIN highlight_notes hn ON hn.highlight_id = h.id
            WHERE h.document_id = $1
            ORDER BY h.created_at ASC
            "#,
            document_id.into_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(SearchIndexedHighlight::from).collect())
    }
}
