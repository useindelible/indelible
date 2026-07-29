//! Feed delivery HTTP surface for the document/feed/library model.
//!
//! Lists query `feed_deliveries JOIN feed_source_entries LEFT JOIN documents LEFT JOIN
//! library_entries`, so unprepared deliveries (`document_id = NULL`) still render and saved
//! documents are excluded. Seen/dismiss/mark-all-seen mutate only `feed_deliveries` and never
//! materialize a document or enqueue a renderer job. Saving a delivery goes through the Library
//! surface (`POST /api/v1/library/from-delivery`).

pub(crate) mod dto;

use axum::Router;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use ind_application::ports::FeedDeliveryOperations;
use ind_application::repos::{Cursor, Page};
use ind_domain::{FeedDeliveryId, FeedSubscriptionId};

use crate::error::{ApiError, FieldError};
use crate::extract::ValidatedJson;
use crate::middleware::AccountAccess;
use crate::response::{ApiResponse, PaginatedResponse};
use crate::state::AppState;

pub(crate) mod core;
pub(crate) mod mutations;
pub(crate) mod prepare;

pub use core::{get_feed_delivery, get_feed_delivery_stats, list_feed_deliveries};
pub use dto::*;
pub use mutations::{dismiss_delivery, mark_all_deliveries_seen, mark_delivery_seen};
pub use prepare::{prepare_feed_delivery, prepare_feed_read_ahead};

fn require_feed_delivery_ops(state: &AppState) -> Result<&dyn FeedDeliveryOperations, ApiError> {
    state
        .feed_delivery_ops
        .as_deref()
        .ok_or(ApiError::ServiceUnavailable {
            message: "feed delivery service not configured".into(),
        })
}

fn parse_delivery_id(raw: &str) -> Result<FeedDeliveryId, ApiError> {
    raw.parse().map_err(|_| ApiError::NotFound {
        entity: "FeedDelivery",
        id: raw.to_string(),
    })
}

fn parse_subscription_id(raw: Option<&str>) -> Result<Option<FeedSubscriptionId>, ApiError> {
    match raw {
        None => Ok(None),
        Some(raw) => {
            raw.parse::<FeedSubscriptionId>()
                .map(Some)
                .map_err(|_| ApiError::ValidationError {
                    errors: vec![FieldError {
                        field: "subscription_id".into(),
                        message: "invalid feed subscription id".into(),
                    }],
                })
        }
    }
}

pub fn feed_delivery_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/feeds/deliveries", get(list_feed_deliveries))
        .route(
            "/api/v1/feeds/deliveries/stats",
            get(get_feed_delivery_stats),
        )
        .route(
            "/api/v1/feeds/deliveries/mark-all-seen",
            post(mark_all_deliveries_seen),
        )
        .route(
            "/api/v1/feeds/deliveries/read-ahead",
            post(prepare_feed_read_ahead),
        )
        .route(
            "/api/v1/feeds/deliveries/{delivery_id}",
            get(get_feed_delivery),
        )
        .route(
            "/api/v1/feeds/deliveries/{delivery_id}/seen",
            post(mark_delivery_seen),
        )
        .route(
            "/api/v1/feeds/deliveries/{delivery_id}/dismiss",
            post(dismiss_delivery),
        )
        .route(
            "/api/v1/feeds/deliveries/{delivery_id}/prepare",
            post(prepare_feed_delivery),
        )
}
