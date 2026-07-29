use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateEntityRequest {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
}

pub trait EntityOperations: Send + Sync {
    fn list_entities(
        &self,
        user_id: UserId,
        entity_type: Option<EntityType>,
        cursor: Option<String>,
        limit: Option<u32>,
    ) -> BoxFuture<'_, Result<Page<EntitySummary>, AppError>>;

    fn get_entity(
        &self,
        user_id: UserId,
        entity_id: EntityId,
    ) -> BoxFuture<'_, Result<EntityDetail, AppError>>;

    fn list_entity_documents(
        &self,
        user_id: UserId,
        entity_id: EntityId,
        cursor: Option<String>,
        limit: Option<u32>,
    ) -> BoxFuture<'_, Result<Page<crate::repos::entity::EntityDocument>, AppError>>;

    fn list_entities_for_document(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> BoxFuture<'_, Result<Vec<EntitySummary>, AppError>>;

    fn update_entity(
        &self,
        user_id: UserId,
        entity_id: EntityId,
        request: UpdateEntityRequest,
    ) -> BoxFuture<'_, Result<EntityDetail, AppError>>;

    fn merge_entity(
        &self,
        user_id: UserId,
        source_id: EntityId,
        target_id: EntityId,
    ) -> BoxFuture<'_, Result<EntityDetail, AppError>>;
}
