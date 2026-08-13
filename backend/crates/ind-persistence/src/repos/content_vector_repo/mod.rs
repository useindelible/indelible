mod fts;
mod refs;
mod semantic;
mod types;
mod vectors;

use sqlx::PgPool;
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::content_vector::{
    CollectionDocumentFtsQuery, CollectionDocumentVectorQuery, ContentVectorRepository,
    ContentVectorSourceRef, CrossDocumentFtsQuery, CrossDocumentVectorQuery,
    SingleDocumentFtsQuery, SingleDocumentVectorQuery, VectorReplacementOutcome,
};
use ind_application::repos::embedding_backfill::EffectiveEmbeddingTarget;
use ind_domain::{ContentVector, DocumentId, MilaPlatformDefaults, SearchHit, UserId};

pub struct PgContentVectorRepository {
    pool: PgPool,
}

impl PgContentVectorRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ContentVectorRepository for PgContentVectorRepository {
    async fn upsert_chunk(&self, vector: &ContentVector) -> Result<ContentVector, AppError> {
        self.upsert_chunk_impl(vector).await
    }

    async fn replace_for_document(
        &self,
        document_id: DocumentId,
        vectors: &[ContentVector],
    ) -> Result<(), AppError> {
        self.replace_for_document_impl(document_id, vectors).await
    }

    async fn replace_for_document_if_target_current(
        &self,
        document_id: DocumentId,
        user_id: UserId,
        vectors: &[ContentVector],
        generated_target: &EffectiveEmbeddingTarget,
        platform_defaults: &MilaPlatformDefaults,
    ) -> Result<VectorReplacementOutcome, AppError> {
        self.replace_for_document_if_target_current_impl(
            document_id,
            user_id,
            vectors,
            generated_target,
            platform_defaults,
        )
        .await
    }

    async fn delete_for_document(&self, document_id: DocumentId) -> Result<(), AppError> {
        self.delete_for_document_impl(document_id).await
    }

    async fn delete_for_user(&self, user_id: UserId) -> Result<(), AppError> {
        self.delete_for_user_impl(user_id).await
    }

    async fn search_single_document(
        &self,
        query: &SingleDocumentVectorQuery,
    ) -> Result<Vec<SearchHit>, AppError> {
        self.search_single_document_impl(query).await
    }

    async fn search_cross_document(
        &self,
        query: &CrossDocumentVectorQuery,
    ) -> Result<Vec<SearchHit>, AppError> {
        self.search_cross_document_impl(query).await
    }

    async fn search_collection_document(
        &self,
        query: &CollectionDocumentVectorQuery,
    ) -> Result<Vec<SearchHit>, AppError> {
        self.search_collection_document_impl(query).await
    }

    async fn count_documents_by_user(&self, user_id: UserId) -> Result<i64, AppError> {
        self.count_documents_by_user_impl(user_id).await
    }

    async fn source_refs_for_chunks(
        &self,
        chunk_ids: &[Uuid],
    ) -> Result<Vec<ContentVectorSourceRef>, AppError> {
        self.source_refs_for_chunks_impl(chunk_ids).await
    }

    async fn fts_single_document(
        &self,
        query: &SingleDocumentFtsQuery,
    ) -> Result<Vec<SearchHit>, AppError> {
        self.fts_single_document_impl(query).await
    }

    async fn fts_cross_document(
        &self,
        query: &CrossDocumentFtsQuery,
    ) -> Result<Vec<SearchHit>, AppError> {
        self.fts_cross_document_impl(query).await
    }

    async fn fts_collection_document(
        &self,
        query: &CollectionDocumentFtsQuery,
    ) -> Result<Vec<SearchHit>, AppError> {
        self.fts_collection_document_impl(query).await
    }
}
