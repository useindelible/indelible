use axum::extract::{Path, State};
use ind_domain::IntegrationConnectionId;

use crate::error::ApiError;
use crate::middleware::AccountAccess;
use crate::response::{ApiResponse, EmptyResponse};
use crate::state::AppState;

use super::dto::{IntegrationConnectionDto, IntegrationListResponse, SyncIntegrationResponse};

#[utoipa::path(
    get,
    path = "/api/v1/integrations",
    responses(
        (status = 200, description = "List of user integrations", body = IntegrationListResponse),
        (status = 401, description = "Authentication required"),
    ),
    security(("session_cookie" = []), ("api_token" = [])),
    tag = "Integrations",
)]
pub async fn list_integrations(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
) -> Result<ApiResponse<IntegrationListResponse>, ApiError> {
    let ops = state.integration_ops.as_ref().ok_or(ApiError::NotFound {
        entity: "integrations",
        id: String::new(),
    })?;

    let connections = ops
        .list_connections(auth_user.user_id)
        .await
        .map_err(ApiError::from)?;
    let pending = ops
        .pending_jobs_per_connection(auth_user.user_id)
        .await
        .map_err(ApiError::from)?;

    Ok(ApiResponse::new(IntegrationListResponse {
        connections: connections
            .into_iter()
            .map(|c| {
                let count = pending.get(&c.id).copied().unwrap_or(0);
                IntegrationConnectionDto::from_with_pending(c, count)
            })
            .collect(),
    }))
}
#[utoipa::path(
    delete,
    path = "/api/v1/integrations/{id}",
    params(("id" = String, Path, description = "Integration connection ID")),
    responses(
        (status = 204, description = "Disconnected"),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Connection not found"),
    ),
    security(("session_cookie" = [])),
    tag = "Integrations",
)]
pub async fn delete_integration(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<EmptyResponse, ApiError> {
    let parsed_id: IntegrationConnectionId = id.parse().map_err(|_| ApiError::NotFound {
        entity: "integration",
        id: id.clone(),
    })?;
    let ops = state.integration_ops.as_ref().ok_or(ApiError::NotFound {
        entity: "integrations",
        id: id.clone(),
    })?;

    ops.delete_connection(auth_user.user_id, parsed_id)
        .await
        .map_err(ApiError::from)?;

    Ok(EmptyResponse)
}

#[utoipa::path(
    post,
    path = "/api/v1/integrations/{id}/sync",
    params(("id" = String, Path, description = "Integration connection ID")),
    responses(
        (status = 202, description = "Sync enqueued", body = SyncIntegrationResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Connection not found"),
    ),
    security(("session_cookie" = [])),
    tag = "Integrations",
)]
pub async fn sync_integration(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<
    (
        http::StatusCode,
        crate::extract::Json<SyncIntegrationResponse>,
    ),
    ApiError,
> {
    let parsed_id: IntegrationConnectionId = id.parse().map_err(|_| ApiError::NotFound {
        entity: "integration",
        id: id.clone(),
    })?;
    let ops = state.integration_ops.as_ref().ok_or(ApiError::NotFound {
        entity: "integrations",
        id: id.clone(),
    })?;

    let enqueued = ops
        .sync_now(auth_user.user_id, parsed_id)
        .await
        .map_err(ApiError::from)?;

    Ok((
        http::StatusCode::ACCEPTED,
        crate::extract::Json(SyncIntegrationResponse {
            job_id: enqueued.job_id,
        }),
    ))
}
