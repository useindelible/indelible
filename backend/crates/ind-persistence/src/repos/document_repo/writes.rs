use ind_application::repos::document::{DocumentRenderedMetadata, DocumentYoutubeEnrichment};
use ind_application::{AppError, normalize_language_tag};
use ind_domain::{
    Document, DocumentId, DocumentOriginType, DomainError, NewOriginDocument, NewUrlDocument,
    UserId,
};
use uuid::Uuid;

use super::PgDocumentRepository;
use super::rows::map_document_error;
use super::tx_writes::{materialize_origin_backed_tx, materialize_url_backed_tx, record_origin_tx};

impl PgDocumentRepository {
    pub(super) async fn set_language_if_missing_impl(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        language: &str,
    ) -> Result<bool, AppError> {
        let Some(language) = normalize_language_tag(Some(language)) else {
            return Ok(false);
        };
        let result = sqlx::query!(
            "UPDATE documents \
             SET language = $3, updated_at = now() \
             WHERE id = $1 AND user_id = $2 AND language IS NULL",
            document_id.into_uuid(),
            user_id.into_uuid(),
            language,
        )
        .execute(&self.pool)
        .await
        .map_err(map_document_error)?;

        Ok(result.rows_affected() == 1)
    }

    pub(super) async fn apply_rendered_metadata_impl(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        metadata: DocumentRenderedMetadata,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"UPDATE documents
               SET title = CASE
                       WHEN $3::text IS NOT NULL
                            AND (btrim(title) = '' OR title = canonical_url OR title = original_url)
                           THEN $3
                       ELSE title
                   END,
                   author = COALESCE(NULLIF(author, ''), $4),
                   excerpt = COALESCE(NULLIF(excerpt, ''), $5),
                   updated_at = now()
               WHERE id = $1 AND user_id = $2"#,
            document_id.into_uuid(),
            user_id.into_uuid(),
            metadata.title,
            metadata.author,
            metadata.excerpt,
        )
        .execute(&self.pool)
        .await
        .map_err(map_document_error)?;
        Ok(())
    }

    pub(super) async fn upsert_url_backed_impl(
        &self,
        doc: NewUrlDocument,
    ) -> Result<Document, AppError> {
        let mut tx = self.pool.begin().await.map_err(map_document_error)?;
        let (row, _created) = materialize_url_backed_tx(&mut tx, &doc).await?;
        tx.commit().await.map_err(map_document_error)?;
        row.into_document()
    }

    pub(super) async fn upsert_origin_backed_impl(
        &self,
        doc: NewOriginDocument,
        origin_type: DocumentOriginType,
        origin_id: Uuid,
    ) -> Result<Document, AppError> {
        let mut tx = self.pool.begin().await.map_err(map_document_error)?;
        let (row, _created) =
            materialize_origin_backed_tx(&mut tx, &doc, origin_type, origin_id).await?;
        tx.commit().await.map_err(map_document_error)?;
        row.into_document()
    }

    pub(super) async fn record_origin_impl(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        origin_type: DocumentOriginType,
        origin_id: Uuid,
    ) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await.map_err(map_document_error)?;
        record_origin_tx(&mut tx, user_id, document_id, origin_type, origin_id).await?;
        tx.commit().await.map_err(map_document_error)
    }

    /// Fill-if-absent so a feed/RSS image set at materialize time is preserved; the rendered
    /// og:image only lands when the document has none. `thumbnail_url` is filled from the same URL
    /// when it too is absent, mirroring the extension save. Best-effort: 0 rows means an image
    /// already existed (or the document is gone), neither of which is an error for enrichment.
    pub(super) async fn set_lead_image_impl(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        lead_image_url: &str,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE documents \
             SET lead_image_url = $3, \
                 thumbnail_url = COALESCE(thumbnail_url, $3), \
                 updated_at = now() \
             WHERE id = $1 AND user_id = $2 AND lead_image_url IS NULL",
            document_id.into_uuid(),
            user_id.into_uuid(),
            lead_image_url,
        )
        .execute(&self.pool)
        .await
        .map_err(map_document_error)?;
        Ok(())
    }

    pub(super) async fn set_reading_metrics_impl(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        word_count: i32,
        reading_time_minutes: i32,
    ) -> Result<(), AppError> {
        let result = sqlx::query!(
            "UPDATE documents \
             SET word_count = $3, reading_time_minutes = $4, updated_at = now() \
             WHERE id = $1 AND user_id = $2 \
               AND (word_count IS DISTINCT FROM $3 OR reading_time_minutes IS DISTINCT FROM $4)",
            document_id.into_uuid(),
            user_id.into_uuid(),
            word_count,
            reading_time_minutes,
        )
        .execute(&self.pool)
        .await
        .map_err(map_document_error)?;

        // Distinguish "no-op same values" from "document missing": retried attach/prepare jobs
        // must stay idempotent, but a vanished document is a caller bug worth surfacing.
        if result.rows_affected() == 0 {
            let exists = sqlx::query_scalar!(
                r#"SELECT EXISTS(SELECT 1 FROM documents WHERE id = $1 AND user_id = $2) AS "exists!""#,
                document_id.into_uuid(),
                user_id.into_uuid(),
            )
            .fetch_one(&self.pool)
            .await
            .map_err(map_document_error)?;
            if !exists {
                return Err(AppError::Domain(DomainError::NotFound {
                    entity: "Document",
                    id: document_id.to_string(),
                }));
            }
        }
        Ok(())
    }

    pub(super) async fn apply_youtube_enrichment_impl(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        enrichment: DocumentYoutubeEnrichment,
    ) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await.map_err(map_document_error)?;

        let result = sqlx::query!(
            r#"UPDATE documents
               SET document_type = 'video',
                   title = COALESCE($3, title),
                   excerpt = COALESCE($4, excerpt),
                   lead_image_url = COALESCE($5, lead_image_url),
                   updated_at = now()
               WHERE id = $1 AND user_id = $2"#,
            document_id.into_uuid(),
            user_id.into_uuid(),
            enrichment.title,
            enrichment.excerpt,
            enrichment.lead_image_url,
        )
        .execute(&mut *tx)
        .await
        .map_err(map_document_error)?;
        if result.rows_affected() == 0 {
            return Err(AppError::Domain(DomainError::NotFound {
                entity: "Document",
                id: document_id.to_string(),
            }));
        }

        // Persist the video sidecar in the SAME transaction, and only after the document update
        // succeeded (TASK-240). Skip the row entirely when both fields are absent to keep it sparse.
        if enrichment.duration_seconds.is_some() || enrichment.youtube_channel_name.is_some() {
            sqlx::query!(
                r#"INSERT INTO document_video_metadata
                       (document_id, duration_seconds, channel_name, created_at, updated_at)
                   VALUES ($1, $2, $3, now(), now())
                   ON CONFLICT (document_id) DO UPDATE
                   SET duration_seconds = COALESCE(
                           EXCLUDED.duration_seconds,
                           document_video_metadata.duration_seconds
                       ),
                       channel_name = COALESCE(
                           EXCLUDED.channel_name,
                           document_video_metadata.channel_name
                       ),
                       updated_at = now()"#,
                document_id.into_uuid(),
                enrichment.duration_seconds,
                enrichment.youtube_channel_name,
            )
            .execute(&mut *tx)
            .await
            .map_err(map_document_error)?;
        }

        tx.commit().await.map_err(map_document_error)?;
        Ok(())
    }
}
