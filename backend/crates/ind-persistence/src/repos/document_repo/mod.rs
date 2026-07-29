mod reads;
pub(crate) mod rows;
pub(crate) mod tx_writes;
mod writes;

use sqlx::PgPool;
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::document::{
    DocumentRenderedMetadata, DocumentRepository, DocumentYoutubeEnrichment,
};
use ind_domain::{
    Document, DocumentId, DocumentOriginType, DocumentProvenance, NewOriginDocument,
    NewUrlDocument, UserId,
};

pub struct PgDocumentRepository {
    pool: PgPool,
}

impl PgDocumentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl DocumentRepository for PgDocumentRepository {
    async fn find_by_id(
        &self,
        user_id: UserId,
        id: DocumentId,
    ) -> Result<Option<Document>, AppError> {
        self.find_by_id_impl(user_id, id).await
    }

    async fn find_by_id_global(&self, id: DocumentId) -> Result<Option<Document>, AppError> {
        self.find_by_id_global_impl(id).await
    }

    async fn list_ids_for_reindex(
        &self,
        after_created_at: Option<chrono::DateTime<chrono::Utc>>,
        after_id: Option<uuid::Uuid>,
        limit: i64,
    ) -> Result<Vec<(DocumentId, chrono::DateTime<chrono::Utc>)>, AppError> {
        self.list_ids_for_reindex_impl(after_created_at, after_id, limit)
            .await
    }

    async fn find_by_canonical_url(
        &self,
        user_id: UserId,
        canonical_url: &str,
    ) -> Result<Option<Document>, AppError> {
        self.find_by_canonical_url_impl(user_id, canonical_url)
            .await
    }

    async fn find_by_origin(
        &self,
        user_id: UserId,
        origin_type: DocumentOriginType,
        origin_id: Uuid,
    ) -> Result<Option<Document>, AppError> {
        self.find_by_origin_impl(user_id, origin_type, origin_id)
            .await
    }

    async fn upsert_url_backed(&self, document: NewUrlDocument) -> Result<Document, AppError> {
        self.upsert_url_backed_impl(document).await
    }

    async fn upsert_origin_backed(
        &self,
        document: NewOriginDocument,
        origin_type: DocumentOriginType,
        origin_id: Uuid,
    ) -> Result<Document, AppError> {
        self.upsert_origin_backed_impl(document, origin_type, origin_id)
            .await
    }

    async fn record_origin(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        origin_type: DocumentOriginType,
        origin_id: Uuid,
    ) -> Result<(), AppError> {
        self.record_origin_impl(user_id, document_id, origin_type, origin_id)
            .await
    }

    async fn set_reading_metrics(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        word_count: i32,
        reading_time_minutes: i32,
    ) -> Result<(), AppError> {
        self.set_reading_metrics_impl(user_id, document_id, word_count, reading_time_minutes)
            .await
    }

    async fn set_language_if_missing(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        language: &str,
    ) -> Result<bool, AppError> {
        self.set_language_if_missing_impl(user_id, document_id, language)
            .await
    }

    async fn set_lead_image(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        lead_image_url: &str,
    ) -> Result<(), AppError> {
        self.set_lead_image_impl(user_id, document_id, lead_image_url)
            .await
    }

    async fn apply_rendered_metadata(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        metadata: DocumentRenderedMetadata,
    ) -> Result<(), AppError> {
        self.apply_rendered_metadata_impl(user_id, document_id, metadata)
            .await
    }

    async fn apply_youtube_enrichment(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        enrichment: DocumentYoutubeEnrichment,
    ) -> Result<(), AppError> {
        self.apply_youtube_enrichment_impl(user_id, document_id, enrichment)
            .await
    }

    async fn load_provenance(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<Option<DocumentProvenance>, AppError> {
        self.load_provenance_impl(user_id, document_id).await
    }
}
