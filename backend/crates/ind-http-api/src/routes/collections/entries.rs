//! Library-entry-keyed collection membership routes. Collections organize saved Library content,
//! so these operate on `library_entry_id` and return the canonical `LibraryEntryResponse`.

use super::*;
use crate::routes::library::{dto::LibraryEntryResponse, library_entry_responses};

#[utoipa::path(
    get,
    path = "/api/v1/collections/{id}/entries",
    params(
        ("id" = String, Path, description = "Collection ID"),
        ListCollectionsParams,
    ),
    responses(
        (status = 200, description = "Saved library entries in collection", body = PaginatedResponse<LibraryEntryResponse>),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Collection not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:read"]))),
    tag = "Collections",
)]
pub async fn list_collection_entries(
    RequireLibraryRead {
        principal: auth_user,
        ..
    }: RequireLibraryRead,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<ListCollectionsParams>,
) -> Result<PaginatedResponse<LibraryEntryResponse>, ApiError> {
    let ops = require_collection_ops(&state)?;
    let col_id = dto::parse_collection_id(&id)?;

    let page = ops
        .list_entries(auth_user.user_id, col_id, params.cursor, params.limit)
        .await
        .map_err(ApiError::from)?;

    let items = library_entry_responses(&state, page.items).await?;
    Ok(PaginatedResponse::from(ind_application::Page {
        items,
        next_cursor: page.next_cursor,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/collections/{id}/entries",
    params(("id" = String, Path, description = "Collection ID")),
    request_body = AddLibraryEntryBody,
    responses(
        (status = 204, description = "Library entry added to collection"),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Collection or library entry not found"),
        (status = 422, description = "Validation error"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:write"]))),
    tag = "Collections",
)]
pub async fn add_entry_to_collection(
    RequireLibraryWrite {
        principal: auth_user,
        ..
    }: RequireLibraryWrite,
    State(state): State<AppState>,
    Path(id): Path<String>,
    ValidatedJson(body): ValidatedJson<AddLibraryEntryBody>,
) -> Result<http::StatusCode, ApiError> {
    let ops = require_collection_ops(&state)?;
    let col_id = dto::parse_collection_id(&id)?;
    let entry_id = dto::parse_library_entry_id(&body.library_entry_id)?;

    // Validate library-entry ownership up front so an unknown/other-user id is a clean 404, not a
    // composite-FK 500. Composite FK remains the ultimate cross-tenant guard.
    let library_ops = state
        .library_ops
        .as_deref()
        .ok_or(ApiError::ServiceUnavailable {
            message: "library service not configured".into(),
        })?;
    library_ops
        .get(auth_user.user_id, entry_id)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound {
            entity: "LibraryEntry",
            id: body.library_entry_id,
        })?;

    ops.add_entry(auth_user.user_id, col_id, entry_id)
        .await
        .map_err(ApiError::from)?;

    Ok(http::StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct RemoveEntryPath {
    id: String,
    library_entry_id: String,
}

#[utoipa::path(
    delete,
    path = "/api/v1/collections/{id}/entries/{library_entry_id}",
    params(
        ("id" = String, Path, description = "Collection ID"),
        ("library_entry_id" = String, Path, description = "Library entry ID to remove"),
    ),
    responses(
        (status = 204, description = "Library entry removed from collection"),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Membership not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:write"]))),
    tag = "Collections",
)]
pub async fn remove_entry_from_collection(
    RequireLibraryWrite {
        principal: auth_user,
        ..
    }: RequireLibraryWrite,
    State(state): State<AppState>,
    Path(path): Path<RemoveEntryPath>,
) -> Result<http::StatusCode, ApiError> {
    let ops = require_collection_ops(&state)?;
    let col_id = dto::parse_collection_id(&path.id)?;
    let entry_id = dto::parse_library_entry_id(&path.library_entry_id)?;

    ops.remove_entry(auth_user.user_id, col_id, entry_id)
        .await
        .map_err(ApiError::from)?;

    Ok(http::StatusCode::NO_CONTENT)
}
