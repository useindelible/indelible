use super::*;

pub struct CreateSmartListRequest {
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub filter_expression: FilterNode,
    pub default_sort: Option<String>,
}

pub struct UpdateSmartListRequest {
    pub name: Option<String>,
    pub icon: Option<Option<String>>,
    pub color: Option<Option<String>>,
    pub filter_expression: Option<FilterNode>,
    pub default_sort: Option<Option<String>>,
    pub is_pinned: Option<bool>,
}

pub trait SmartListOperations: Send + Sync {
    fn create_smart_list(
        &self,
        user_id: UserId,
        req: CreateSmartListRequest,
    ) -> BoxFuture<'_, Result<SmartList, AppError>>;

    fn get_smart_list(
        &self,
        user_id: UserId,
        id: SmartListId,
    ) -> BoxFuture<'_, Result<SmartList, AppError>>;

    fn list_smart_lists(
        &self,
        user_id: UserId,
        cursor: Option<String>,
        limit: Option<u32>,
    ) -> BoxFuture<'_, Result<Page<SmartList>, AppError>>;

    fn update_smart_list(
        &self,
        user_id: UserId,
        id: SmartListId,
        req: UpdateSmartListRequest,
    ) -> BoxFuture<'_, Result<SmartList, AppError>>;

    fn delete_smart_list(
        &self,
        user_id: UserId,
        id: SmartListId,
    ) -> BoxFuture<'_, Result<(), AppError>>;

    fn evaluate_smart_list_entries(
        &self,
        user_id: UserId,
        id: SmartListId,
        cursor: Option<String>,
        limit: Option<u32>,
    ) -> BoxFuture<'_, Result<Page<LibraryEntryWithDocument>, AppError>>;

    fn evaluate_library_filter(
        &self,
        user_id: UserId,
        filter_expression: ind_domain::FilterNode,
        cursor: Option<String>,
        limit: Option<u32>,
    ) -> BoxFuture<'_, Result<Page<LibraryEntryWithDocument>, AppError>>;

    fn pin_smart_list(
        &self,
        user_id: UserId,
        id: SmartListId,
        is_pinned: bool,
    ) -> BoxFuture<'_, Result<SmartList, AppError>>;
}
