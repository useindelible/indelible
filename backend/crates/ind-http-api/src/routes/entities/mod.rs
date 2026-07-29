pub(crate) mod dto;

use axum::Router;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use ind_application::ports::{EntityOperations, UpdateEntityRequest};

use crate::error::ApiError;
use crate::extract::ValidatedJson;
use crate::middleware::AccountAccess;
use crate::response::PaginatedResponse;
use crate::state::AppState;

pub(crate) use dto::{
    EntityCoOccurrenceResponse, EntityDetailResponse, EntityDocumentResponse,
    EntitySummaryResponse, ListEntitiesParams, ListEntityDocumentsParams, MergeEntityBody,
    UpdateEntityBody,
};

fn require_entity_ops(state: &AppState) -> Result<&dyn EntityOperations, ApiError> {
    state
        .entity_ops
        .as_deref()
        .ok_or(ApiError::ServiceUnavailable {
            message: "entity service not configured".into(),
        })
}

#[utoipa::path(
    get,
    path = "/api/v1/entities",
    params(ListEntitiesParams),
    responses(
        (status = 200, description = "Paginated entity summaries", body = PaginatedResponse<EntitySummaryResponse>),
        (status = 401, description = "Authentication required"),
        (status = 422, description = "Validation error"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Entities",
)]
pub async fn list_entities(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Query(params): Query<ListEntitiesParams>,
) -> Result<PaginatedResponse<EntitySummaryResponse>, ApiError> {
    let ops = require_entity_ops(&state)?;
    let entity_type = params
        .r#type
        .as_deref()
        .map(dto::parse_entity_type_param)
        .transpose()
        .map_err(|error| ApiError::ValidationError {
            errors: vec![error],
        })?;

    let page = ops
        .list_entities(auth_user.user_id, entity_type, params.cursor, params.limit)
        .await
        .map_err(ApiError::from)?;

    let items = page
        .items
        .into_iter()
        .map(EntitySummaryResponse::from_domain)
        .collect();
    Ok(PaginatedResponse::from(ind_application::Page {
        items,
        next_cursor: page.next_cursor,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/entities/{id}",
    params(("id" = String, Path, description = "Entity ID with ent_ prefix")),
    responses(
        (status = 200, description = "Entity detail and co-occurrence stats", body = EntityDetailResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Entity not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Entities",
)]
pub async fn get_entity(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<crate::extract::Json<EntityDetailResponse>, ApiError> {
    let ops = require_entity_ops(&state)?;
    let entity_id = dto::parse_entity_id(&id)?;
    let detail = ops
        .get_entity(auth_user.user_id, entity_id)
        .await
        .map_err(ApiError::from)?;
    Ok(crate::extract::Json(EntityDetailResponse::from_domain(
        detail,
    )))
}

#[utoipa::path(
    get,
    path = "/api/v1/entities/{id}/documents",
    params(
        ("id" = String, Path, description = "Entity ID with ent_ prefix"),
        ListEntityDocumentsParams,
    ),
    responses(
        (status = 200, description = "Documents mentioning the entity", body = PaginatedResponse<EntityDocumentResponse>),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Entity not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Entities",
)]
pub async fn list_entity_documents(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<ListEntityDocumentsParams>,
) -> Result<PaginatedResponse<EntityDocumentResponse>, ApiError> {
    let ops = require_entity_ops(&state)?;
    let entity_id = dto::parse_entity_id(&id)?;
    let page = ops
        .list_entity_documents(auth_user.user_id, entity_id, params.cursor, params.limit)
        .await
        .map_err(ApiError::from)?;

    let items = page
        .items
        .into_iter()
        .map(EntityDocumentResponse::from_domain)
        .collect();
    Ok(PaginatedResponse::from(ind_application::Page {
        items,
        next_cursor: page.next_cursor,
    }))
}

#[utoipa::path(
    patch,
    path = "/api/v1/entities/{id}",
    params(("id" = String, Path, description = "Entity ID with ent_ prefix")),
    request_body = UpdateEntityBody,
    responses(
        (status = 200, description = "Entity updated", body = EntityDetailResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Entity not found"),
        (status = 409, description = "Duplicate entity name"),
        (status = 422, description = "Validation error"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Entities",
)]
pub async fn update_entity(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(id): Path<String>,
    ValidatedJson(body): ValidatedJson<UpdateEntityBody>,
) -> Result<crate::extract::Json<EntityDetailResponse>, ApiError> {
    let ops = require_entity_ops(&state)?;
    let entity_id = dto::parse_entity_id(&id)?;
    let detail = ops
        .update_entity(
            auth_user.user_id,
            entity_id,
            UpdateEntityRequest {
                name: body.name,
                description: body.description,
            },
        )
        .await
        .map_err(ApiError::from)?;
    Ok(crate::extract::Json(EntityDetailResponse::from_domain(
        detail,
    )))
}

#[utoipa::path(
    post,
    path = "/api/v1/entities/{id}/merge",
    params(("id" = String, Path, description = "Source entity ID with ent_ prefix")),
    request_body = MergeEntityBody,
    responses(
        (status = 200, description = "Source entity merged into target", body = EntityDetailResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Entity not found"),
        (status = 422, description = "Validation error"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Entities",
)]
pub async fn merge_entity(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(id): Path<String>,
    ValidatedJson(body): ValidatedJson<MergeEntityBody>,
) -> Result<crate::extract::Json<EntityDetailResponse>, ApiError> {
    let ops = require_entity_ops(&state)?;
    let source_id = dto::parse_entity_id(&id)?;
    let target_id = dto::parse_entity_id(&body.target_id)?;
    let detail = ops
        .merge_entity(auth_user.user_id, source_id, target_id)
        .await
        .map_err(ApiError::from)?;
    Ok(crate::extract::Json(EntityDetailResponse::from_domain(
        detail,
    )))
}

pub fn entity_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/entities", get(list_entities))
        .route(
            "/api/v1/entities/{id}",
            get(get_entity).patch(update_entity),
        )
        .route(
            "/api/v1/entities/{id}/documents",
            get(list_entity_documents),
        )
        .route("/api/v1/entities/{id}/merge", post(merge_entity))
}
