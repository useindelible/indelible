use ind_application::AppError;
use ind_domain::{
    ContentSource, Document, DocumentId, DocumentOriginType, DocumentProvenance, DomainError,
    UserId,
};
use uuid::Uuid;

use super::PgDocumentRepository;
use super::rows::{DocumentRow, map_document_error, map_origin_error, origin_type_to_str};

impl PgDocumentRepository {
    pub(super) async fn find_by_id_impl(
        &self,
        user_id: UserId,
        id: DocumentId,
    ) -> Result<Option<Document>, AppError> {
        let row = sqlx::query_as!(
            DocumentRow,
            "SELECT id, user_id, document_type, canonical_url, original_url, content_hash, \
                    title, author, excerpt, published_at, language, domain, lead_image_url, \
                    thumbnail_url, word_count, reading_time_minutes, created_at, updated_at \
             FROM documents WHERE id = $1 AND user_id = $2",
            id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_document_error)?;

        row.map(DocumentRow::into_document).transpose()
    }

    pub(super) async fn load_provenance_impl(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<Option<DocumentProvenance>, AppError> {
        let Some(row) = sqlx::query!(
            r#"SELECT
                EXISTS(
                    SELECT 1 FROM library_entries le
                    WHERE le.user_id = $1 AND le.document_id = $2 AND le.deleted_at IS NULL
                ) AS "is_saved!",
                (
                    SELECT le.source FROM library_entries le
                    WHERE le.user_id = $1 AND le.document_id = $2 AND le.deleted_at IS NULL
                    LIMIT 1
                ) AS "library_source?",
                EXISTS(
                    SELECT 1 FROM highlights h
                    WHERE h.user_id = $1 AND h.document_id = $2
                ) AS "has_highlights!",
                EXISTS(
                    SELECT 1 FROM item_notes n
                    WHERE n.user_id = $1 AND n.document_id = $2
                ) AS "has_note!",
                EXISTS(
                    SELECT 1 FROM mila_sessions m
                    WHERE m.user_id = $1 AND m.document_id = $2
                ) AS "has_mila_session!"
               FROM documents d
               WHERE d.id = $2 AND d.user_id = $1"#,
            user_id.into_uuid(),
            document_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_document_error)?
        else {
            return Ok(None);
        };

        let origin_types = sqlx::query_scalar!(
            "SELECT origin_type FROM document_origins WHERE user_id = $1 AND document_id = $2 \
             ORDER BY created_at",
            user_id.into_uuid(),
            document_id.into_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_origin_error)?;

        let origins = origin_types
            .iter()
            .map(|value| {
                value.parse::<DocumentOriginType>().map_err(|_| {
                    AppError::Domain(DomainError::InvariantViolation {
                        message: format!("unknown document origin_type: {value}"),
                    })
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let library_source = row
            .library_source
            .map(|value| {
                value.parse::<ContentSource>().map_err(|_| {
                    AppError::Domain(DomainError::InvariantViolation {
                        message: format!("unknown library_entries.source: {value}"),
                    })
                })
            })
            .transpose()?;

        Ok(Some(DocumentProvenance {
            document_id,
            is_saved: row.is_saved,
            library_source,
            origins,
            has_highlights: row.has_highlights,
            has_note: row.has_note,
            has_mila_session: row.has_mila_session,
        }))
    }

    pub(super) async fn find_by_id_global_impl(
        &self,
        id: DocumentId,
    ) -> Result<Option<Document>, AppError> {
        let row = sqlx::query_as!(
            DocumentRow,
            "SELECT id, user_id, document_type, canonical_url, original_url, content_hash, \
                    title, author, excerpt, published_at, language, domain, lead_image_url, \
                    thumbnail_url, word_count, reading_time_minutes, created_at, updated_at \
             FROM documents WHERE id = $1",
            id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_document_error)?;

        row.map(DocumentRow::into_document).transpose()
    }

    pub(super) async fn list_ids_for_reindex_impl(
        &self,
        after_created_at: Option<chrono::DateTime<chrono::Utc>>,
        after_id: Option<Uuid>,
        limit: i64,
    ) -> Result<Vec<(ind_domain::DocumentId, chrono::DateTime<chrono::Utc>)>, AppError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, created_at
            FROM documents
            WHERE (
                    $1::timestamptz IS NULL
                    OR created_at < $1
                    OR (created_at = $1 AND id < $2)
              )
            ORDER BY created_at DESC, id DESC
            LIMIT $3
            "#,
            after_created_at,
            after_id,
            limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_document_error)?;

        Ok(rows
            .into_iter()
            .map(|row| (ind_domain::DocumentId::from_uuid(row.id), row.created_at))
            .collect())
    }

    pub(super) async fn find_by_canonical_url_impl(
        &self,
        user_id: UserId,
        canonical_url: &str,
    ) -> Result<Option<Document>, AppError> {
        let row = sqlx::query_as!(
            DocumentRow,
            "SELECT id, user_id, document_type, canonical_url, original_url, content_hash, \
                    title, author, excerpt, published_at, language, domain, lead_image_url, \
                    thumbnail_url, word_count, reading_time_minutes, created_at, updated_at \
             FROM documents WHERE user_id = $1 AND canonical_url = $2",
            user_id.into_uuid(),
            canonical_url,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_document_error)?;

        row.map(DocumentRow::into_document).transpose()
    }

    pub(super) async fn find_by_origin_impl(
        &self,
        user_id: UserId,
        origin_type: DocumentOriginType,
        origin_id: Uuid,
    ) -> Result<Option<Document>, AppError> {
        let row = sqlx::query_as!(
            DocumentRow,
            "SELECT d.id, d.user_id, d.document_type, d.canonical_url, d.original_url, \
                    d.content_hash, d.title, d.author, d.excerpt, d.published_at, d.language, \
                    d.domain, d.lead_image_url, d.thumbnail_url, d.word_count, d.reading_time_minutes, d.created_at, d.updated_at \
             FROM documents d \
             JOIN document_origins o ON o.document_id = d.id AND o.user_id = d.user_id \
             WHERE o.user_id = $1 AND o.origin_type = $2 AND o.origin_id = $3",
            user_id.into_uuid(),
            origin_type_to_str(origin_type),
            origin_id,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_document_error)?;

        row.map(DocumentRow::into_document).transpose()
    }
}
