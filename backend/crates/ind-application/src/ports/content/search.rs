use super::*;

pub trait SearchOperations: Send + Sync {
    fn consume_search_limit(
        &self,
        user_id: UserId,
    ) -> BoxFuture<'_, Result<SearchRateLimitStatus, AppError>>;

    fn consume_suggestions_limit(
        &self,
        user_id: UserId,
    ) -> BoxFuture<'_, Result<SearchRateLimitStatus, AppError>>;

    fn search(
        &self,
        user_id: UserId,
        query: String,
        cursor: Option<String>,
        limit: u32,
    ) -> BoxFuture<'_, Result<SearchPage, AppError>>;

    fn suggestions(
        &self,
        user_id: UserId,
        query: String,
        limit: u32,
    ) -> BoxFuture<'_, Result<Vec<SearchSuggestion>, AppError>>;

    fn list_recent_searches(
        &self,
        user_id: UserId,
        limit: i64,
    ) -> BoxFuture<'_, Result<Vec<RecentSearch>, AppError>>;

    fn clear_recent_searches(&self, user_id: UserId) -> BoxFuture<'_, Result<(), AppError>>;

    fn delete_recent_search(
        &self,
        user_id: UserId,
        recent_search_id: RecentSearchId,
    ) -> BoxFuture<'_, Result<(), AppError>>;
}
