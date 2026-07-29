use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::error::AppError;
use crate::repos::event::MutationSideEffects;
use ind_domain::{
    DocumentId, Highlight, HighlightId, HighlightNote, NewHighlight, Tag, TagId, UserId,
};

#[async_trait::async_trait]
pub trait HighlightRepository: Send + Sync {
    async fn create(
        &self,
        highlight: &NewHighlight,
        effects: MutationSideEffects,
    ) -> Result<Highlight, AppError>;
    /// Create a document-keyed highlight (`item_id` NULL) and commit any `effects` (the
    /// `document.highlighted` event plus document search reindex / embed outbox rows) atomically
    /// with the annotation.
    async fn create_for_document(
        &self,
        highlight: &NewHighlight,
        effects: MutationSideEffects,
    ) -> Result<Highlight, AppError>;
    async fn list_by_document(
        &self,
        document_id: DocumentId,
        user_id: UserId,
    ) -> Result<Vec<Highlight>, AppError>;
    async fn count_by_document(
        &self,
        document_id: DocumentId,
        user_id: UserId,
    ) -> Result<i64, AppError>;
    async fn get_by_id(
        &self,
        id: HighlightId,
        user_id: UserId,
    ) -> Result<Option<Highlight>, AppError>;
    async fn list_recent_by_user(
        &self,
        user_id: UserId,
        limit: i64,
    ) -> Result<Vec<Highlight>, AppError>;
    async fn update_color(
        &self,
        id: HighlightId,
        user_id: UserId,
        color: &str,
        effects: MutationSideEffects,
    ) -> Result<Highlight, AppError>;
    async fn delete(
        &self,
        id: HighlightId,
        user_id: UserId,
        effects: MutationSideEffects,
    ) -> Result<(), AppError>;
    async fn upsert_note(
        &self,
        highlight_id: HighlightId,
        user_id: UserId,
        body: &str,
        effects: MutationSideEffects,
    ) -> Result<HighlightNote, AppError>;
    async fn delete_note(
        &self,
        highlight_id: HighlightId,
        user_id: UserId,
        effects: MutationSideEffects,
    ) -> Result<(), AppError>;
    async fn get_note(
        &self,
        highlight_id: HighlightId,
        user_id: UserId,
    ) -> Result<Option<HighlightNote>, AppError>;
    async fn add_tag(
        &self,
        highlight_id: HighlightId,
        user_id: UserId,
        tag_id: TagId,
        effects: MutationSideEffects,
    ) -> Result<(), AppError>;
    async fn remove_tag(
        &self,
        highlight_id: HighlightId,
        user_id: UserId,
        tag_id: TagId,
        effects: MutationSideEffects,
    ) -> Result<(), AppError>;
    async fn list_tags(
        &self,
        highlight_id: HighlightId,
        user_id: UserId,
    ) -> Result<Vec<Tag>, AppError>;
    async fn list_tags_for_highlights(
        &self,
        highlight_ids: &[HighlightId],
        user_id: UserId,
    ) -> Result<HashMap<HighlightId, Vec<Tag>>, AppError>;

    /// Cursor-paginated highlights for a document, used by the Notion export job (TASK-236).
    async fn list_by_document_after_cursor(
        &self,
        document_id: DocumentId,
        user_id: UserId,
        after_created_at: Option<DateTime<Utc>>,
        after_id: Option<HighlightId>,
        limit: i64,
    ) -> Result<Vec<Highlight>, AppError>;
}
