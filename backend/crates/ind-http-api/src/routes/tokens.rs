mod dto;

use axum::Router;
use axum::extract::{Path, State};
use axum::routing::{delete, get};
use ind_domain::ApiTokenId;

use crate::error::ApiError;
use crate::extract::ValidatedJson;
use crate::middleware::RequireVerifiedWebAccess;
use crate::response::{ApiResponse, EmptyResponse};
use crate::state::AppState;
pub(crate) use dto::{
    ApiTokenResponse, CreateApiTokenRequest, CreateApiTokenResponse, TokenListResponse,
};

// -- Handlers --

#[utoipa::path(
    get,
    path = "/api/v1/tokens",
    responses(
        (status = 200, description = "List of API tokens", body = TokenListResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Verified web session required"),
    ),
    security(("bearer" = [])),
    tag = "API Tokens",
)]
pub async fn list_tokens(
    RequireVerifiedWebAccess(auth_user): RequireVerifiedWebAccess,
    State(state): State<AppState>,
) -> Result<ApiResponse<TokenListResponse>, ApiError> {
    let tokens = state
        .api_token_ops
        .list_tokens(auth_user.user_id)
        .await
        .map_err(ApiError::from)?;

    Ok(ApiResponse::new(TokenListResponse {
        data: tokens.into_iter().map(ApiTokenResponse::from).collect(),
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/tokens",
    request_body = CreateApiTokenRequest,
    responses(
        (status = 201, description = "Token created", body = CreateApiTokenResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Verified web session required"),
        (status = 422, description = "Validation error"),
    ),
    security(("bearer" = [])),
    tag = "API Tokens",
)]
pub async fn create_token(
    RequireVerifiedWebAccess(auth_user): RequireVerifiedWebAccess,
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<CreateApiTokenRequest>,
) -> Result<
    (
        http::StatusCode,
        crate::extract::Json<CreateApiTokenResponse>,
    ),
    ApiError,
> {
    let (token, raw_token) = state
        .api_token_ops
        .create_token(
            auth_user.user_id,
            body.name,
            body.permissions.into_iter().map(Into::into).collect(),
            body.expires_in.into_duration(),
        )
        .await
        .map_err(ApiError::from)?;

    Ok((
        http::StatusCode::CREATED,
        crate::extract::Json(CreateApiTokenResponse {
            token: ApiTokenResponse::from(token),
            raw_token,
        }),
    ))
}

#[utoipa::path(
    delete,
    path = "/api/v1/tokens/{token_id}",
    params(
        ("token_id" = String, Path, description = "Token ID with tok_ prefix"),
    ),
    responses(
        (status = 204, description = "Token revoked"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Verified web session required"),
        (status = 404, description = "Token not found"),
    ),
    security(("bearer" = [])),
    tag = "API Tokens",
)]
pub async fn revoke_token(
    RequireVerifiedWebAccess(auth_user): RequireVerifiedWebAccess,
    State(state): State<AppState>,
    Path(token_id): Path<String>,
) -> Result<EmptyResponse, ApiError> {
    let parsed_id: ApiTokenId = token_id.parse().map_err(|_| ApiError::NotFound {
        entity: "token",
        id: token_id.clone(),
    })?;

    state
        .api_token_ops
        .revoke_token(auth_user.user_id, parsed_id)
        .await
        .map_err(ApiError::from)?;

    Ok(EmptyResponse)
}

pub fn token_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/tokens", get(list_tokens).post(create_token))
        .route("/api/v1/tokens/{token_id}", delete(revoke_token))
}
