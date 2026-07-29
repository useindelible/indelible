use sqlx::PgPool;

use ind_application::AppError;
use ind_application::repos::entity::{EntityDocument, EntityRepository};
use ind_application::repos::{Cursor, Page};
use ind_domain::{DocumentId, Entity, EntityDetail, EntityId, EntitySummary, EntityType, UserId};

mod documents;
mod mutations;
mod reads;
mod resolution;
mod rows;

#[cfg(test)]
mod tests;

pub struct PgEntityRepository {
    pub(super) pool: PgPool,
}

impl PgEntityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

pub(super) fn map_sqlx_error(err: sqlx::Error) -> AppError {
    super::map_sqlx_error("entity", "duplicate entity", err)
}

#[async_trait::async_trait]
impl EntityRepository for PgEntityRepository {
    async fn find_by_id_for_user(
        &self,
        id: EntityId,
        user_id: UserId,
    ) -> Result<Option<Entity>, AppError> {
        self.find_by_id_for_user_impl(id, user_id).await
    }

    async fn list_summaries(
        &self,
        user_id: UserId,
        entity_type: Option<EntityType>,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<EntitySummary>, AppError> {
        self.list_summaries_impl(user_id, entity_type, cursor, limit)
            .await
    }

    async fn get_detail(
        &self,
        user_id: UserId,
        entity_id: EntityId,
        co_occurrence_limit: u32,
    ) -> Result<EntityDetail, AppError> {
        self.get_detail_impl(user_id, entity_id, co_occurrence_limit)
            .await
    }

    async fn list_entity_documents(
        &self,
        user_id: UserId,
        entity_id: EntityId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<EntityDocument>, AppError> {
        self.list_entity_documents_impl(user_id, entity_id, cursor, limit)
            .await
    }

    async fn list_document_ids_for_entity(
        &self,
        user_id: UserId,
        entity_id: EntityId,
    ) -> Result<Vec<DocumentId>, AppError> {
        self.list_document_ids_for_entity_impl(user_id, entity_id)
            .await
    }

    async fn list_entities_for_document(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<Vec<EntitySummary>, AppError> {
        self.list_entities_for_document_impl(user_id, document_id)
            .await
    }

    async fn update_fields(
        &self,
        user_id: UserId,
        entity_id: EntityId,
        name: Option<&str>,
        description: Option<Option<&str>>,
    ) -> Result<Entity, AppError> {
        self.update_fields_impl(user_id, entity_id, name, description)
            .await
    }

    async fn merge_entities(
        &self,
        user_id: UserId,
        source_id: EntityId,
        target_id: EntityId,
    ) -> Result<Entity, AppError> {
        self.merge_entities_impl(user_id, source_id, target_id)
            .await
    }

    async fn set_document_mentions(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        mentions: &[(EntityId, i32)],
    ) -> Result<(), AppError> {
        self.set_document_mentions_impl(user_id, document_id, mentions)
            .await
    }

    async fn find_for_resolution(
        &self,
        user_id: UserId,
        name: &str,
        entity_type: EntityType,
    ) -> Result<Option<Entity>, AppError> {
        self.find_for_resolution_impl(user_id, name, entity_type)
            .await
    }

    async fn block_candidates(
        &self,
        user_id: UserId,
        name: &str,
        entity_type: EntityType,
        limit: i64,
    ) -> Result<Vec<Entity>, AppError> {
        self.block_candidates_impl(user_id, name, entity_type, limit)
            .await
    }

    async fn insert_canonical(
        &self,
        user_id: UserId,
        name: &str,
        entity_type: EntityType,
        description: Option<&str>,
    ) -> Result<Entity, AppError> {
        self.insert_canonical_impl(user_id, name, entity_type, description)
            .await
    }

    async fn insert_alias(
        &self,
        user_id: UserId,
        name: &str,
        entity_type: EntityType,
        entity_id: EntityId,
    ) -> Result<(), AppError> {
        self.insert_alias_impl(user_id, name, entity_type, entity_id)
            .await
    }

    async fn register_alias_if_absent(
        &self,
        user_id: UserId,
        name: &str,
        entity_type: EntityType,
        entity_id: EntityId,
    ) -> Result<(), AppError> {
        self.register_alias_if_absent_impl(user_id, name, entity_type, entity_id)
            .await
    }
}
