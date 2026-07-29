pub(crate) mod dto;

use axum::Router;
use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use http::HeaderMap;
use http::header::{HeaderName, HeaderValue, RETRY_AFTER};
use ind_application::ports::SearchOperations;

use crate::error::{ApiError, FieldError};
use crate::middleware::AccountAccess;
use crate::state::AppState;

pub(crate) use dto::{
    RecentSearchListResponse, RecentSearchResponse, SearchEmbeddedSenderResponse,
    SearchEntityCardResponse, SearchEntityChipResponse, SearchParams, SearchRecentParams,
    SearchResultResponse, SearchResultsResponse, SearchSectionResponse, SearchSuggestionResponse,
    SearchSuggestionsParams, SearchSuggestionsResponse,
};

const SEARCH_DEFAULT_LIMIT: u32 = 20;
const SEARCH_MAX_LIMIT: u32 = 50;
const SUGGESTIONS_DEFAULT_LIMIT: u32 = 8;
const SUGGESTIONS_MAX_LIMIT: u32 = 20;
const RECENT_DEFAULT_LIMIT: i64 = 10;
const RECENT_MAX_LIMIT: i64 = 50;

fn require_search_ops(state: &AppState) -> Result<&dyn SearchOperations, ApiError> {
    state
        .search_ops
        .as_deref()
        .ok_or(ApiError::ServiceUnavailable {
            message: "search service not configured".into(),
        })
}

fn normalize_query(raw: &str) -> Result<String, ApiError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(ApiError::ValidationError {
            errors: vec![FieldError {
                field: "q".into(),
                message: "must not be empty".into(),
            }],
        });
    }
    Ok(trimmed.to_string())
}

fn clamp_limit(value: Option<u32>, default_value: u32, max_value: u32) -> u32 {
    value.unwrap_or(default_value).clamp(1, max_value)
}

fn clamp_recent_limit(value: Option<u32>) -> i64 {
    i64::from(clamp_limit(
        value,
        RECENT_DEFAULT_LIMIT as u32,
        RECENT_MAX_LIMIT as u32,
    ))
}

fn apply_rate_limit_headers(
    headers: &mut HeaderMap,
    status: &ind_domain::SearchRateLimitStatus,
) -> Result<(), ApiError> {
    let pairs = [
        (
            HeaderName::from_static("x-ratelimit-limit"),
            status.limit.to_string(),
        ),
        (
            HeaderName::from_static("x-ratelimit-remaining"),
            status.remaining.to_string(),
        ),
        (
            HeaderName::from_static("x-ratelimit-reset"),
            status.reset_at.timestamp().to_string(),
        ),
    ];

    for (name, value) in pairs {
        let value = HeaderValue::from_str(&value).map_err(|err| ApiError::Internal {
            message: format!("invalid rate limit header value: {err}"),
        })?;
        headers.insert(name, value);
    }

    if let Some(retry_after) = status.retry_after_secs {
        let value =
            HeaderValue::from_str(&retry_after.to_string()).map_err(|err| ApiError::Internal {
                message: format!("invalid retry-after header value: {err}"),
            })?;
        headers.insert(RETRY_AFTER, value);
    }

    Ok(())
}

fn rate_limited_response(status: &ind_domain::SearchRateLimitStatus) -> Result<Response, ApiError> {
    let mut response = ApiError::RateLimited.into_response();
    apply_rate_limit_headers(response.headers_mut(), status)?;
    Ok(response)
}

#[utoipa::path(
    get,
    path = "/api/v1/search",
    params(SearchParams),
    responses(
        (status = 200, description = "Search results", body = SearchResultsResponse),
        (status = 401, description = "Authentication required"),
        (status = 422, description = "Validation error"),
        (status = 429, description = "Rate limited"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Search",
)]
pub async fn search(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Response, ApiError> {
    let search_ops = require_search_ops(&state)?;
    let query = normalize_query(&params.q)?;
    let limit = clamp_limit(params.limit, SEARCH_DEFAULT_LIMIT, SEARCH_MAX_LIMIT);
    let rate_limit = search_ops
        .consume_search_limit(auth_user.user_id)
        .await
        .map_err(ApiError::from)?;

    if !rate_limit.allowed {
        return rate_limited_response(&rate_limit);
    }

    let page = search_ops
        .search(auth_user.user_id, query.clone(), params.cursor, limit)
        .await
        .map_err(ApiError::from)?;

    let mut results = SearchResultsResponse::from_domain(page);
    enrich_results_with_senders(&state, auth_user.user_id, &mut results).await?;
    let mut response = axum::Json(results).into_response();
    apply_rate_limit_headers(response.headers_mut(), &rate_limit)?;
    Ok(response)
}

async fn enrich_results_with_senders(
    state: &AppState,
    user_id: ind_domain::UserId,
    results: &mut SearchResultsResponse,
) -> Result<(), ApiError> {
    let Some(ops) = state.email_sender_ops.as_ref() else {
        return Ok(());
    };
    let mut seen = std::collections::HashSet::new();
    let sender_ids: Vec<ind_domain::EmailSenderId> = results
        .results
        .iter()
        .filter_map(|r| r.sender_id.as_deref())
        .filter_map(|s| s.parse().ok())
        .filter(|id: &ind_domain::EmailSenderId| seen.insert(*id))
        .collect();
    if sender_ids.is_empty() {
        return Ok(());
    }

    let senders = ops
        .list_by_ids(user_id, sender_ids)
        .await
        .map_err(ApiError::from)?;
    let by_id: std::collections::HashMap<String, ind_domain::EmailSender> =
        senders.into_iter().map(|s| (s.id.to_string(), s)).collect();
    for result in &mut results.results {
        if let Some(sid) = result.sender_id.as_deref()
            && let Some(sender) = by_id.get(sid).cloned()
        {
            result.attach_sender(sender);
        }
    }
    Ok(())
}

#[utoipa::path(
    get,
    path = "/api/v1/search/suggestions",
    params(SearchSuggestionsParams),
    responses(
        (status = 200, description = "Search suggestions", body = SearchSuggestionsResponse),
        (status = 401, description = "Authentication required"),
        (status = 422, description = "Validation error"),
        (status = 429, description = "Rate limited"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Search",
)]
pub async fn suggestions(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Query(params): Query<SearchSuggestionsParams>,
) -> Result<Response, ApiError> {
    let search_ops = require_search_ops(&state)?;
    let query = params.q.trim().to_string();
    let limit = clamp_limit(
        params.limit,
        SUGGESTIONS_DEFAULT_LIMIT,
        SUGGESTIONS_MAX_LIMIT,
    );
    let rate_limit = search_ops
        .consume_suggestions_limit(auth_user.user_id)
        .await
        .map_err(ApiError::from)?;

    if !rate_limit.allowed {
        return rate_limited_response(&rate_limit);
    }

    let suggestions = search_ops
        .suggestions(auth_user.user_id, query.clone(), limit)
        .await
        .map_err(ApiError::from)?;

    let mut response =
        axum::Json(SearchSuggestionsResponse::from_domain(query, suggestions)).into_response();
    apply_rate_limit_headers(response.headers_mut(), &rate_limit)?;
    Ok(response)
}

#[utoipa::path(
    get,
    path = "/api/v1/search/recent",
    params(SearchRecentParams),
    responses(
        (status = 200, description = "Recent searches", body = RecentSearchListResponse),
        (status = 401, description = "Authentication required"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Search",
)]
pub async fn list_recent_searches(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Query(params): Query<SearchRecentParams>,
) -> Result<axum::Json<RecentSearchListResponse>, ApiError> {
    let search_ops = require_search_ops(&state)?;
    let items = search_ops
        .list_recent_searches(auth_user.user_id, clamp_recent_limit(params.limit))
        .await
        .map_err(ApiError::from)?;
    Ok(axum::Json(RecentSearchListResponse::from_domain(items)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/search/recent",
    responses(
        (status = 204, description = "Recent searches cleared"),
        (status = 401, description = "Authentication required"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Search",
)]
pub async fn clear_recent_searches(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
) -> Result<http::StatusCode, ApiError> {
    let search_ops = require_search_ops(&state)?;
    search_ops
        .clear_recent_searches(auth_user.user_id)
        .await
        .map_err(ApiError::from)?;
    Ok(http::StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/v1/search/recent/{recent_search_id}",
    params(
        ("recent_search_id" = String, Path, description = "Recent search ID"),
    ),
    responses(
        (status = 204, description = "Recent search deleted"),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Recent search not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Search",
)]
pub async fn delete_recent_search(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(recent_search_id): Path<String>,
) -> Result<http::StatusCode, ApiError> {
    let search_ops = require_search_ops(&state)?;
    let recent_search_id = dto::parse_recent_search_id(&recent_search_id)?;
    search_ops
        .delete_recent_search(auth_user.user_id, recent_search_id)
        .await
        .map_err(ApiError::from)?;
    Ok(http::StatusCode::NO_CONTENT)
}

pub fn search_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/search", get(search))
        .route("/api/v1/search/suggestions", get(suggestions))
        .route(
            "/api/v1/search/recent",
            get(list_recent_searches).delete(clear_recent_searches),
        )
        .route(
            "/api/v1/search/recent/{recent_search_id}",
            axum::routing::delete(delete_recent_search),
        )
}

#[cfg(test)]
mod tests;
