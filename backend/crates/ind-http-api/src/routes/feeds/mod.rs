pub(crate) mod dto;

use axum::Router;
use axum::routing::{get, post};
use ind_application::ports::FeedOperations;

use crate::error::ApiError;
use crate::state::AppState;

mod search;
mod subscriptions;

pub use search::{__path_search_sources, search_sources};
pub use subscriptions::{
    __path_import_opml, __path_list_subscriptions, __path_retry_subscription, __path_subscribe,
    __path_unsubscribe, __path_update_subscription, OpmlUploadSchema, import_opml,
    list_subscriptions, retry_subscription, subscribe, unsubscribe, update_subscription,
};

pub(crate) use dto::{
    FeedSearchResponse, FeedSourceResponse, FeedSubscriptionResponse, ListSubscriptionsParams,
    OpmlImportResponse, SearchFeedSourcesParams, SubscribeBody, SubscribeResponse,
    UpdateSubscriptionBody,
};

fn require_feed_ops(state: &AppState) -> Result<&dyn FeedOperations, ApiError> {
    state
        .feed_ops
        .as_deref()
        .ok_or(ApiError::ServiceUnavailable {
            message: "feed service not configured".into(),
        })
}

pub fn feed_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/feeds/search", get(search_sources))
        .route(
            "/api/v1/feeds/subscriptions",
            post(subscribe).get(list_subscriptions),
        )
        .route("/api/v1/feeds/subscriptions/opml", post(import_opml))
        .route(
            "/api/v1/feeds/subscriptions/{id}",
            axum::routing::patch(update_subscription).delete(unsubscribe),
        )
        .route(
            "/api/v1/feeds/subscriptions/{id}/retry",
            post(retry_subscription),
        )
}
