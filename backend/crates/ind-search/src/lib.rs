#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::unimplemented,
        clippy::todo
    )
)]

mod engine;
mod entity;
mod indexer;
mod metadata;
mod query;
mod rate_limit;
mod suggesters;

#[cfg(test)]
mod tests;

pub use engine::SearchEngine;
pub use indexer::SearchIndexer;
pub use rate_limit::{SearchRateLimitDefaults, SearchRateLimiter};

use futures::future::BoxFuture;
use ind_application::AppError;
use ind_application::ports::SearchOperations;
use ind_domain::{
    RecentSearch, RecentSearchId, SearchPage, SearchRateLimitStatus, SearchSuggestion, UserId,
};

pub struct SearchService {
    engine: SearchEngine,
    rate_limiter: SearchRateLimiter,
}

impl SearchService {
    pub fn new(engine: SearchEngine, rate_limiter: SearchRateLimiter) -> Self {
        Self {
            engine,
            rate_limiter,
        }
    }
}

impl SearchOperations for SearchService {
    fn consume_search_limit(
        &self,
        user_id: UserId,
    ) -> BoxFuture<'_, Result<SearchRateLimitStatus, AppError>> {
        Box::pin(self.rate_limiter.consume_search(user_id))
    }

    fn consume_suggestions_limit(
        &self,
        user_id: UserId,
    ) -> BoxFuture<'_, Result<SearchRateLimitStatus, AppError>> {
        Box::pin(self.rate_limiter.consume_suggestions(user_id))
    }

    fn search(
        &self,
        user_id: UserId,
        query: String,
        cursor: Option<String>,
        limit: u32,
    ) -> BoxFuture<'_, Result<SearchPage, AppError>> {
        Box::pin(async move {
            self.engine
                .search(user_id, &query, cursor.as_deref(), limit)
                .await
        })
    }

    fn suggestions(
        &self,
        user_id: UserId,
        query: String,
        limit: u32,
    ) -> BoxFuture<'_, Result<Vec<SearchSuggestion>, AppError>> {
        Box::pin(async move { self.engine.suggestions(user_id, &query, limit).await })
    }

    fn list_recent_searches(
        &self,
        user_id: UserId,
        limit: i64,
    ) -> BoxFuture<'_, Result<Vec<RecentSearch>, AppError>> {
        Box::pin(self.engine.list_recent_searches(user_id, limit))
    }

    fn clear_recent_searches(&self, user_id: UserId) -> BoxFuture<'_, Result<(), AppError>> {
        Box::pin(self.engine.clear_recent_searches(user_id))
    }

    fn delete_recent_search(
        &self,
        user_id: UserId,
        recent_search_id: RecentSearchId,
    ) -> BoxFuture<'_, Result<(), AppError>> {
        Box::pin(self.engine.delete_recent_search(user_id, recent_search_id))
    }
}

pub const SEARCH_INDEX_VERSION: i32 = 2;
