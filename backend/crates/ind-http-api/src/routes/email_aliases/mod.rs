pub(crate) mod dto;

use std::sync::Arc;

use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get};
use ind_application::ports::{EmailAliasCreateError, EmailAliasOperations};

use crate::error::{ApiError, FieldError};
use crate::extract::{Json, ValidatedJson};
use crate::middleware::{RequireFeedsRead, RequireFeedsWrite};
use crate::state::AppState;

pub use dto::{
    AliasDestinationDto, AliasStatusDto, CreateEmailAliasRequest, EmailAliasResponse,
    ListEmailAliasesResponse,
};

fn service(state: &AppState) -> Result<&Arc<dyn EmailAliasOperations>, ApiError> {
    state
        .email_alias_ops
        .as_ref()
        .ok_or_else(|| ApiError::ServiceUnavailable {
            message: "email alias operations are not configured".into(),
        })
}

fn alias_response(state: &AppState, alias: ind_domain::EmailAlias) -> EmailAliasResponse {
    EmailAliasResponse::from_domain(
        alias,
        state.config.email_feed_domain.as_deref(),
        state.config.email_library_domain.as_deref(),
    )
}

#[utoipa::path(
    get,
    path = "/api/v1/email-aliases",
    responses(
        (status = 200, description = "All aliases for the authenticated user", body = ListEmailAliasesResponse),
        (status = 401, description = "Authentication required"),
        (status = 503, description = "Email alias operations not configured"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["feeds:read"]))),
    tag = "Email Aliases",
)]
pub async fn list_email_aliases(
    RequireFeedsRead {
        principal: auth_user,
        ..
    }: RequireFeedsRead,
    State(state): State<AppState>,
) -> Result<Json<ListEmailAliasesResponse>, ApiError> {
    let ops = service(&state)?;
    let aliases = ops.list(auth_user.user_id).await.map_err(ApiError::from)?;
    let data = aliases
        .into_iter()
        .map(|a| alias_response(&state, a))
        .collect();
    Ok(Json(ListEmailAliasesResponse { data }))
}

#[utoipa::path(
    post,
    path = "/api/v1/email-aliases",
    request_body = CreateEmailAliasRequest,
    responses(
        (status = 201, description = "Alias created", body = EmailAliasResponse),
        (status = 401, description = "Authentication required"),
        (status = 409, description = "Local part already taken on this destination"),
        (status = 422, description = "Validation error"),
        (status = 503, description = "Email alias operations not configured"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["feeds:write"]))),
    tag = "Email Aliases",
)]
pub async fn create_email_alias(
    RequireFeedsWrite {
        principal: auth_user,
        ..
    }: RequireFeedsWrite,
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<CreateEmailAliasRequest>,
) -> Result<(StatusCode, Json<EmailAliasResponse>), ApiError> {
    let ops = service(&state)?;
    let alias = ops
        .create(
            auth_user.user_id,
            body.destination.into(),
            body.local_part,
            body.is_default,
        )
        .await
        .map_err(|err| match err {
            EmailAliasCreateError::InvalidLocalPart(e) => ApiError::ValidationError {
                errors: vec![FieldError {
                    field: "local_part".into(),
                    message: e.to_string(),
                }],
            },
            EmailAliasCreateError::SeedTokenCollision => ApiError::Conflict {
                entity: "email_alias",
                message: "local part collides with another account's seed token".into(),
            },
            EmailAliasCreateError::Application(app_err) => ApiError::from(app_err),
        })?;
    Ok((StatusCode::CREATED, Json(alias_response(&state, alias))))
}

#[utoipa::path(
    delete,
    path = "/api/v1/email-aliases/{id}",
    params(("id" = String, Path, description = "Email alias ID with als_ prefix")),
    responses(
        (status = 204, description = "Alias retired"),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Email alias not found"),
        (status = 503, description = "Email alias operations not configured"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["feeds:write"]))),
    tag = "Email Aliases",
)]
pub async fn delete_email_alias(
    RequireFeedsWrite {
        principal: auth_user,
        ..
    }: RequireFeedsWrite,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let ops = service(&state)?;
    let alias_id = dto::parse_alias_id(&id)?;
    ops.delete(auth_user.user_id, alias_id)
        .await
        .map_err(ApiError::from)?;
    Ok(StatusCode::NO_CONTENT)
}

pub fn email_alias_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/email-aliases",
            get(list_email_aliases).post(create_email_alias),
        )
        .route("/api/v1/email-aliases/{id}", delete(delete_email_alias))
}
