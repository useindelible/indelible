pub(crate) mod dto;

use axum::Router;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use ind_application::ports::{CreateTagRequest, TagOperations, UpdateTagRequest};

use crate::error::ApiError;
use crate::extract::ValidatedJson;
use crate::middleware::{RequireLibraryRead, RequireLibraryWrite};
use crate::response::PaginatedResponse;
use crate::state::AppState;

pub(crate) use dto::{CreateTagBody, ListTagsParams, MergeTagsBody, TagResponse, UpdateTagBody};

fn require_tag_ops(state: &AppState) -> Result<&dyn TagOperations, ApiError> {
    state
        .tag_ops
        .as_deref()
        .ok_or(ApiError::ServiceUnavailable {
            message: "tag service not configured".into(),
        })
}

#[utoipa::path(
    get,
    path = "/api/v1/tags",
    params(ListTagsParams),
    responses(
        (status = 200, description = "List of tags with item counts", body = PaginatedResponse<TagResponse>),
        (status = 401, description = "Authentication required"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:read"]))),
    tag = "Tags",
)]
pub async fn list_tags(
    RequireLibraryRead {
        principal: auth_user,
        ..
    }: RequireLibraryRead,
    State(state): State<AppState>,
    Query(params): Query<ListTagsParams>,
) -> Result<PaginatedResponse<TagResponse>, ApiError> {
    let ops = require_tag_ops(&state)?;
    let page = ops
        .list_tags(auth_user.user_id, params.cursor, params.limit, params.scope)
        .await
        .map_err(ApiError::from)?;

    let items = page
        .items
        .into_iter()
        .map(TagResponse::from_domain)
        .collect();
    Ok(PaginatedResponse::from(ind_application::Page {
        items,
        next_cursor: page.next_cursor,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/tags",
    request_body = CreateTagBody,
    responses(
        (status = 201, description = "Tag created", body = TagResponse),
        (status = 401, description = "Authentication required"),
        (status = 409, description = "Tag name already exists"),
        (status = 422, description = "Validation error"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:write"]))),
    tag = "Tags",
)]
pub async fn create_tag(
    RequireLibraryWrite {
        principal: auth_user,
        ..
    }: RequireLibraryWrite,
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<CreateTagBody>,
) -> Result<(http::StatusCode, crate::extract::Json<TagResponse>), ApiError> {
    let ops = require_tag_ops(&state)?;

    let parent_id = body
        .parent_id
        .as_deref()
        .map(dto::parse_tag_id)
        .transpose()?;

    let req = CreateTagRequest {
        name: body.name,
        color: body.color,
        parent_id,
    };

    let result = ops
        .create_tag(auth_user.user_id, req)
        .await
        .map_err(ApiError::from)?;

    Ok((
        http::StatusCode::CREATED,
        crate::extract::Json(TagResponse::from_domain(result)),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/tags/{id}",
    params(("id" = String, Path, description = "Tag ID with tag_ prefix")),
    responses(
        (status = 200, description = "Tag details", body = TagResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Tag not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:read"]))),
    tag = "Tags",
)]
pub async fn get_tag(
    RequireLibraryRead {
        principal: auth_user,
        ..
    }: RequireLibraryRead,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<crate::extract::Json<TagResponse>, ApiError> {
    let ops = require_tag_ops(&state)?;
    let tag_id = dto::parse_tag_id(&id)?;

    let result = ops
        .get_tag(auth_user.user_id, tag_id)
        .await
        .map_err(ApiError::from)?;

    Ok(crate::extract::Json(TagResponse::from_domain(result)))
}

#[utoipa::path(
    patch,
    path = "/api/v1/tags/{id}",
    params(("id" = String, Path, description = "Tag ID with tag_ prefix")),
    request_body = UpdateTagBody,
    responses(
        (status = 200, description = "Tag updated", body = TagResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Tag not found"),
        (status = 409, description = "Tag name conflict"),
        (status = 422, description = "Validation error"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:write"]))),
    tag = "Tags",
)]
pub async fn update_tag(
    RequireLibraryWrite {
        principal: auth_user,
        ..
    }: RequireLibraryWrite,
    State(state): State<AppState>,
    Path(id): Path<String>,
    ValidatedJson(body): ValidatedJson<UpdateTagBody>,
) -> Result<crate::extract::Json<TagResponse>, ApiError> {
    let ops = require_tag_ops(&state)?;
    let tag_id = dto::parse_tag_id(&id)?;

    let parent_id: Option<Option<ind_domain::TagId>> = match body.parent_id {
        None => None,
        Some(None) => Some(None),
        Some(Some(ref pid)) => Some(Some(dto::parse_tag_id(pid)?)),
    };

    let color: Option<Option<String>> = body.color;

    let req = UpdateTagRequest {
        name: body.name,
        color,
        parent_id,
    };

    let result = ops
        .update_tag(auth_user.user_id, tag_id, req)
        .await
        .map_err(ApiError::from)?;

    Ok(crate::extract::Json(TagResponse::from_domain(result)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/tags/{id}",
    params(("id" = String, Path, description = "Tag ID with tag_ prefix")),
    responses(
        (status = 204, description = "Tag deleted"),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Tag not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:write"]))),
    tag = "Tags",
)]
pub async fn delete_tag(
    RequireLibraryWrite {
        principal: auth_user,
        ..
    }: RequireLibraryWrite,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<http::StatusCode, ApiError> {
    let ops = require_tag_ops(&state)?;
    let tag_id = dto::parse_tag_id(&id)?;

    ops.delete_tag(auth_user.user_id, tag_id)
        .await
        .map_err(ApiError::from)?;

    Ok(http::StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/tags/merge",
    request_body = MergeTagsBody,
    responses(
        (status = 200, description = "Tags merged into target", body = TagResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Source or target tag not found"),
        (status = 422, description = "Validation error"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:write"]))),
    tag = "Tags",
)]
pub async fn merge_tags(
    RequireLibraryWrite {
        principal: auth_user,
        ..
    }: RequireLibraryWrite,
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<MergeTagsBody>,
) -> Result<crate::extract::Json<TagResponse>, ApiError> {
    let ops = require_tag_ops(&state)?;

    let source_ids: Vec<ind_domain::TagId> = body
        .source_ids
        .iter()
        .map(|s| dto::parse_tag_id(s))
        .collect::<Result<_, _>>()?;
    let target_id = dto::parse_tag_id(&body.target_id)?;

    let result = ops
        .merge_tags(auth_user.user_id, source_ids, target_id)
        .await
        .map_err(ApiError::from)?;

    Ok(crate::extract::Json(TagResponse::from_domain(result)))
}

use crate::routes::highlights::HighlightResponse;
use crate::routes::library::{dto::LibraryEntryResponse, library_entry_responses};

#[utoipa::path(
    get,
    path = "/api/v1/tags/{id}/highlights",
    params(
        ("id" = String, Path, description = "Tag ID"),
        ListTagsParams,
    ),
    responses(
        (status = 200, description = "Highlights with this tag", body = PaginatedResponse<HighlightResponse>),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Tag not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:read"]))),
    tag = "Tags",
)]
pub async fn list_tag_highlights(
    RequireLibraryRead {
        principal: auth_user,
        ..
    }: RequireLibraryRead,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<ListTagsParams>,
) -> Result<PaginatedResponse<HighlightResponse>, ApiError> {
    let ops = require_tag_ops(&state)?;
    let tag_id = dto::parse_tag_id(&id)?;

    let page = ops
        .list_tag_highlights(auth_user.user_id, tag_id, params.cursor, params.limit)
        .await
        .map_err(ApiError::from)?;

    let items = page
        .items
        .into_iter()
        .map(HighlightResponse::from_domain)
        .collect();
    Ok(PaginatedResponse::from(ind_application::Page {
        items,
        next_cursor: page.next_cursor,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/tags/{id}/entries",
    params(
        ("id" = String, Path, description = "Tag ID"),
        ListTagsParams,
    ),
    responses(
        (status = 200, description = "Saved library entries with this tag", body = PaginatedResponse<LibraryEntryResponse>),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Tag not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:read"]))),
    tag = "Tags",
)]
pub async fn list_tag_entries(
    RequireLibraryRead {
        principal: auth_user,
        ..
    }: RequireLibraryRead,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<ListTagsParams>,
) -> Result<PaginatedResponse<LibraryEntryResponse>, ApiError> {
    let ops = require_tag_ops(&state)?;
    let tag_id = dto::parse_tag_id(&id)?;

    let page = ops
        .list_tag_library_entries(auth_user.user_id, tag_id, params.cursor, params.limit)
        .await
        .map_err(ApiError::from)?;

    let items = library_entry_responses(&state, page.items).await?;
    Ok(PaginatedResponse::from(ind_application::Page {
        items,
        next_cursor: page.next_cursor,
    }))
}

pub fn tag_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/tags", get(list_tags).post(create_tag))
        .route("/api/v1/tags/merge", post(merge_tags))
        .route(
            "/api/v1/tags/{id}",
            get(get_tag).patch(update_tag).delete(delete_tag),
        )
        .route("/api/v1/tags/{id}/entries", get(list_tag_entries))
        .route("/api/v1/tags/{id}/highlights", get(list_tag_highlights))
}
