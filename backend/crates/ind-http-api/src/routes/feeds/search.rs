use super::{
    FeedSearchResponse, FeedSourceResponse, SearchFeedSourcesParams, dto, require_feed_ops,
};
use crate::error::ApiError;
use crate::middleware::AccountAccess;
use crate::state::AppState;
use axum::extract::{Query, State};

#[utoipa::path(
    get,
    path = "/api/v1/feeds/search",
    params(SearchFeedSourcesParams),
    responses(
        (status = 200, description = "Public feed search results", body = FeedSearchResponse),
        (status = 401, description = "Authentication required"),
        (status = 422, description = "Validation error"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Feeds",
)]
pub async fn search_sources(
    AccountAccess(_auth_user): AccountAccess,
    State(state): State<AppState>,
    Query(params): Query<SearchFeedSourcesParams>,
) -> Result<crate::extract::Json<FeedSearchResponse>, ApiError> {
    let feed_ops = require_feed_ops(&state)?;
    if params.query.trim().is_empty() {
        return Err(ApiError::ValidationError {
            errors: vec![crate::error::FieldError {
                field: "query".into(),
                message: "must not be empty".into(),
            }],
        });
    }

    let surface = match params.surface.as_deref() {
        None => ind_domain::FeedSearchSurface::All,
        Some(surface) => {
            dto::parse_feed_search_surface(surface).ok_or_else(|| ApiError::ValidationError {
                errors: vec![crate::error::FieldError {
                    field: "surface".into(),
                    message: format!(
                        "must be one of: {}",
                        dto::valid_feed_search_surfaces().join(", ")
                    ),
                }],
            })?
        }
    };

    let items = feed_ops
        .search_public_sources(params.query, surface, params.limit)
        .await
        .map_err(ApiError::from)?;

    Ok(crate::extract::Json(FeedSearchResponse {
        items: items
            .into_iter()
            .map(FeedSourceResponse::from_domain)
            .collect(),
    }))
}
