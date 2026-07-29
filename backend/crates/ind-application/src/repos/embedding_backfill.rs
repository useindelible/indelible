use crate::error::AppError;
use ind_domain::{DocumentId, UserId};

#[async_trait::async_trait]
pub trait EmbeddingBackfillRepository: Send + Sync {
    async fn readable_html_document_ids_missing_vectors(
        &self,
        limit: i64,
    ) -> Result<Vec<DocumentId>, AppError>;

    async fn epub_pdf_document_ids_missing_vectors(
        &self,
        limit: i64,
    ) -> Result<Vec<DocumentId>, AppError>;

    async fn enqueue_missing_vector_repairs(&self, limit: i64) -> Result<i64, AppError>;

    async fn eligible_document_ids_for_backfill(
        &self,
        user_id: UserId,
        embedding_model: &str,
        embedding_dim: i32,
    ) -> Result<Vec<DocumentId>, AppError>;

    async fn eligible_document_ids_for_full_reindex(
        &self,
        user_id: UserId,
    ) -> Result<Vec<DocumentId>, AppError>;

    async fn count_eligible_items(&self, user_id: UserId) -> Result<i64, AppError>;

    async fn count_indexed_items(
        &self,
        user_id: UserId,
        embedding_model: &str,
        embedding_dim: i32,
    ) -> Result<i64, AppError>;

    async fn count_stale_items(
        &self,
        user_id: UserId,
        embedding_model: &str,
        embedding_dim: i32,
    ) -> Result<i64, AppError>;

    async fn has_pending_outbox(&self, user_id: UserId) -> Result<bool, AppError>;
}
