pub(crate) mod dto;

use axum::Router;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use ind_application::ports::{CreateSmartListRequest, SmartListOperations, UpdateSmartListRequest};

use crate::error::ApiError;
use crate::extract::ValidatedJson;
use crate::middleware::{RequireLibraryRead, RequireLibraryWrite};
use crate::response::PaginatedResponse;
use crate::state::AppState;

pub(crate) use dto::{
    CreateSmartListBody, FilterExpressionNode, FilterExpressionOperator, FilterExpressionValue,
    ListSmartListsParams, PinSmartListBody, SmartListResponse, UpdateSmartListBody,
};

fn require_smart_list_ops(state: &AppState) -> Result<&dyn SmartListOperations, ApiError> {
    state
        .smart_list_ops
        .as_deref()
        .ok_or(ApiError::ServiceUnavailable {
            message: "smart list service not configured".into(),
        })
}

#[utoipa::path(
    get,
    path = "/api/v1/smart-lists",
    params(ListSmartListsParams),
    responses(
        (status = 200, description = "List of smart lists", body = PaginatedResponse<SmartListResponse>),
        (status = 401, description = "Authentication required"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:read"]))),
    tag = "Smart Lists",
)]
pub async fn list_smart_lists(
    RequireLibraryRead {
        principal: auth_user,
        ..
    }: RequireLibraryRead,
    State(state): State<AppState>,
    Query(params): Query<ListSmartListsParams>,
) -> Result<PaginatedResponse<SmartListResponse>, ApiError> {
    let ops = require_smart_list_ops(&state)?;
    let page = ops
        .list_smart_lists(auth_user.user_id, params.cursor, params.limit)
        .await
        .map_err(ApiError::from)?;

    let items = page
        .items
        .into_iter()
        .map(SmartListResponse::from_domain)
        .collect();
    Ok(PaginatedResponse::from(ind_application::Page {
        items,
        next_cursor: page.next_cursor,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/smart-lists",
    request_body = CreateSmartListBody,
    responses(
        (status = 201, description = "Smart list created", body = SmartListResponse),
        (status = 401, description = "Authentication required"),
        (status = 422, description = "Validation error"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:write"]))),
    tag = "Smart Lists",
)]
pub async fn create_smart_list(
    RequireLibraryWrite {
        principal: auth_user,
        ..
    }: RequireLibraryWrite,
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<CreateSmartListBody>,
) -> Result<(http::StatusCode, crate::extract::Json<SmartListResponse>), ApiError> {
    let ops = require_smart_list_ops(&state)?;

    let req = CreateSmartListRequest {
        name: body.name,
        icon: body.icon,
        color: body.color,
        filter_expression: body.filter_expression,
        default_sort: body.default_sort,
    };

    let result = ops
        .create_smart_list(auth_user.user_id, req)
        .await
        .map_err(ApiError::from)?;

    Ok((
        http::StatusCode::CREATED,
        crate::extract::Json(SmartListResponse::from_domain(result)),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/smart-lists/{id}",
    params(("id" = String, Path, description = "Smart list ID with sml_ prefix")),
    responses(
        (status = 200, description = "Smart list details", body = SmartListResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Smart list not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:read"]))),
    tag = "Smart Lists",
)]
pub async fn get_smart_list(
    RequireLibraryRead {
        principal: auth_user,
        ..
    }: RequireLibraryRead,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<crate::extract::Json<SmartListResponse>, ApiError> {
    let ops = require_smart_list_ops(&state)?;
    let sl_id = dto::parse_smart_list_id(&id)?;

    let result = ops
        .get_smart_list(auth_user.user_id, sl_id)
        .await
        .map_err(ApiError::from)?;

    Ok(crate::extract::Json(SmartListResponse::from_domain(result)))
}

#[utoipa::path(
    patch,
    path = "/api/v1/smart-lists/{id}",
    params(("id" = String, Path, description = "Smart list ID with sml_ prefix")),
    request_body = UpdateSmartListBody,
    responses(
        (status = 200, description = "Smart list updated", body = SmartListResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Smart list not found"),
        (status = 422, description = "Validation error"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:write"]))),
    tag = "Smart Lists",
)]
pub async fn update_smart_list(
    RequireLibraryWrite {
        principal: auth_user,
        ..
    }: RequireLibraryWrite,
    State(state): State<AppState>,
    Path(id): Path<String>,
    ValidatedJson(body): ValidatedJson<UpdateSmartListBody>,
) -> Result<crate::extract::Json<SmartListResponse>, ApiError> {
    let ops = require_smart_list_ops(&state)?;
    let sl_id = dto::parse_smart_list_id(&id)?;

    let req = UpdateSmartListRequest {
        name: body.name,
        icon: body.icon,
        color: body.color,
        filter_expression: body.filter_expression,
        default_sort: body.default_sort,
        is_pinned: body.is_pinned,
    };

    let result = ops
        .update_smart_list(auth_user.user_id, sl_id, req)
        .await
        .map_err(ApiError::from)?;

    Ok(crate::extract::Json(SmartListResponse::from_domain(result)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/smart-lists/{id}",
    params(("id" = String, Path, description = "Smart list ID with sml_ prefix")),
    responses(
        (status = 204, description = "Smart list deleted"),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Smart list not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:write"]))),
    tag = "Smart Lists",
)]
pub async fn delete_smart_list(
    RequireLibraryWrite {
        principal: auth_user,
        ..
    }: RequireLibraryWrite,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<http::StatusCode, ApiError> {
    let ops = require_smart_list_ops(&state)?;
    let sl_id = dto::parse_smart_list_id(&id)?;

    ops.delete_smart_list(auth_user.user_id, sl_id)
        .await
        .map_err(ApiError::from)?;

    Ok(http::StatusCode::NO_CONTENT)
}

use crate::routes::library::{dto::LibraryEntryResponse, library_entry_responses};

#[utoipa::path(
    get,
    path = "/api/v1/smart-lists/{id}/entries",
    params(
        ("id" = String, Path, description = "Smart list ID"),
        ListSmartListsParams,
    ),
    responses(
        (status = 200, description = "Library entries matching smart list filter", body = PaginatedResponse<LibraryEntryResponse>),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Smart list not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:read"]))),
    tag = "Smart Lists",
)]
pub async fn evaluate_smart_list_entries(
    RequireLibraryRead {
        principal: auth_user,
        ..
    }: RequireLibraryRead,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<ListSmartListsParams>,
) -> Result<PaginatedResponse<LibraryEntryResponse>, ApiError> {
    let ops = require_smart_list_ops(&state)?;
    let sl_id = dto::parse_smart_list_id(&id)?;

    let page = ops
        .evaluate_smart_list_entries(auth_user.user_id, sl_id, params.cursor, params.limit)
        .await
        .map_err(ApiError::from)?;

    let items = library_entry_responses(&state, page.items).await?;
    Ok(PaginatedResponse::from(ind_application::Page {
        items,
        next_cursor: page.next_cursor,
    }))
}

#[utoipa::path(
    patch,
    path = "/api/v1/smart-lists/{id}/pin",
    params(("id" = String, Path, description = "Smart list ID")),
    request_body = PinSmartListBody,
    responses(
        (status = 200, description = "Pin state updated", body = SmartListResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Smart list not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:write"]))),
    tag = "Smart Lists",
)]
pub async fn pin_smart_list(
    RequireLibraryWrite {
        principal: auth_user,
        ..
    }: RequireLibraryWrite,
    State(state): State<AppState>,
    Path(id): Path<String>,
    ValidatedJson(body): ValidatedJson<PinSmartListBody>,
) -> Result<crate::extract::Json<SmartListResponse>, ApiError> {
    let ops = require_smart_list_ops(&state)?;
    let sl_id = dto::parse_smart_list_id(&id)?;

    let result = ops
        .pin_smart_list(auth_user.user_id, sl_id, body.is_pinned)
        .await
        .map_err(ApiError::from)?;

    Ok(crate::extract::Json(SmartListResponse::from_domain(result)))
}

pub fn smart_list_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/smart-lists",
            get(list_smart_lists).post(create_smart_list),
        )
        .route(
            "/api/v1/smart-lists/{id}",
            get(get_smart_list)
                .patch(update_smart_list)
                .delete(delete_smart_list),
        )
        .route(
            "/api/v1/smart-lists/{id}/entries",
            get(evaluate_smart_list_entries),
        )
        .route(
            "/api/v1/smart-lists/{id}/pin",
            axum::routing::patch(pin_smart_list),
        )
}
