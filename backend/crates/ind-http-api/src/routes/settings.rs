mod dto;

use axum::Router;
use axum::extract::State;
use axum::routing::get;

use crate::error::ApiError;
use crate::extract::Json;
use crate::middleware::AccountAccess;
use crate::response::ApiResponse;
use crate::state::AppState;
pub(crate) use dto::{
    ArchivalSettingsResponse, NotificationsSettingsResponse, PreferencesSettingsResponse,
    UpdateArchivalRequest, UpdateNotificationsRequest, UpdatePreferencesRequest,
};

#[utoipa::path(
    get,
    path = "/api/v1/settings/preferences",
    responses((status = 200, description = "Preference settings", body = PreferencesSettingsResponse)),
    security(("session_cookie" = []), ("api_token" = [])),
    tag = "Settings",
)]
pub async fn get_preferences(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
) -> Result<ApiResponse<PreferencesSettingsResponse>, ApiError> {
    let settings = state
        .settings_ops
        .get_preferences(auth_user.user_id)
        .await
        .map_err(ApiError::from)?;
    Ok(ApiResponse::new(settings.into()))
}

#[utoipa::path(
    patch,
    path = "/api/v1/settings/preferences",
    request_body = UpdatePreferencesRequest,
    responses((status = 200, description = "Updated preference settings", body = PreferencesSettingsResponse)),
    security(("session_cookie" = []), ("api_token" = [])),
    tag = "Settings",
)]
pub async fn update_preferences(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Json(body): Json<UpdatePreferencesRequest>,
) -> Result<ApiResponse<PreferencesSettingsResponse>, ApiError> {
    let section = body.into_domain();
    let settings = state
        .settings_ops
        .update_preferences(auth_user.user_id, section.theme, section.settings)
        .await
        .map_err(ApiError::from)?;
    Ok(ApiResponse::new(settings.into()))
}

#[utoipa::path(
    get,
    path = "/api/v1/settings/notifications",
    responses((status = 200, description = "Notification settings", body = NotificationsSettingsResponse)),
    security(("session_cookie" = []), ("api_token" = [])),
    tag = "Settings",
)]
pub async fn get_notifications(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
) -> Result<ApiResponse<NotificationsSettingsResponse>, ApiError> {
    let settings = state
        .settings_ops
        .get_notifications(auth_user.user_id)
        .await
        .map_err(ApiError::from)?;
    Ok(ApiResponse::new(settings.into()))
}

#[utoipa::path(
    patch,
    path = "/api/v1/settings/notifications",
    request_body = UpdateNotificationsRequest,
    responses((status = 200, description = "Updated notification settings", body = NotificationsSettingsResponse)),
    security(("session_cookie" = []), ("api_token" = [])),
    tag = "Settings",
)]
pub async fn update_notifications(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Json(body): Json<UpdateNotificationsRequest>,
) -> Result<ApiResponse<NotificationsSettingsResponse>, ApiError> {
    let settings = state
        .settings_ops
        .update_notifications(auth_user.user_id, body.into_domain(auth_user.user_id))
        .await
        .map_err(ApiError::from)?;
    Ok(ApiResponse::new(settings.into()))
}

#[utoipa::path(
    get,
    path = "/api/v1/settings/archival",
    responses((status = 200, description = "Archival settings", body = ArchivalSettingsResponse)),
    security(("session_cookie" = []), ("api_token" = [])),
    tag = "Settings",
)]
pub async fn get_archival(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
) -> Result<ApiResponse<ArchivalSettingsResponse>, ApiError> {
    let settings = state
        .settings_ops
        .get_archival(auth_user.user_id)
        .await
        .map_err(ApiError::from)?;
    Ok(ApiResponse::new(settings.into()))
}

#[utoipa::path(
    patch,
    path = "/api/v1/settings/archival",
    request_body = UpdateArchivalRequest,
    responses((status = 200, description = "Updated archival settings", body = ArchivalSettingsResponse)),
    security(("session_cookie" = []), ("api_token" = [])),
    tag = "Settings",
)]
pub async fn update_archival(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Json(body): Json<UpdateArchivalRequest>,
) -> Result<ApiResponse<ArchivalSettingsResponse>, ApiError> {
    let settings = state
        .settings_ops
        .update_archival(auth_user.user_id, body.into())
        .await
        .map_err(ApiError::from)?;
    Ok(ApiResponse::new(settings.into()))
}

pub fn settings_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/settings/preferences",
            get(get_preferences).patch(update_preferences),
        )
        .route(
            "/api/v1/settings/notifications",
            get(get_notifications).patch(update_notifications),
        )
        .route(
            "/api/v1/settings/archival",
            get(get_archival).patch(update_archival),
        )
}
