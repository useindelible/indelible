pub(crate) mod dto;

use axum::Router;
use axum::extract::{Path, Query, State};
use axum::routing::{get, put};
use ind_application::ports::HighlightOperations;
use serde::Deserialize;

use crate::error::ApiError;
use crate::extract::ValidatedJson;
use crate::middleware::AccountAccess;
use crate::state::AppState;

pub(crate) use dto::{
    CreateHighlightBody, HighlightListResponse, HighlightNoteResponse, HighlightResponse,
    HighlightTagsBody, HighlightTagsResponse, HighlightWithNoteResponse, LocatorSchema,
    LocatorSchemaFlat, PatchHighlightBody, RecentHighlightsResponse, SourceLocatorSchema,
    SourceLocatorSchemaFlat, UpsertNoteBody,
};

fn require_highlight_ops(state: &AppState) -> Result<&dyn HighlightOperations, ApiError> {
    state
        .highlight_ops
        .as_deref()
        .ok_or(ApiError::ServiceUnavailable {
            message: "highlight service not configured".into(),
        })
}

#[utoipa::path(
    patch,
    path = "/api/v1/highlights/{highlight_id}",
    params(
        ("highlight_id" = String, Path, description = "Highlight ID with hlt_ prefix"),
    ),
    request_body = PatchHighlightBody,
    responses(
        (status = 200, description = "Highlight updated", body = HighlightResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Highlight not found"),
        (status = 422, description = "Validation error"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Highlights",
)]
pub async fn patch_highlight(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(highlight_id): Path<String>,
    ValidatedJson(body): ValidatedJson<PatchHighlightBody>,
) -> Result<crate::extract::Json<HighlightResponse>, ApiError> {
    let highlight_ops = require_highlight_ops(&state)?;
    let highlight_id = dto::parse_highlight_id(&highlight_id)?;
    let color = body.color.ok_or_else(|| ApiError::ValidationError {
        errors: vec![crate::error::FieldError {
            field: "color".into(),
            message: "at least one field must be provided".into(),
        }],
    })?;

    let highlight = highlight_ops
        .update_highlight_color(auth_user.user_id, highlight_id, color)
        .await
        .map_err(ApiError::from)?;

    Ok(crate::extract::Json(HighlightResponse::from_domain(
        highlight,
    )))
}

#[utoipa::path(
    delete,
    path = "/api/v1/highlights/{highlight_id}",
    params(
        ("highlight_id" = String, Path, description = "Highlight ID with hlt_ prefix"),
    ),
    responses(
        (status = 204, description = "Highlight deleted"),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Highlight not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Highlights",
)]
pub async fn delete_highlight(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(highlight_id): Path<String>,
) -> Result<http::StatusCode, ApiError> {
    let highlight_ops = require_highlight_ops(&state)?;
    let highlight_id = dto::parse_highlight_id(&highlight_id)?;

    highlight_ops
        .delete_highlight(auth_user.user_id, highlight_id)
        .await
        .map_err(ApiError::from)?;

    Ok(http::StatusCode::NO_CONTENT)
}

#[utoipa::path(
    put,
    path = "/api/v1/highlights/{highlight_id}/note",
    params(
        ("highlight_id" = String, Path, description = "Highlight ID with hlt_ prefix"),
    ),
    request_body = UpsertNoteBody,
    responses(
        (status = 200, description = "Note upserted", body = HighlightNoteResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Highlight not found"),
        (status = 422, description = "Validation error"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Highlights",
)]
pub async fn upsert_note(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(highlight_id): Path<String>,
    ValidatedJson(body): ValidatedJson<UpsertNoteBody>,
) -> Result<crate::extract::Json<HighlightNoteResponse>, ApiError> {
    let highlight_ops = require_highlight_ops(&state)?;
    let highlight_id = dto::parse_highlight_id(&highlight_id)?;

    let note = highlight_ops
        .upsert_note(auth_user.user_id, highlight_id, body.body)
        .await
        .map_err(ApiError::from)?;

    Ok(crate::extract::Json(HighlightNoteResponse::from_domain(
        note,
    )))
}

#[utoipa::path(
    delete,
    path = "/api/v1/highlights/{highlight_id}/note",
    params(
        ("highlight_id" = String, Path, description = "Highlight ID with hlt_ prefix"),
    ),
    responses(
        (status = 204, description = "Note deleted"),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Highlight not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Highlights",
)]
pub async fn delete_note(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(highlight_id): Path<String>,
) -> Result<http::StatusCode, ApiError> {
    let highlight_ops = require_highlight_ops(&state)?;
    let highlight_id = dto::parse_highlight_id(&highlight_id)?;

    highlight_ops
        .delete_note(auth_user.user_id, highlight_id)
        .await
        .map_err(ApiError::from)?;

    Ok(http::StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/highlights/{highlight_id}/tags",
    params(("highlight_id" = String, Path, description = "Highlight ID")),
    responses(
        (status = 200, description = "Highlight tags", body = HighlightTagsResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Highlight not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Highlights",
)]
pub async fn get_highlight_tags(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(highlight_id): Path<String>,
) -> Result<axum::Json<HighlightTagsResponse>, ApiError> {
    let highlight_ops = require_highlight_ops(&state)?;
    let highlight_id = dto::parse_highlight_id(&highlight_id)?;
    let tags = highlight_ops
        .list_highlight_tags(auth_user.user_id, highlight_id)
        .await
        .map_err(ApiError::from)?;
    let mut tag_names: Vec<String> = tags.into_iter().map(|t| t.name).collect();
    tag_names.sort();
    Ok(axum::Json(HighlightTagsResponse { tags: tag_names }))
}

#[utoipa::path(
    put,
    path = "/api/v1/highlights/{highlight_id}/tags",
    params(("highlight_id" = String, Path, description = "Highlight ID")),
    request_body = HighlightTagsBody,
    responses(
        (status = 200, description = "Tags replaced", body = HighlightTagsResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Highlight not found"),
        (status = 422, description = "Too many tags"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Highlights",
)]
pub async fn set_highlight_tags(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(highlight_id): Path<String>,
    axum::Json(body): axum::Json<HighlightTagsBody>,
) -> Result<axum::Json<HighlightTagsResponse>, ApiError> {
    let highlight_ops = require_highlight_ops(&state)?;
    let highlight_id = dto::parse_highlight_id(&highlight_id)?;

    if body.tags.len() > 20 {
        return Err(ApiError::ValidationError {
            errors: vec![crate::error::FieldError {
                field: "tags".into(),
                message: "maximum 20 tags per highlight".into(),
            }],
        });
    }

    let normalized: Vec<String> = body
        .tags
        .into_iter()
        .map(|t| t.trim().to_lowercase())
        .filter(|t| !t.is_empty())
        .collect();

    let tags = highlight_ops
        .set_highlight_tags(auth_user.user_id, highlight_id, normalized)
        .await
        .map_err(ApiError::from)?;

    let mut tag_names: Vec<String> = tags.into_iter().map(|t| t.name).collect();
    tag_names.sort();
    Ok(axum::Json(HighlightTagsResponse { tags: tag_names }))
}

#[derive(Debug, Deserialize)]
pub struct RecentHighlightsParams {
    limit: Option<i64>,
}

#[utoipa::path(
    get,
    path = "/api/v1/highlights/recent",
    params(
        ("limit" = Option<i64>, Query, description = "Maximum number of highlights to return (default 10, max 20)"),
    ),
    responses(
        (status = 200, description = "List recent highlights across all items", body = RecentHighlightsResponse),
        (status = 401, description = "Authentication required"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "highlights",
    summary = "List recent highlights across all items",
)]
pub async fn list_recent_highlights(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Query(params): Query<RecentHighlightsParams>,
) -> Result<crate::extract::Json<RecentHighlightsResponse>, ApiError> {
    let highlight_ops = require_highlight_ops(&state)?;
    let limit = params.limit.unwrap_or(10).clamp(1, 20);

    let highlights = highlight_ops
        .list_recent_highlights(auth_user.user_id, limit)
        .await
        .map_err(ApiError::from)?;

    let items: Vec<HighlightWithNoteResponse> = highlights
        .into_iter()
        .map(HighlightWithNoteResponse::from_domain)
        .collect();

    let count = items.len();
    Ok(crate::extract::Json(RecentHighlightsResponse {
        highlights: items,
        count,
    }))
}

pub fn highlight_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/highlights/recent", get(list_recent_highlights))
        .route(
            "/api/v1/highlights/{highlight_id}",
            axum::routing::patch(patch_highlight).delete(delete_highlight),
        )
        .route(
            "/api/v1/highlights/{highlight_id}/note",
            put(upsert_note).delete(delete_note),
        )
        .route(
            "/api/v1/highlights/{highlight_id}/tags",
            get(get_highlight_tags).put(set_highlight_tags),
        )
}
