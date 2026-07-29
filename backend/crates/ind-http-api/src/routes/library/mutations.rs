use super::*;

#[utoipa::path(
    post,
    path = "/api/v1/library/{library_entry_id}/triage",
    params(("library_entry_id" = String, Path, description = "Library entry id")),
    request_body = LibraryTriageBody,
    responses(
        (status = 200, description = "Triage state updated", body = LibraryEntryResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Library entry not found"),
        (status = 422, description = "Validation error"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:write"]))),
    tag = "Library",
)]
pub async fn triage_entry(
    RequireLibraryWrite {
        principal: auth_user,
        ..
    }: RequireLibraryWrite,
    State(state): State<AppState>,
    Path(library_entry_id): Path<String>,
    ValidatedJson(body): ValidatedJson<LibraryTriageBody>,
) -> Result<ApiResponse<LibraryEntryResponse>, ApiError> {
    let ops = require_library_ops(&state)?;
    let id = parse_entry_id(&library_entry_id)?;
    let state_value =
        body.triage_state
            .parse::<TriageState>()
            .map_err(|_| ApiError::ValidationError {
                errors: vec![FieldError {
                    field: "triage_state".into(),
                    message: "must be one of: inbox, later, archive".into(),
                }],
            })?;

    let entry = ops
        .set_triage(auth_user.user_id, id, state_value)
        .await
        .map_err(ApiError::from)?;
    fetch_response(&state, ops, auth_user.user_id, entry.id).await
}

#[utoipa::path(
    post,
    path = "/api/v1/library/{library_entry_id}/favorite",
    operation_id = "toggle_library_favorite",
    params(("library_entry_id" = String, Path, description = "Library entry id")),
    responses(
        (status = 200, description = "Favorite toggled", body = LibraryEntryResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Library entry not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:write"]))),
    tag = "Library",
)]
pub async fn toggle_favorite(
    RequireLibraryWrite {
        principal: auth_user,
        ..
    }: RequireLibraryWrite,
    State(state): State<AppState>,
    Path(library_entry_id): Path<String>,
) -> Result<ApiResponse<LibraryEntryResponse>, ApiError> {
    let ops = require_library_ops(&state)?;
    let id = parse_entry_id(&library_entry_id)?;
    let entry = ops
        .toggle_favorite(auth_user.user_id, id)
        .await
        .map_err(ApiError::from)?;
    fetch_response(&state, ops, auth_user.user_id, entry.id).await
}

#[utoipa::path(
    post,
    path = "/api/v1/library/{library_entry_id}/shortlist",
    operation_id = "toggle_library_shortlist",
    params(("library_entry_id" = String, Path, description = "Library entry id")),
    responses(
        (status = 200, description = "Shortlist toggled", body = LibraryEntryResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Library entry not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:write"]))),
    tag = "Library",
)]
pub async fn toggle_shortlist(
    RequireLibraryWrite {
        principal: auth_user,
        ..
    }: RequireLibraryWrite,
    State(state): State<AppState>,
    Path(library_entry_id): Path<String>,
) -> Result<ApiResponse<LibraryEntryResponse>, ApiError> {
    let ops = require_library_ops(&state)?;
    let id = parse_entry_id(&library_entry_id)?;
    let entry = ops
        .toggle_shortlist(auth_user.user_id, id)
        .await
        .map_err(ApiError::from)?;
    fetch_response(&state, ops, auth_user.user_id, entry.id).await
}

#[utoipa::path(
    delete,
    path = "/api/v1/library/{library_entry_id}",
    params(("library_entry_id" = String, Path, description = "Library entry id")),
    responses(
        (status = 204, description = "Library entry removed"),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Library entry not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:write"]))),
    tag = "Library",
)]
pub async fn delete_library_entry(
    RequireLibraryWrite {
        principal: auth_user,
        ..
    }: RequireLibraryWrite,
    State(state): State<AppState>,
    Path(library_entry_id): Path<String>,
) -> Result<EmptyResponse, ApiError> {
    let ops = require_library_ops(&state)?;
    let id = parse_entry_id(&library_entry_id)?;
    ops.delete(auth_user.user_id, id)
        .await
        .map_err(ApiError::from)?;
    Ok(EmptyResponse)
}

#[utoipa::path(
    post,
    path = "/api/v1/library/{library_entry_id}/restore",
    params(("library_entry_id" = String, Path, description = "Library entry id")),
    responses(
        (status = 200, description = "Library entry restored", body = LibraryEntryResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Library entry not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:write"]))),
    tag = "Library",
)]
pub async fn restore_entry(
    RequireLibraryWrite {
        principal: auth_user,
        ..
    }: RequireLibraryWrite,
    State(state): State<AppState>,
    Path(library_entry_id): Path<String>,
) -> Result<ApiResponse<LibraryEntryResponse>, ApiError> {
    let ops = require_library_ops(&state)?;
    let id = parse_entry_id(&library_entry_id)?;
    let entry = ops
        .restore(auth_user.user_id, id)
        .await
        .map_err(ApiError::from)?;
    fetch_response(&state, ops, auth_user.user_id, entry.id).await
}

#[utoipa::path(
    post,
    path = "/api/v1/library/{library_entry_id}/purge",
    params(("library_entry_id" = String, Path, description = "Library entry id")),
    responses(
        (status = 204, description = "Library entry permanently removed (document retained)"),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Library entry not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:write"]))),
    tag = "Library",
)]
pub async fn purge_entry(
    RequireLibraryWrite {
        principal: auth_user,
        ..
    }: RequireLibraryWrite,
    State(state): State<AppState>,
    Path(library_entry_id): Path<String>,
) -> Result<EmptyResponse, ApiError> {
    let ops = require_library_ops(&state)?;
    let id = parse_entry_id(&library_entry_id)?;
    ops.purge(auth_user.user_id, id)
        .await
        .map_err(ApiError::from)?;
    Ok(EmptyResponse)
}

#[utoipa::path(
    post,
    path = "/api/v1/library/trash/empty",
    operation_id = "empty_library_trash",
    responses(
        (status = 200, description = "Trash emptied (documents retained)", body = EmptyTrashResponse),
        (status = 401, description = "Authentication required"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:write"]))),
    tag = "Library",
)]
pub async fn empty_trash(
    RequireLibraryWrite {
        principal: auth_user,
        ..
    }: RequireLibraryWrite,
    State(state): State<AppState>,
) -> Result<ApiResponse<EmptyTrashResponse>, ApiError> {
    let ops = require_library_ops(&state)?;
    let purged = ops
        .empty_trash(auth_user.user_id)
        .await
        .map_err(ApiError::from)?;
    Ok(ApiResponse::new(EmptyTrashResponse { purged }))
}

#[utoipa::path(
    get,
    path = "/api/v1/library/trash",
    operation_id = "list_library_trash",
    params(ListLibraryParams),
    responses(
        (status = 200, description = "Paginated trashed library entries", body = PaginatedResponse<LibraryEntryResponse>),
        (status = 401, description = "Authentication required"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:read"]))),
    tag = "Library",
)]
pub async fn list_trash(
    RequireLibraryRead {
        principal: auth_user,
        ..
    }: RequireLibraryRead,
    State(state): State<AppState>,
    Query(params): Query<ListLibraryParams>,
) -> Result<PaginatedResponse<LibraryEntryResponse>, ApiError> {
    let ops = require_library_ops(&state)?;
    let cursor = params.cursor.map(Cursor);
    let limit = params.limit.unwrap_or(state.config.default_page_size);

    let page = ops
        .list_trashed(auth_user.user_id, cursor, limit)
        .await
        .map_err(ApiError::from)?;

    let items = library_entry_responses(&state, page.items).await?;
    Ok(PaginatedResponse::from(Page {
        items,
        next_cursor: page.next_cursor,
    }))
}

#[utoipa::path(
    put,
    path = "/api/v1/library/{library_entry_id}/tags",
    params(("library_entry_id" = String, Path, description = "Library entry id")),
    request_body = LibraryEntryTagsBody,
    responses(
        (status = 200, description = "Tag set replaced", body = LibraryEntryTagsResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Library entry not found"),
        (status = 422, description = "Validation error"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:write"]))),
    tag = "Library",
)]
pub async fn set_entry_tags(
    RequireLibraryWrite {
        principal: auth_user,
        ..
    }: RequireLibraryWrite,
    State(state): State<AppState>,
    Path(library_entry_id): Path<String>,
    axum::Json(body): axum::Json<LibraryEntryTagsBody>,
) -> Result<ApiResponse<LibraryEntryTagsResponse>, ApiError> {
    let library_ops = require_library_ops(&state)?;
    let tag_ops = state
        .tag_ops
        .as_deref()
        .ok_or(ApiError::ServiceUnavailable {
            message: "tag service not configured".into(),
        })?;
    let id = parse_entry_id(&library_entry_id)?;

    if body.tags.len() > 20 {
        return Err(ApiError::ValidationError {
            errors: vec![FieldError {
                field: "tags".into(),
                message: "maximum 20 tags per library entry".into(),
            }],
        });
    }

    // Validate ownership of an active entry first so an unknown/other-user id is a clean 404
    // rather than a silently-empty tag replace.
    let joined = library_ops
        .get(auth_user.user_id, id)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound {
            entity: "LibraryEntry",
            id: library_entry_id,
        })?;

    let normalized: Vec<String> = body
        .tags
        .into_iter()
        .map(|t| t.trim().to_lowercase())
        .filter(|t| !t.is_empty())
        .collect();

    let tags = tag_ops
        .set_library_entry_tags(auth_user.user_id, id, joined.document.id, normalized)
        .await
        .map_err(ApiError::from)?;

    let mut tag_names: Vec<String> = tags.into_iter().map(|t| t.name).collect();
    tag_names.sort();
    Ok(ApiResponse::new(LibraryEntryTagsResponse {
        tags: tag_names,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/library/{library_entry_id}/tags",
    params(("library_entry_id" = String, Path, description = "Library entry id")),
    responses(
        (status = 200, description = "Library entry tags", body = LibraryEntryTagsResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Library entry not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:read"]))),
    tag = "Library",
)]
pub async fn get_entry_tags(
    RequireLibraryRead {
        principal: auth_user,
        ..
    }: RequireLibraryRead,
    State(state): State<AppState>,
    Path(library_entry_id): Path<String>,
) -> Result<ApiResponse<LibraryEntryTagsResponse>, ApiError> {
    let library_ops = require_library_ops(&state)?;
    let tag_ops = state
        .tag_ops
        .as_deref()
        .ok_or(ApiError::ServiceUnavailable {
            message: "tag service not configured".into(),
        })?;
    let id = parse_entry_id(&library_entry_id)?;

    library_ops
        .get(auth_user.user_id, id)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound {
            entity: "LibraryEntry",
            id: library_entry_id,
        })?;

    let mut tag_names: Vec<String> = tag_ops
        .list_library_entry_tags(auth_user.user_id, id)
        .await
        .map_err(ApiError::from)?
        .into_iter()
        .map(|tag| tag.name)
        .collect();
    tag_names.sort();
    Ok(ApiResponse::new(LibraryEntryTagsResponse {
        tags: tag_names,
    }))
}

/// Re-read the entry with its document so the mutation response carries full document fields.
async fn fetch_response(
    state: &AppState,
    ops: &dyn LibraryOperations,
    user_id: ind_domain::UserId,
    id: LibraryEntryId,
) -> Result<ApiResponse<LibraryEntryResponse>, ApiError> {
    let joined = ops
        .get(user_id, id)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound {
            entity: "LibraryEntry",
            id: id.to_string(),
        })?;
    Ok(ApiResponse::new(
        library_entry_response(state, joined).await?,
    ))
}
