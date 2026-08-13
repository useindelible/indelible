use crate::error::AppError;
use ind_domain::{DocumentId, MilaPlatformDefaults, UserId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveEmbeddingTarget {
    pub embedding_model: String,
    pub embedding_dim: i32,
}

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

    async fn enqueue_target_vector_repairs(
        &self,
        defaults: &MilaPlatformDefaults,
        limit: i64,
    ) -> Result<i64, AppError>;

    async fn enqueue_user_vector_repairs(
        &self,
        user_id: UserId,
        target: &EffectiveEmbeddingTarget,
        limit: i64,
    ) -> Result<i64, AppError>;

    async fn retry_user_vector_repairs(
        &self,
        user_id: UserId,
        target: &EffectiveEmbeddingTarget,
        limit: i64,
    ) -> Result<i64, AppError>;

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

    async fn has_active_embedding_work(&self, user_id: UserId) -> Result<bool, AppError>;
}
