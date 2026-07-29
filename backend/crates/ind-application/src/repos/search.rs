use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::error::AppError;
use ind_domain::{
    ContentVector, DocumentId, RecentSearch, RecentSearchId, SearchDocument, SearchEntityCard,
    SearchEntityChip, SearchHit, SearchIndexedHighlight, UserId,
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SearchFtsQuery {
    pub user_id: UserId,
    pub text_query: Option<String>,
    pub tag_values: Vec<String>,
    pub negated_tag_values: Vec<String>,
    pub collection_values: Vec<String>,
    pub negated_collection_values: Vec<String>,
    pub type_values: Vec<String>,
    pub negated_type_values: Vec<String>,
    pub author_values: Vec<String>,
    pub negated_author_values: Vec<String>,
    pub url_values: Vec<String>,
    pub negated_url_values: Vec<String>,
    pub entity_values: Vec<String>,
    pub negated_entity_values: Vec<String>,
    pub sender_values: Vec<String>,
    pub negated_sender_values: Vec<String>,
    pub sender_domain_values: Vec<String>,
    pub negated_sender_domain_values: Vec<String>,
    pub list_values: Vec<String>,
    pub negated_list_values: Vec<String>,
    pub subject_values: Vec<String>,
    pub negated_subject_values: Vec<String>,
    pub before_saved_at: Option<DateTime<Utc>>,
    pub after_saved_at: Option<DateTime<Utc>>,
    pub require_read: bool,
    pub exclude_read: bool,
    pub require_unread: bool,
    pub exclude_unread: bool,
    pub require_archived: bool,
    pub exclude_archived: bool,
    pub require_favorited: bool,
    pub exclude_favorited: bool,
    pub require_has_highlights: bool,
    pub exclude_has_highlights: bool,
    pub require_has_notes: bool,
    pub exclude_has_notes: bool,
    pub require_has_unsubscribe: bool,
    pub exclude_has_unsubscribe: bool,
    pub require_pinned: bool,
    pub exclude_pinned: bool,
    pub require_sender_blocked: bool,
    pub exclude_sender_blocked: bool,
    pub require_feed_only: bool,
    pub exclude_feed_only: bool,
    pub score_reference_at: DateTime<Utc>,
    pub cursor_score: Option<f64>,
    pub cursor_saved_at: Option<DateTime<Utc>>,
    pub cursor_result_id: Option<Uuid>,
    pub cursor_section_key: Option<String>,
    pub limit: i64,
}

#[async_trait::async_trait]
pub trait SearchRepository: Send + Sync {
    async fn upsert_search_document(
        &self,
        document: &SearchDocument,
    ) -> Result<SearchDocument, AppError>;

    /// Replace all durable search rows for a document (root + section rows).
    async fn replace_search_documents_for_document(
        &self,
        document_id: DocumentId,
        documents: &[SearchDocument],
    ) -> Result<(), AppError>;

    async fn delete_search_documents_for_document(
        &self,
        document_id: DocumentId,
    ) -> Result<(), AppError>;

    async fn upsert_content_vector(
        &self,
        vector: &ContentVector,
    ) -> Result<ContentVector, AppError>;

    async fn search_fts(&self, query: &SearchFtsQuery) -> Result<Vec<SearchHit>, AppError>;

    async fn get_document_note_text(
        &self,
        document_id: DocumentId,
    ) -> Result<Option<String>, AppError>;

    async fn list_highlights_for_document(
        &self,
        document_id: DocumentId,
    ) -> Result<Vec<SearchIndexedHighlight>, AppError>;

    async fn list_entity_chips_for_documents(
        &self,
        user_id: UserId,
        document_ids: &[DocumentId],
    ) -> Result<HashMap<DocumentId, Vec<SearchEntityChip>>, AppError>;

    async fn suggest_entities(
        &self,
        user_id: UserId,
        prefix: &str,
        limit: i64,
    ) -> Result<Vec<SearchEntityChip>, AppError>;

    async fn find_entity_card(
        &self,
        user_id: UserId,
        query: &str,
    ) -> Result<Option<SearchEntityCard>, AppError>;

    async fn suggest_tags(
        &self,
        user_id: UserId,
        prefix: &str,
        limit: i64,
    ) -> Result<Vec<String>, AppError>;

    async fn suggest_collections(
        &self,
        user_id: UserId,
        prefix: &str,
        limit: i64,
    ) -> Result<Vec<String>, AppError>;

    async fn suggest_senders(
        &self,
        user_id: UserId,
        prefix: &str,
        limit: i64,
    ) -> Result<Vec<String>, AppError>;

    async fn suggest_sender_domains(
        &self,
        user_id: UserId,
        prefix: &str,
        limit: i64,
    ) -> Result<Vec<String>, AppError>;

    async fn suggest_list_ids(
        &self,
        user_id: UserId,
        prefix: &str,
        limit: i64,
    ) -> Result<Vec<String>, AppError>;

    async fn suggest_authors(
        &self,
        user_id: UserId,
        prefix: &str,
        limit: i64,
    ) -> Result<Vec<String>, AppError>;
}

#[async_trait::async_trait]
pub trait RecentSearchRepository: Send + Sync {
    async fn upsert_recent_search(
        &self,
        user_id: UserId,
        raw_query: &str,
        normalized_query: &str,
        max_entries: i64,
    ) -> Result<RecentSearch, AppError>;

    async fn list_recent_searches(
        &self,
        user_id: UserId,
        limit: i64,
    ) -> Result<Vec<RecentSearch>, AppError>;

    async fn suggest_recent_searches(
        &self,
        user_id: UserId,
        prefix: &str,
        limit: i64,
    ) -> Result<Vec<RecentSearch>, AppError>;

    async fn delete_recent_search(
        &self,
        user_id: UserId,
        recent_search_id: RecentSearchId,
    ) -> Result<(), AppError>;

    async fn clear_recent_searches(&self, user_id: UserId) -> Result<(), AppError>;
}
