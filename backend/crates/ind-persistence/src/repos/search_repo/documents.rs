use super::types::{SearchDocumentRow, map_sqlx_error, search_document_kind_to_str};
use super::*;

async fn insert_search_document(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    document: &SearchDocument,
) -> Result<(), AppError> {
    sqlx::query!(
        r#"
        INSERT INTO search_documents (
            id, document_id, user_id, document_kind, section_key,
            section_title, title, body_text, highlight_text, metadata_text, search_config,
            saved_at, updated_at
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, ($11::text)::regconfig, $12, $13
        )
        "#,
        document.id.as_uuid(),
        document.document_id().into_uuid(),
        document.user_id.into_uuid(),
        search_document_kind_to_str(document.document_kind),
        document.section_key,
        document.section_title.as_deref(),
        document.title,
        document.body_text,
        document.highlight_text,
        document.metadata_text,
        document.search_config,
        document.saved_at,
        document.updated_at,
    )
    .execute(&mut **tx)
    .await
    .map_err(map_sqlx_error)?;
    Ok(())
}

impl PgSearchRepository {
    pub(super) async fn upsert_search_document_impl(
        &self,
        document: &SearchDocument,
    ) -> Result<SearchDocument, AppError> {
        let document_id = document.document_id().into_uuid();
        let row = sqlx::query_as!(
            SearchDocumentRow,
            r#"
            INSERT INTO search_documents (
                id, document_id, user_id, document_kind, section_key,
                section_title, title, body_text, highlight_text, metadata_text,
                search_config, saved_at, updated_at
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, ($11::text)::regconfig, $12, $13
            )
            ON CONFLICT (document_id, section_key) WHERE document_id IS NOT NULL DO UPDATE SET
                user_id = EXCLUDED.user_id,
                document_kind = EXCLUDED.document_kind,
                section_title = EXCLUDED.section_title,
                title = EXCLUDED.title,
                body_text = EXCLUDED.body_text,
                highlight_text = EXCLUDED.highlight_text,
                metadata_text = EXCLUDED.metadata_text,
                search_config = EXCLUDED.search_config,
                saved_at = EXCLUDED.saved_at,
                updated_at = EXCLUDED.updated_at
            RETURNING
                id,
                document_id AS "document_id!",
                user_id,
                document_kind,
                section_key,
                section_title,
                title,
                body_text,
                highlight_text,
                metadata_text,
                search_config::text AS "search_config!",
                saved_at,
                updated_at
            "#,
            document.id.as_uuid(),
            document_id,
            document.user_id.into_uuid(),
            search_document_kind_to_str(document.document_kind),
            document.section_key,
            document.section_title.as_deref(),
            document.title,
            document.body_text,
            document.highlight_text,
            document.metadata_text,
            document.search_config,
            document.saved_at,
            document.updated_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        SearchDocument::try_from(row)
    }
    pub(super) async fn replace_search_documents_for_document_impl(
        &self,
        document_id: DocumentId,
        documents: &[SearchDocument],
    ) -> Result<(), AppError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;

        sqlx::query!(
            "DELETE FROM search_documents WHERE document_id = $1",
            document_id.into_uuid()
        )
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        for document in documents {
            insert_search_document(&mut tx, document).await?;
        }

        tx.commit()
            .await
            .map_err(|err| AppError::Repository(Box::new(err)))?;

        Ok(())
    }
    pub(super) async fn delete_search_documents_for_document_impl(
        &self,
        document_id: DocumentId,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "DELETE FROM search_documents WHERE document_id = $1",
            document_id.into_uuid()
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }
}
