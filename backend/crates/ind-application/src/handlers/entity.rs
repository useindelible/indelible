use std::sync::Arc;

use futures::future::BoxFuture;
use ind_domain::{
    DocumentId, DomainError, EntityDetail, EntityId, EntitySummary, EntityType, UserId,
};

use crate::AppError;
use crate::ports::{EntityOperations, UpdateEntityRequest};
use crate::repos::entity::{EntityDocument, EntityRepository};
use crate::repos::outbox::JobOutboxRepository;
use crate::repos::{Cursor, Page};

pub struct UpdateEntityInput {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
}

pub struct EntityService {
    entity_repo: Arc<dyn EntityRepository>,
}

impl EntityService {
    pub fn new(entity_repo: Arc<dyn EntityRepository>) -> Self {
        Self { entity_repo }
    }

    pub async fn list(
        &self,
        user_id: UserId,
        entity_type: Option<EntityType>,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<EntitySummary>, AppError> {
        self.entity_repo
            .list_summaries(user_id, entity_type, cursor, limit)
            .await
    }

    pub async fn get(
        &self,
        user_id: UserId,
        entity_id: EntityId,
    ) -> Result<EntityDetail, AppError> {
        self.entity_repo.get_detail(user_id, entity_id, 10).await
    }

    pub async fn list_for_document(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<Vec<EntitySummary>, AppError> {
        self.entity_repo
            .list_entities_for_document(user_id, document_id)
            .await
    }

    pub async fn list_documents(
        &self,
        user_id: UserId,
        entity_id: EntityId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<EntityDocument>, AppError> {
        self.entity_repo
            .find_by_id_for_user(entity_id, user_id)
            .await?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "entity",
                    id: entity_id.to_string(),
                })
            })?;

        self.entity_repo
            .list_entity_documents(user_id, entity_id, cursor, limit)
            .await
    }

    pub async fn update(
        &self,
        user_id: UserId,
        entity_id: EntityId,
        input: UpdateEntityInput,
    ) -> Result<EntityDetail, AppError> {
        if input.name.is_none() && input.description.is_none() {
            return Err(AppError::Domain(DomainError::Validation {
                field: "_".into(),
                message: "at least one field must be provided".into(),
            }));
        }

        let name = if let Some(name) = input.name {
            let trimmed = name.trim().to_string();
            if trimmed.is_empty() {
                return Err(AppError::Domain(DomainError::Validation {
                    field: "name".into(),
                    message: "must not be empty".into(),
                }));
            }
            Some(trimmed)
        } else {
            None
        };

        let description = input.description.map(|description| {
            description
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        });

        self.entity_repo
            .update_fields(
                user_id,
                entity_id,
                name.as_deref(),
                description.as_ref().map(|value| value.as_deref()),
            )
            .await?;
        self.entity_repo.get_detail(user_id, entity_id, 10).await
    }

    pub async fn merge(
        &self,
        user_id: UserId,
        source_id: EntityId,
        target_id: EntityId,
    ) -> Result<EntityDetail, AppError> {
        if source_id == target_id {
            return Err(AppError::Domain(DomainError::Validation {
                field: "target_id".into(),
                message: "target entity must differ from source entity".into(),
            }));
        }

        self.entity_repo
            .find_by_id_for_user(source_id, user_id)
            .await?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "entity",
                    id: source_id.to_string(),
                })
            })?;

        self.entity_repo
            .find_by_id_for_user(target_id, user_id)
            .await?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "entity",
                    id: target_id.to_string(),
                })
            })?;

        self.entity_repo
            .merge_entities(user_id, source_id, target_id)
            .await?;
        self.entity_repo.get_detail(user_id, target_id, 10).await
    }
}

pub struct EntityOperationsService {
    service: EntityService,
    entity_repo: Arc<dyn EntityRepository>,
    outbox_repo: Arc<dyn JobOutboxRepository>,
}

impl EntityOperationsService {
    pub fn new(
        entity_repo: Arc<dyn EntityRepository>,
        outbox_repo: Arc<dyn JobOutboxRepository>,
    ) -> Self {
        Self {
            service: EntityService::new(entity_repo.clone()),
            entity_repo,
            outbox_repo,
        }
    }

    async fn enqueue_reindex(
        &self,
        document_ids: impl IntoIterator<Item = DocumentId>,
    ) -> Result<(), AppError> {
        let mut seen = std::collections::HashSet::new();
        for document_id in document_ids {
            if !seen.insert(document_id) {
                continue;
            }
            let payload =
                serde_json::to_value(ind_domain::SearchReindexDocumentJob { document_id })
                    .map_err(|error| AppError::Repository(Box::new(error)))?;
            self.outbox_repo
                .enqueue(
                    ind_domain::job_types::SEARCH_REINDEX_DOCUMENT,
                    payload,
                    Some(format!(
                        "{}:{document_id}",
                        ind_domain::job_types::SEARCH_REINDEX_DOCUMENT
                    )),
                    chrono::Utc::now(),
                )
                .await?;
        }
        Ok(())
    }
}

impl EntityOperations for EntityOperationsService {
    fn list_entities(
        &self,
        user_id: UserId,
        entity_type: Option<EntityType>,
        cursor: Option<String>,
        limit: Option<u32>,
    ) -> BoxFuture<'_, Result<Page<EntitySummary>, AppError>> {
        Box::pin(async move {
            self.service
                .list(
                    user_id,
                    entity_type,
                    cursor.map(Cursor),
                    limit.unwrap_or(50),
                )
                .await
        })
    }

    fn get_entity(
        &self,
        user_id: UserId,
        entity_id: EntityId,
    ) -> BoxFuture<'_, Result<EntityDetail, AppError>> {
        Box::pin(self.service.get(user_id, entity_id))
    }

    fn list_entity_documents(
        &self,
        user_id: UserId,
        entity_id: EntityId,
        cursor: Option<String>,
        limit: Option<u32>,
    ) -> BoxFuture<'_, Result<Page<EntityDocument>, AppError>> {
        Box::pin(async move {
            self.service
                .list_documents(user_id, entity_id, cursor.map(Cursor), limit.unwrap_or(50))
                .await
        })
    }

    fn list_entities_for_document(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> BoxFuture<'_, Result<Vec<EntitySummary>, AppError>> {
        Box::pin(self.service.list_for_document(user_id, document_id))
    }

    fn update_entity(
        &self,
        user_id: UserId,
        entity_id: EntityId,
        request: UpdateEntityRequest,
    ) -> BoxFuture<'_, Result<EntityDetail, AppError>> {
        Box::pin(async move {
            let affected = self
                .entity_repo
                .list_document_ids_for_entity(user_id, entity_id)
                .await?;
            let detail = self
                .service
                .update(
                    user_id,
                    entity_id,
                    UpdateEntityInput {
                        name: request.name,
                        description: request.description,
                    },
                )
                .await?;
            self.enqueue_reindex(affected).await?;
            Ok(detail)
        })
    }

    fn merge_entity(
        &self,
        user_id: UserId,
        source_id: EntityId,
        target_id: EntityId,
    ) -> BoxFuture<'_, Result<EntityDetail, AppError>> {
        Box::pin(async move {
            let mut affected = self
                .entity_repo
                .list_document_ids_for_entity(user_id, source_id)
                .await?;
            affected.extend(
                self.entity_repo
                    .list_document_ids_for_entity(user_id, target_id)
                    .await?,
            );
            let detail = self.service.merge(user_id, source_id, target_id).await?;
            self.enqueue_reindex(affected).await?;
            Ok(detail)
        })
    }
}
