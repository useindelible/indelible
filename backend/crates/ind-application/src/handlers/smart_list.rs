use std::sync::Arc;

use chrono::Utc;
use futures::future::BoxFuture;
use ind_domain::{
    DomainError, FilterNode, LibraryEntryWithDocument, SmartList, SmartListId, UserId,
};

use crate::AppError;
use crate::ports::{CreateSmartListRequest, SmartListOperations, UpdateSmartListRequest};
use crate::repos::smart_list::SmartListRepository;
use crate::repos::{Cursor, Page};

pub struct CreateSmartListInput {
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub filter_expression: FilterNode,
    pub default_sort: Option<String>,
}

pub struct UpdateSmartListInput {
    pub name: Option<String>,
    pub icon: Option<Option<String>>,
    pub color: Option<Option<String>>,
    pub filter_expression: Option<FilterNode>,
    pub default_sort: Option<Option<String>>,
    pub is_pinned: Option<bool>,
}

pub struct SmartListService {
    smart_list_repo: Arc<dyn SmartListRepository>,
}

impl SmartListService {
    pub fn new(smart_list_repo: Arc<dyn SmartListRepository>) -> Self {
        Self { smart_list_repo }
    }

    pub async fn create(
        &self,
        user_id: UserId,
        input: CreateSmartListInput,
    ) -> Result<SmartList, AppError> {
        ensure_filter_expression(&input.filter_expression)?;

        let now = Utc::now();
        let smart_list = SmartList {
            id: SmartListId::new(),
            user_id,
            name: input.name,
            icon: input.icon,
            color: input.color,
            is_pinned: false,
            filter_expression: input.filter_expression,
            default_sort: input.default_sort,
            created_at: now,
            updated_at: now,
        };

        self.smart_list_repo.create(smart_list).await
    }

    pub async fn get(&self, user_id: UserId, id: SmartListId) -> Result<SmartList, AppError> {
        self.smart_list_repo
            .find_by_id(id, user_id)
            .await?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "smart_list",
                    id: id.to_string(),
                })
            })
    }

    pub async fn list(
        &self,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<SmartList>, AppError> {
        self.smart_list_repo
            .list_by_user(user_id, cursor, limit)
            .await
    }

    pub async fn update(
        &self,
        user_id: UserId,
        id: SmartListId,
        input: UpdateSmartListInput,
    ) -> Result<SmartList, AppError> {
        let mut smart_list = self.get(user_id, id).await?;

        if let Some(ref name) = input.name {
            smart_list = self.smart_list_repo.update_name(id, user_id, name).await?;
        }
        if let Some(ref filter) = input.filter_expression {
            ensure_filter_expression(filter)?;
            smart_list = self
                .smart_list_repo
                .update_filter(id, user_id, filter)
                .await?;
        }
        if let Some(is_pinned) = input.is_pinned {
            smart_list = self
                .smart_list_repo
                .update_pin(id, user_id, is_pinned)
                .await?;
        }

        let needs_metadata_update =
            input.icon.is_some() || input.color.is_some() || input.default_sort.is_some();
        if needs_metadata_update {
            let icon = match &input.icon {
                Some(v) => v.as_deref(),
                None => smart_list.icon.as_deref(),
            };
            let color = match &input.color {
                Some(v) => v.as_deref(),
                None => smart_list.color.as_deref(),
            };
            let default_sort = match &input.default_sort {
                Some(v) => v.as_deref(),
                None => smart_list.default_sort.as_deref(),
            };
            smart_list = self
                .smart_list_repo
                .update_metadata(id, user_id, icon, color, default_sort)
                .await?;
        }

        Ok(smart_list)
    }

    pub async fn delete(&self, user_id: UserId, id: SmartListId) -> Result<(), AppError> {
        self.smart_list_repo.delete(id, user_id).await
    }

    pub async fn pin(
        &self,
        user_id: UserId,
        id: SmartListId,
        is_pinned: bool,
    ) -> Result<SmartList, AppError> {
        self.get(user_id, id).await?;
        self.smart_list_repo
            .update_pin(id, user_id, is_pinned)
            .await
    }

    pub async fn evaluate_items(
        &self,
        user_id: UserId,
        id: SmartListId,
        cursor: Option<String>,
        limit: Option<u32>,
    ) -> Result<Page<LibraryEntryWithDocument>, AppError> {
        let smart_list = self.get(user_id, id).await?;

        let page_limit = limit.unwrap_or(50).min(200);
        let cursor = cursor.map(Cursor);

        self.smart_list_repo
            .evaluate_filter(user_id, &smart_list.filter_expression, cursor, page_limit)
            .await
    }

    /// Ad-hoc library filtering (type pages, filter panel) through the same engine as
    /// saved smart lists — the expression just isn't persisted.
    pub async fn evaluate_ad_hoc(
        &self,
        user_id: UserId,
        filter_expression: FilterNode,
        cursor: Option<String>,
        limit: Option<u32>,
    ) -> Result<Page<LibraryEntryWithDocument>, AppError> {
        ensure_filter_expression(&filter_expression)?;

        let page_limit = limit.unwrap_or(50).min(200);
        let cursor = cursor.map(Cursor);

        self.smart_list_repo
            .evaluate_filter(user_id, &filter_expression, cursor, page_limit)
            .await
    }
}

impl SmartListOperations for SmartListService {
    fn create_smart_list(
        &self,
        user_id: UserId,
        request: CreateSmartListRequest,
    ) -> BoxFuture<'_, Result<SmartList, AppError>> {
        Box::pin(self.create(
            user_id,
            CreateSmartListInput {
                name: request.name,
                icon: request.icon,
                color: request.color,
                filter_expression: request.filter_expression,
                default_sort: request.default_sort,
            },
        ))
    }

    fn get_smart_list(
        &self,
        user_id: UserId,
        id: SmartListId,
    ) -> BoxFuture<'_, Result<SmartList, AppError>> {
        Box::pin(self.get(user_id, id))
    }

    fn list_smart_lists(
        &self,
        user_id: UserId,
        cursor: Option<String>,
        limit: Option<u32>,
    ) -> BoxFuture<'_, Result<Page<SmartList>, AppError>> {
        Box::pin(self.list(user_id, cursor.map(Cursor), limit.unwrap_or(50)))
    }

    fn update_smart_list(
        &self,
        user_id: UserId,
        id: SmartListId,
        request: UpdateSmartListRequest,
    ) -> BoxFuture<'_, Result<SmartList, AppError>> {
        Box::pin(self.update(
            user_id,
            id,
            UpdateSmartListInput {
                name: request.name,
                icon: request.icon,
                color: request.color,
                filter_expression: request.filter_expression,
                default_sort: request.default_sort,
                is_pinned: request.is_pinned,
            },
        ))
    }

    fn delete_smart_list(
        &self,
        user_id: UserId,
        id: SmartListId,
    ) -> BoxFuture<'_, Result<(), AppError>> {
        Box::pin(self.delete(user_id, id))
    }

    fn evaluate_smart_list_entries(
        &self,
        user_id: UserId,
        id: SmartListId,
        cursor: Option<String>,
        limit: Option<u32>,
    ) -> BoxFuture<'_, Result<Page<LibraryEntryWithDocument>, AppError>> {
        Box::pin(self.evaluate_items(user_id, id, cursor, limit))
    }

    fn evaluate_library_filter(
        &self,
        user_id: UserId,
        filter_expression: FilterNode,
        cursor: Option<String>,
        limit: Option<u32>,
    ) -> BoxFuture<'_, Result<Page<LibraryEntryWithDocument>, AppError>> {
        Box::pin(self.evaluate_ad_hoc(user_id, filter_expression, cursor, limit))
    }

    fn pin_smart_list(
        &self,
        user_id: UserId,
        id: SmartListId,
        is_pinned: bool,
    ) -> BoxFuture<'_, Result<SmartList, AppError>> {
        Box::pin(self.pin(user_id, id, is_pinned))
    }
}

fn ensure_filter_expression(node: &FilterNode) -> Result<(), AppError> {
    node.validate().map_err(|message| {
        AppError::Domain(DomainError::Validation {
            field: "filter_expression".into(),
            message,
        })
    })
}
