use chrono::{DateTime, Utc};

use crate::error::AppError;
use crate::repos::{Cursor, Page};
use ind_domain::{DocumentId, Entity, EntityDetail, EntityId, EntitySummary, EntityType, UserId};

/// Document read-model for an entity's mention listing. Keyed by `document_id`; the field set is
/// exactly what the entity-documents query projects, so it is not a full `Document`.
#[derive(Debug, Clone)]
pub struct EntityDocument {
    pub document_id: DocumentId,
    pub title: String,
    pub author: Option<String>,
    pub excerpt: Option<String>,
    pub domain: Option<String>,
    pub saved_at: DateTime<Utc>,
}

#[async_trait::async_trait]
pub trait EntityRepository: Send + Sync {
    async fn find_by_id_for_user(
        &self,
        id: EntityId,
        user_id: UserId,
    ) -> Result<Option<Entity>, AppError>;

    async fn list_summaries(
        &self,
        user_id: UserId,
        entity_type: Option<EntityType>,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<EntitySummary>, AppError>;

    async fn get_detail(
        &self,
        user_id: UserId,
        entity_id: EntityId,
        co_occurrence_limit: u32,
    ) -> Result<EntityDetail, AppError>;

    async fn list_entity_documents(
        &self,
        user_id: UserId,
        entity_id: EntityId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<EntityDocument>, AppError>;

    async fn list_document_ids_for_entity(
        &self,
        user_id: UserId,
        entity_id: EntityId,
    ) -> Result<Vec<DocumentId>, AppError>;

    async fn list_entities_for_document(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<Vec<EntitySummary>, AppError>;

    async fn update_fields(
        &self,
        user_id: UserId,
        entity_id: EntityId,
        name: Option<&str>,
        description: Option<Option<&str>>,
    ) -> Result<Entity, AppError>;

    async fn merge_entities(
        &self,
        user_id: UserId,
        source_id: EntityId,
        target_id: EntityId,
    ) -> Result<Entity, AppError>;

    /// Set this document's mentions to already-resolved entity ids (counts aggregated per id),
    /// pruning mentions no longer present and cleaning up entities left with no mentions and no
    /// aliases. Replaces the entity-creating `replace_mentions_for_document` write path.
    async fn set_document_mentions(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        mentions: &[(EntityId, i32)],
    ) -> Result<(), AppError>;

    /// Resolve a surface form to its real entity: exact entity by name, else a recorded alias
    /// mapping to one. `None` means the form has never been seen.
    async fn find_for_resolution(
        &self,
        user_id: UserId,
        name: &str,
        entity_type: EntityType,
    ) -> Result<Option<Entity>, AppError>;

    /// Existing same-type entities lexically close to `name` (case-insensitive equality, trigram
    /// similarity, word-subset). High-recall finder for the adjudicator; aliases are never returned.
    async fn block_candidates(
        &self,
        user_id: UserId,
        name: &str,
        entity_type: EntityType,
        limit: i64,
    ) -> Result<Vec<Entity>, AppError>;

    /// Insert a real entity for a name with no existing referent.
    async fn insert_canonical(
        &self,
        user_id: UserId,
        name: &str,
        entity_type: EntityType,
        description: Option<&str>,
    ) -> Result<Entity, AppError>;

    /// Record a confirmed surface-form redirect to `entity_id` (upsert).
    async fn insert_alias(
        &self,
        user_id: UserId,
        name: &str,
        entity_type: EntityType,
        entity_id: EntityId,
    ) -> Result<(), AppError>;

    /// Proactively record an alias only when the name is not already a real entity or alias.
    async fn register_alias_if_absent(
        &self,
        user_id: UserId,
        name: &str,
        entity_type: EntityType,
        entity_id: EntityId,
    ) -> Result<(), AppError>;
}
