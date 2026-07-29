use sqlx::PgPool;
use std::collections::HashMap;

use ind_application::AppError;
use ind_application::repos::search::{RecentSearchRepository, SearchFtsQuery, SearchRepository};
use ind_domain::{
    ContentVector, DocumentId, RecentSearch, RecentSearchId, SearchDocument, SearchEntityCard,
    SearchEntityChip, SearchHit, SearchIndexedHighlight, UserId,
};

use crate::repos::PgContentVectorRepository;

mod documents;
mod entities;
mod fts;
mod metadata;
mod recent;
mod suggestions;
mod types;
mod vectors;

pub struct PgSearchRepository {
    pool: PgPool,
    content_vectors: PgContentVectorRepository,
}

impl PgSearchRepository {
    pub fn new(pool: PgPool) -> Self {
        Self {
            content_vectors: PgContentVectorRepository::new(pool.clone()),
            pool,
        }
    }
}

#[async_trait::async_trait]
impl SearchRepository for PgSearchRepository {
    async fn upsert_search_document(
        &self,
        document: &SearchDocument,
    ) -> Result<SearchDocument, AppError> {
        self.upsert_search_document_impl(document).await
    }

    async fn replace_search_documents_for_document(
        &self,
        document_id: DocumentId,
        documents: &[SearchDocument],
    ) -> Result<(), AppError> {
        self.replace_search_documents_for_document_impl(document_id, documents)
            .await
    }

    async fn delete_search_documents_for_document(
        &self,
        document_id: DocumentId,
    ) -> Result<(), AppError> {
        self.delete_search_documents_for_document_impl(document_id)
            .await
    }

    async fn upsert_content_vector(
        &self,
        vector: &ContentVector,
    ) -> Result<ContentVector, AppError> {
        self.upsert_content_vector_impl(vector).await
    }

    async fn search_fts(&self, query: &SearchFtsQuery) -> Result<Vec<SearchHit>, AppError> {
        self.search_fts_impl(query).await
    }

    async fn get_document_note_text(
        &self,
        document_id: DocumentId,
    ) -> Result<Option<String>, AppError> {
        self.get_document_note_text_impl(document_id).await
    }

    async fn list_highlights_for_document(
        &self,
        document_id: DocumentId,
    ) -> Result<Vec<SearchIndexedHighlight>, AppError> {
        self.list_highlights_for_document_impl(document_id).await
    }

    async fn list_entity_chips_for_documents(
        &self,
        user_id: UserId,
        document_ids: &[DocumentId],
    ) -> Result<HashMap<DocumentId, Vec<SearchEntityChip>>, AppError> {
        self.list_entity_chips_for_documents_impl(user_id, document_ids)
            .await
    }

    async fn suggest_entities(
        &self,
        user_id: UserId,
        prefix: &str,
        limit: i64,
    ) -> Result<Vec<SearchEntityChip>, AppError> {
        self.suggest_entities_impl(user_id, prefix, limit).await
    }

    async fn find_entity_card(
        &self,
        user_id: UserId,
        query: &str,
    ) -> Result<Option<SearchEntityCard>, AppError> {
        self.find_entity_card_impl(user_id, query).await
    }

    async fn suggest_tags(
        &self,
        user_id: UserId,
        prefix: &str,
        limit: i64,
    ) -> Result<Vec<String>, AppError> {
        self.suggest_tags_impl(user_id, prefix, limit).await
    }

    async fn suggest_collections(
        &self,
        user_id: UserId,
        prefix: &str,
        limit: i64,
    ) -> Result<Vec<String>, AppError> {
        self.suggest_collections_impl(user_id, prefix, limit).await
    }

    async fn suggest_senders(
        &self,
        user_id: UserId,
        prefix: &str,
        limit: i64,
    ) -> Result<Vec<String>, AppError> {
        self.suggest_senders_impl(user_id, prefix, limit).await
    }

    async fn suggest_sender_domains(
        &self,
        user_id: UserId,
        prefix: &str,
        limit: i64,
    ) -> Result<Vec<String>, AppError> {
        self.suggest_sender_domains_impl(user_id, prefix, limit)
            .await
    }

    async fn suggest_list_ids(
        &self,
        user_id: UserId,
        prefix: &str,
        limit: i64,
    ) -> Result<Vec<String>, AppError> {
        self.suggest_list_ids_impl(user_id, prefix, limit).await
    }

    async fn suggest_authors(
        &self,
        user_id: UserId,
        prefix: &str,
        limit: i64,
    ) -> Result<Vec<String>, AppError> {
        self.suggest_authors_impl(user_id, prefix, limit).await
    }
}

#[async_trait::async_trait]
impl RecentSearchRepository for PgSearchRepository {
    async fn upsert_recent_search(
        &self,
        user_id: UserId,
        raw_query: &str,
        normalized_query: &str,
        max_entries: i64,
    ) -> Result<RecentSearch, AppError> {
        self.upsert_recent_search_impl(user_id, raw_query, normalized_query, max_entries)
            .await
    }

    async fn list_recent_searches(
        &self,
        user_id: UserId,
        limit: i64,
    ) -> Result<Vec<RecentSearch>, AppError> {
        self.list_recent_searches_impl(user_id, limit).await
    }

    async fn suggest_recent_searches(
        &self,
        user_id: UserId,
        prefix: &str,
        limit: i64,
    ) -> Result<Vec<RecentSearch>, AppError> {
        self.suggest_recent_searches_impl(user_id, prefix, limit)
            .await
    }

    async fn delete_recent_search(
        &self,
        user_id: UserId,
        recent_search_id: RecentSearchId,
    ) -> Result<(), AppError> {
        self.delete_recent_search_impl(user_id, recent_search_id)
            .await
    }

    async fn clear_recent_searches(&self, user_id: UserId) -> Result<(), AppError> {
        self.clear_recent_searches_impl(user_id).await
    }
}
