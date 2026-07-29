use crate::error::AppError;
use ind_domain::{CollectionId, ContentVector, DocumentId, SearchHit, SearchSectionKind, UserId};
use uuid::Uuid;

/// Source reference for a retrieved content-vector chunk, resolved to its document (TASK-234).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentVectorSourceRef {
    pub chunk_id: Uuid,
    pub document_id: DocumentId,
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct SingleDocumentFtsQuery {
    pub user_id: UserId,
    pub document_id: DocumentId,
    pub text_query: String,
    pub limit: i64,
}

#[derive(Debug, Clone)]
pub struct CrossDocumentFtsQuery {
    pub user_id: UserId,
    pub text_query: String,
    pub limit: i64,
}

#[derive(Debug, Clone)]
pub struct CollectionDocumentFtsQuery {
    pub user_id: UserId,
    pub collection_id: CollectionId,
    pub include_descendants: bool,
    pub text_query: String,
    pub limit: i64,
}

#[derive(Debug, Clone)]
pub struct SingleDocumentVectorQuery {
    pub user_id: UserId,
    pub document_id: DocumentId,
    pub query_embedding: Vec<f32>,
    pub embedding_model: String,
    pub embedding_dim: i32,
    pub section_kind: Option<SearchSectionKind>,
    pub limit: i64,
}

#[derive(Debug, Clone)]
pub struct CrossDocumentVectorQuery {
    pub user_id: UserId,
    pub query_embedding: Vec<f32>,
    pub embedding_model: String,
    pub embedding_dim: i32,
    pub section_kind: Option<SearchSectionKind>,
    pub limit: i64,
}

#[derive(Debug, Clone)]
pub struct CollectionDocumentVectorQuery {
    pub user_id: UserId,
    pub collection_id: CollectionId,
    pub include_descendants: bool,
    pub query_embedding: Vec<f32>,
    pub embedding_model: String,
    pub embedding_dim: i32,
    pub section_kind: Option<SearchSectionKind>,
    pub limit: i64,
}

#[async_trait::async_trait]
pub trait ContentVectorRepository: Send + Sync {
    async fn upsert_chunk(&self, vector: &ContentVector) -> Result<ContentVector, AppError>;

    async fn replace_for_document(
        &self,
        document_id: DocumentId,
        vectors: &[ContentVector],
    ) -> Result<(), AppError>;

    async fn delete_for_document(&self, document_id: DocumentId) -> Result<(), AppError>;

    async fn delete_for_user(&self, user_id: UserId) -> Result<(), AppError>;

    async fn search_single_document(
        &self,
        query: &SingleDocumentVectorQuery,
    ) -> Result<Vec<SearchHit>, AppError>;

    async fn search_cross_document(
        &self,
        query: &CrossDocumentVectorQuery,
    ) -> Result<Vec<SearchHit>, AppError>;

    async fn search_collection_document(
        &self,
        query: &CollectionDocumentVectorQuery,
    ) -> Result<Vec<SearchHit>, AppError>;

    async fn count_documents_by_user(&self, user_id: UserId) -> Result<i64, AppError>;

    async fn source_refs_for_chunks(
        &self,
        chunk_ids: &[Uuid],
    ) -> Result<Vec<ContentVectorSourceRef>, AppError>;

    async fn fts_single_document(
        &self,
        query: &SingleDocumentFtsQuery,
    ) -> Result<Vec<SearchHit>, AppError>;

    async fn fts_cross_document(
        &self,
        query: &CrossDocumentFtsQuery,
    ) -> Result<Vec<SearchHit>, AppError>;

    async fn fts_collection_document(
        &self,
        query: &CollectionDocumentFtsQuery,
    ) -> Result<Vec<SearchHit>, AppError>;
}
