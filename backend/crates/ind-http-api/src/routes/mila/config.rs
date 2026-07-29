use axum::extract::State;

use crate::error::ApiError;
use crate::extract::ValidatedJson;
use crate::middleware::AccountAccess;
use crate::state::AppState;

use super::dto::{
    MilaConfigResponse, MilaStatusResponse, TestMilaConfigBody, TestMilaConfigResponse,
    UpsertMilaConfigBody, project_mila_config, project_mila_provider_test, project_mila_status,
};
use super::require_mila_config_ops;

#[utoipa::path(
    get,
    path = "/api/v1/mila/status",
    responses((status = 200, description = "Current Mila embedding status", body = MilaStatusResponse)),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Mila",
)]
pub async fn get_status(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
) -> Result<axum::Json<MilaStatusResponse>, ApiError> {
    let status = require_mila_config_ops(&state)?
        .get_status(auth_user.user_id)
        .await
        .map_err(ApiError::from)?;
    Ok(axum::Json(project_mila_status(status)))
}

#[utoipa::path(
    get,
    path = "/api/v1/mila/config",
    responses((status = 200, description = "Current Mila configuration", body = MilaConfigResponse)),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Mila",
)]
pub async fn get_config(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
) -> Result<axum::Json<MilaConfigResponse>, ApiError> {
    let config = require_mila_config_ops(&state)?
        .get_config(auth_user.user_id)
        .await
        .map_err(ApiError::from)?;
    Ok(axum::Json(project_mila_config(config)))
}

#[utoipa::path(
    post,
    path = "/api/v1/mila/config",
    request_body = UpsertMilaConfigBody,
    responses(
        (status = 200, description = "Updated Mila configuration", body = MilaConfigResponse),
        (status = 401, description = "Authentication required"),
        (status = 422, description = "Validation error"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Mila",
)]
pub async fn upsert_config(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<UpsertMilaConfigBody>,
) -> Result<axum::Json<MilaConfigResponse>, ApiError> {
    let config = require_mila_config_ops(&state)?
        .upsert_config(auth_user.user_id, body.into_state_request())
        .await
        .map_err(ApiError::from)?;
    Ok(axum::Json(project_mila_config(config)))
}

#[utoipa::path(
    post,
    path = "/api/v1/mila/config/reindex",
    request_body = UpsertMilaConfigBody,
    responses(
        (status = 200, description = "Updated Mila configuration and queued a full embedding reindex", body = MilaConfigResponse),
        (status = 401, description = "Authentication required"),
        (status = 422, description = "Validation error"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Mila",
)]
pub async fn reindex_config(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<UpsertMilaConfigBody>,
) -> Result<axum::Json<MilaConfigResponse>, ApiError> {
    let config = require_mila_config_ops(&state)?
        .reindex_config(auth_user.user_id, body.into_state_request())
        .await
        .map_err(ApiError::from)?;
    Ok(axum::Json(project_mila_config(config)))
}

#[utoipa::path(
    post,
    path = "/api/v1/mila/config/test",
    request_body = TestMilaConfigBody,
    responses(
        (status = 200, description = "Provider connectivity test result", body = TestMilaConfigResponse),
        (status = 401, description = "Authentication required"),
        (status = 422, description = "Validation error"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Mila",
)]
pub async fn test_config(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<TestMilaConfigBody>,
) -> Result<axum::Json<TestMilaConfigResponse>, ApiError> {
    let result = require_mila_config_ops(&state)?
        .test_config(auth_user.user_id, body.into_state_request())
        .await
        .map_err(ApiError::from)?;
    Ok(axum::Json(project_mila_provider_test(result)))
}
