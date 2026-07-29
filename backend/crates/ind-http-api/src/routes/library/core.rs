use super::*;

fn parse_triage_filter(value: Option<&str>) -> Result<Option<TriageState>, ApiError> {
    value
        .map(|state| {
            state
                .parse::<TriageState>()
                .map_err(|_| ApiError::ValidationError {
                    errors: vec![FieldError {
                        field: "triage_state".into(),
                        message: "must be one of: inbox, later, archive".into(),
                    }],
                })
        })
        .transpose()
}

#[utoipa::path(
    post,
    path = "/api/v1/library",
    request_body = SaveUrlBody,
    responses(
        (status = 200, description = "Document saved to Library", body = LibraryEntryResponse),
        (status = 401, description = "Authentication required"),
        (status = 422, description = "Validation error"),
    ),
    security(("session_cookie" = []), ("api_token" = [])),
    tag = "Library",
)]
pub async fn save_url(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<SaveUrlBody>,
) -> Result<ApiResponse<LibraryEntryResponse>, ApiError> {
    let ops = require_library_ops(&state)?;
    let item_type = body
        .item_type
        .as_deref()
        .and_then(|t| t.parse::<DocumentType>().ok());

    let outcome = ops
        .save_url(
            auth_user.user_id,
            SaveUrlRequest {
                url: body.url,
                title: body.title,
                item_type,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(ApiResponse::new(
        library_entry_response_from_parts(&state, outcome.entry, outcome.document).await?,
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/library/from-delivery",
    request_body = SaveFromDeliveryBody,
    responses(
        (status = 200, description = "Feed delivery saved to Library", body = LibraryEntryResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Feed delivery not found"),
        (status = 422, description = "Validation error"),
    ),
    security(("session_cookie" = []), ("api_token" = [])),
    tag = "Library",
)]
pub async fn save_from_delivery(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<SaveFromDeliveryBody>,
) -> Result<ApiResponse<LibraryEntryResponse>, ApiError> {
    let ops = require_library_ops(&state)?;
    let delivery_id =
        body.delivery_id
            .parse::<FeedDeliveryId>()
            .map_err(|_| ApiError::ValidationError {
                errors: vec![FieldError {
                    field: "delivery_id".into(),
                    message: "invalid feed delivery id".into(),
                }],
            })?;

    let outcome = ops
        .save_from_delivery(auth_user.user_id, delivery_id)
        .await
        .map_err(ApiError::from)?;

    Ok(ApiResponse::new(
        library_entry_response_from_parts(&state, outcome.entry, outcome.document).await?,
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/library",
    params(ListLibraryParams),
    responses(
        (status = 200, description = "Paginated saved library entries", body = PaginatedResponse<LibraryEntryResponse>),
        (status = 401, description = "Authentication required"),
        (status = 422, description = "Validation error"),
    ),
    security(("session_cookie" = []), ("api_token" = [])),
    tag = "Library",
)]
pub async fn list_library(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Query(params): Query<ListLibraryParams>,
) -> Result<PaginatedResponse<LibraryEntryResponse>, ApiError> {
    let ops = require_library_ops(&state)?;
    let triage = parse_triage_filter(params.triage_state.as_deref())?;
    let cursor = params.cursor.map(Cursor);
    let limit = params.limit.unwrap_or(state.config.default_page_size);

    let page = ops
        .list(auth_user.user_id, triage, cursor, limit)
        .await
        .map_err(ApiError::from)?;

    let items = library_entry_responses(&state, page.items).await?;
    Ok(PaginatedResponse::from(Page {
        items,
        next_cursor: page.next_cursor,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/library/query",
    request_body = LibraryQueryBody,
    responses(
        (status = 200, description = "Paginated library entries matching the filter expression", body = PaginatedResponse<LibraryEntryResponse>),
        (status = 401, description = "Authentication required"),
        (status = 422, description = "Validation error"),
    ),
    security(("session_cookie" = []), ("api_token" = [])),
    tag = "Library",
)]
pub async fn query_library(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<LibraryQueryBody>,
) -> Result<PaginatedResponse<LibraryEntryResponse>, ApiError> {
    let page = match body.filter_expression {
        Some(expression) => {
            let ops = super::require_smart_list_ops(&state)?;
            ops.evaluate_library_filter(auth_user.user_id, expression, body.cursor, body.limit)
                .await
                .map_err(ApiError::from)?
        }
        None => {
            let ops = require_library_ops(&state)?;
            let limit = body.limit.unwrap_or(state.config.default_page_size);
            ops.list(auth_user.user_id, None, body.cursor.map(Cursor), limit)
                .await
                .map_err(ApiError::from)?
        }
    };

    let items = library_entry_responses(&state, page.items).await?;
    Ok(PaginatedResponse::from(Page {
        items,
        next_cursor: page.next_cursor,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/library/count",
    responses(
        (status = 200, description = "Active saved library entry count", body = LibraryCountResponse),
        (status = 401, description = "Authentication required"),
    ),
    security(("session_cookie" = []), ("api_token" = [])),
    tag = "Library",
)]
pub async fn count_library(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
) -> Result<ApiResponse<LibraryCountResponse>, ApiError> {
    let ops = require_library_ops(&state)?;
    let saved_count = ops.count(auth_user.user_id).await.map_err(ApiError::from)?;
    Ok(ApiResponse::new(LibraryCountResponse { saved_count }))
}

#[utoipa::path(
    get,
    path = "/api/v1/library/counts",
    params(LibraryCountsParams),
    responses(
        (status = 200, description = "Read-state and item-type counts for the scope", body = LibraryScopeCountsResponse),
        (status = 401, description = "Authentication required"),
        (status = 422, description = "Validation error"),
    ),
    security(("session_cookie" = []), ("api_token" = [])),
    tag = "Library",
)]
pub async fn library_counts(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Query(params): Query<LibraryCountsParams>,
) -> Result<ApiResponse<LibraryScopeCountsResponse>, ApiError> {
    let ops = require_library_ops(&state)?;
    let triage = parse_triage_filter(params.triage_state.as_deref())?;
    let counts = ops
        .scope_counts(auth_user.user_id, triage)
        .await
        .map_err(ApiError::from)?;

    Ok(ApiResponse::new(counts.into()))
}

#[utoipa::path(
    get,
    path = "/api/v1/library/{library_entry_id}",
    params(("library_entry_id" = String, Path, description = "Library entry id")),
    responses(
        (status = 200, description = "Library entry detail", body = LibraryEntryResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Library entry not found"),
    ),
    security(("session_cookie" = []), ("api_token" = [])),
    tag = "Library",
)]
pub async fn get_library_entry(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(library_entry_id): Path<String>,
) -> Result<ApiResponse<LibraryEntryResponse>, ApiError> {
    let ops = require_library_ops(&state)?;
    let id = parse_entry_id(&library_entry_id)?;
    let joined = ops
        .get(auth_user.user_id, id)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound {
            entity: "LibraryEntry",
            id: library_entry_id,
        })?;

    Ok(ApiResponse::new(
        library_entry_response(&state, joined).await?,
    ))
}
