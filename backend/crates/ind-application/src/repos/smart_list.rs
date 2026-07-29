use crate::error::AppError;
use crate::repos::{Cursor, Page};
use ind_domain::{FilterNode, LibraryEntryWithDocument, SmartList, SmartListId, UserId};

#[async_trait::async_trait]
pub trait SmartListRepository: Send + Sync {
    async fn find_by_id(
        &self,
        id: SmartListId,
        user_id: UserId,
    ) -> Result<Option<SmartList>, AppError>;
    async fn create(&self, smart_list: SmartList) -> Result<SmartList, AppError>;
    async fn update_name(
        &self,
        id: SmartListId,
        user_id: UserId,
        name: &str,
    ) -> Result<SmartList, AppError>;
    async fn update_filter(
        &self,
        id: SmartListId,
        user_id: UserId,
        filter: &FilterNode,
    ) -> Result<SmartList, AppError>;
    async fn update_pin(
        &self,
        id: SmartListId,
        user_id: UserId,
        is_pinned: bool,
    ) -> Result<SmartList, AppError>;
    async fn update_metadata(
        &self,
        id: SmartListId,
        user_id: UserId,
        icon: Option<&str>,
        color: Option<&str>,
        default_sort: Option<&str>,
    ) -> Result<SmartList, AppError>;
    async fn delete(&self, id: SmartListId, user_id: UserId) -> Result<(), AppError>;
    async fn list_by_user(
        &self,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<SmartList>, AppError>;
    async fn list_pinned(&self, user_id: UserId) -> Result<Vec<SmartList>, AppError>;

    async fn evaluate_filter(
        &self,
        user_id: UserId,
        filter: &FilterNode,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<LibraryEntryWithDocument>, AppError>;
}
