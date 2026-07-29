pub(crate) mod dto;
pub(crate) mod entries;

pub use entries::{add_entry_to_collection, list_collection_entries, remove_entry_from_collection};

use axum::Router;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use ind_application::ports::{
    CollectionOperations, CreateCollectionRequest, UpdateCollectionRequest,
};
use serde::Deserialize;

use crate::error::ApiError;
use crate::extract::ValidatedJson;
use crate::middleware::{RequireLibraryRead, RequireLibraryWrite};
use crate::response::PaginatedResponse;
use crate::state::AppState;

pub(crate) use dto::{
    AddLibraryEntryBody, CollectionResponse, CreateCollectionBody, ListCollectionsParams,
    UpdateCollectionBody,
};

fn require_collection_ops(state: &AppState) -> Result<&dyn CollectionOperations, ApiError> {
    state
        .collection_ops
        .as_deref()
        .ok_or(ApiError::ServiceUnavailable {
            message: "collection service not configured".into(),
        })
}

#[utoipa::path(
    get,
    path = "/api/v1/collections",
    params(ListCollectionsParams),
    responses(
        (status = 200, description = "List of collections", body = PaginatedResponse<CollectionResponse>),
        (status = 401, description = "Authentication required"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:read"]))),
    tag = "Collections",
)]
pub async fn list_collections(
    RequireLibraryRead {
        principal: auth_user,
        ..
    }: RequireLibraryRead,
    State(state): State<AppState>,
    Query(params): Query<ListCollectionsParams>,
) -> Result<PaginatedResponse<CollectionResponse>, ApiError> {
    let ops = require_collection_ops(&state)?;
    let page = ops
        .list_collections(auth_user.user_id, params.cursor, params.limit)
        .await
        .map_err(ApiError::from)?;

    let items = page
        .items
        .into_iter()
        .map(CollectionResponse::from_domain)
        .collect();
    Ok(PaginatedResponse::from(ind_application::Page {
        items,
        next_cursor: page.next_cursor,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/collections",
    request_body = CreateCollectionBody,
    responses(
        (status = 201, description = "Collection created", body = CollectionResponse),
        (status = 401, description = "Authentication required"),
        (status = 422, description = "Validation error"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:write"]))),
    tag = "Collections",
)]
pub async fn create_collection(
    RequireLibraryWrite {
        principal: auth_user,
        ..
    }: RequireLibraryWrite,
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<CreateCollectionBody>,
) -> Result<(http::StatusCode, crate::extract::Json<CollectionResponse>), ApiError> {
    let ops = require_collection_ops(&state)?;

    let parent_id = body
        .parent_id
        .as_deref()
        .map(dto::parse_collection_id)
        .transpose()?;

    let req = CreateCollectionRequest {
        name: body.name,
        description: body.description,
        icon: body.icon,
        color: body.color,
        sort_order: body.sort_order,
        parent_id,
    };

    let result = ops
        .create_collection(auth_user.user_id, req)
        .await
        .map_err(ApiError::from)?;

    Ok((
        http::StatusCode::CREATED,
        crate::extract::Json(CollectionResponse::from_domain(result)),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/collections/{id}",
    params(("id" = String, Path, description = "Collection ID with col_ prefix")),
    responses(
        (status = 200, description = "Collection details", body = CollectionResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Collection not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:read"]))),
    tag = "Collections",
)]
pub async fn get_collection(
    RequireLibraryRead {
        principal: auth_user,
        ..
    }: RequireLibraryRead,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<crate::extract::Json<CollectionResponse>, ApiError> {
    let ops = require_collection_ops(&state)?;
    let col_id = dto::parse_collection_id(&id)?;

    let result = ops
        .get_collection(auth_user.user_id, col_id)
        .await
        .map_err(ApiError::from)?;

    Ok(crate::extract::Json(CollectionResponse::from_domain(
        result,
    )))
}

#[utoipa::path(
    patch,
    path = "/api/v1/collections/{id}",
    params(("id" = String, Path, description = "Collection ID with col_ prefix")),
    request_body = UpdateCollectionBody,
    responses(
        (status = 200, description = "Collection updated", body = CollectionResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Collection not found"),
        (status = 422, description = "Validation error"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:write"]))),
    tag = "Collections",
)]
pub async fn update_collection(
    RequireLibraryWrite {
        principal: auth_user,
        ..
    }: RequireLibraryWrite,
    State(state): State<AppState>,
    Path(id): Path<String>,
    ValidatedJson(body): ValidatedJson<UpdateCollectionBody>,
) -> Result<crate::extract::Json<CollectionResponse>, ApiError> {
    let ops = require_collection_ops(&state)?;
    let col_id = dto::parse_collection_id(&id)?;

    let parent_id: Option<Option<ind_domain::CollectionId>> = match body.parent_id {
        None => None,
        Some(None) => Some(None),
        Some(Some(ref pid)) => Some(Some(dto::parse_collection_id(pid)?)),
    };

    let req = UpdateCollectionRequest {
        name: body.name,
        description: body.description,
        icon: body.icon,
        color: body.color,
        sort_order: body.sort_order,
        parent_id,
    };

    let result = ops
        .update_collection(auth_user.user_id, col_id, req)
        .await
        .map_err(ApiError::from)?;

    Ok(crate::extract::Json(CollectionResponse::from_domain(
        result,
    )))
}

#[utoipa::path(
    delete,
    path = "/api/v1/collections/{id}",
    params(("id" = String, Path, description = "Collection ID with col_ prefix")),
    responses(
        (status = 204, description = "Collection deleted"),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Collection not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:write"]))),
    tag = "Collections",
)]
pub async fn delete_collection(
    RequireLibraryWrite {
        principal: auth_user,
        ..
    }: RequireLibraryWrite,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<http::StatusCode, ApiError> {
    let ops = require_collection_ops(&state)?;
    let col_id = dto::parse_collection_id(&id)?;

    ops.delete_collection(auth_user.user_id, col_id)
        .await
        .map_err(ApiError::from)?;

    Ok(http::StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/collections/{id}/children",
    params(
        ("id" = String, Path, description = "Parent collection ID"),
        ListCollectionsParams,
    ),
    responses(
        (status = 200, description = "Child collections", body = PaginatedResponse<CollectionResponse>),
        (status = 401, description = "Authentication required"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:read"]))),
    tag = "Collections",
)]
pub async fn list_children(
    RequireLibraryRead {
        principal: auth_user,
        ..
    }: RequireLibraryRead,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<ListCollectionsParams>,
) -> Result<PaginatedResponse<CollectionResponse>, ApiError> {
    let ops = require_collection_ops(&state)?;
    let col_id = dto::parse_collection_id(&id)?;

    let page = ops
        .list_children(auth_user.user_id, col_id, params.cursor, params.limit)
        .await
        .map_err(ApiError::from)?;

    let items = page
        .items
        .into_iter()
        .map(CollectionResponse::from_domain)
        .collect();
    Ok(PaginatedResponse::from(ind_application::Page {
        items,
        next_cursor: page.next_cursor,
    }))
}

pub fn collection_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/collections",
            get(list_collections).post(create_collection),
        )
        .route(
            "/api/v1/collections/{id}",
            get(get_collection)
                .patch(update_collection)
                .delete(delete_collection),
        )
        .route("/api/v1/collections/{id}/children", get(list_children))
        .route(
            "/api/v1/collections/{id}/entries",
            get(list_collection_entries).post(add_entry_to_collection),
        )
        .route(
            "/api/v1/collections/{id}/entries/{library_entry_id}",
            axum::routing::delete(remove_entry_from_collection),
        )
}
