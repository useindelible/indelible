use axum::extract::{Path, State};
use ind_domain::{IntegrationConnectionId, LibraryEntryId};

use crate::error::ApiError;
use crate::extract::ValidatedJson;
use crate::middleware::{RequireIntegrationsRead, RequireIntegrationsWrite};
use crate::response::ApiResponse;
use crate::state::AppState;

use super::dto::{
    IntegrationConnectionDto, ObsidianPreviewRequest, ObsidianPreviewResponse, ObsidianSettingsDto,
    UpdateObsidianSettingsRequest,
};

#[utoipa::path(
    post,
    path = "/api/v1/integrations/obsidian/setup",
    responses(
        (status = 200, description = "Obsidian connection ensured", body = IntegrationConnectionDto),
        (status = 401, description = "Authentication required"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["integrations:write"]))),
    tag = "Integrations",
)]
pub async fn setup_obsidian_connection(
    RequireIntegrationsWrite {
        principal: auth_user,
        ..
    }: RequireIntegrationsWrite,
    State(state): State<AppState>,
) -> Result<ApiResponse<IntegrationConnectionDto>, ApiError> {
    let ops = state.integration_ops.as_ref().ok_or(ApiError::NotFound {
        entity: "integrations",
        id: "obsidian".into(),
    })?;
    let connection = ops
        .setup_obsidian_connection(auth_user.user_id)
        .await
        .map_err(ApiError::from)?;
    Ok(ApiResponse::new(IntegrationConnectionDto::from(connection)))
}

#[utoipa::path(
    get,
    path = "/api/v1/integrations/{id}/obsidian/settings",
    params(("id" = String, Path, description = "Integration connection ID")),
    responses(
        (status = 200, description = "Obsidian export settings", body = ObsidianSettingsDto),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Connection not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["integrations:read"]))),
    tag = "Integrations",
)]
pub async fn get_obsidian_settings(
    RequireIntegrationsRead {
        principal: auth_user,
        ..
    }: RequireIntegrationsRead,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<ApiResponse<ObsidianSettingsDto>, ApiError> {
    let parsed_id: IntegrationConnectionId = id.parse().map_err(|_| ApiError::NotFound {
        entity: "integration",
        id: id.clone(),
    })?;
    let ops = state.integration_ops.as_ref().ok_or(ApiError::NotFound {
        entity: "integrations",
        id: id.clone(),
    })?;
    let settings = ops
        .get_obsidian_settings(auth_user.user_id, parsed_id)
        .await
        .map_err(ApiError::from)?;
    Ok(ApiResponse::new(ObsidianSettingsDto::from(settings)))
}

#[utoipa::path(
    patch,
    path = "/api/v1/integrations/{id}/obsidian/settings",
    params(("id" = String, Path, description = "Integration connection ID")),
    request_body = UpdateObsidianSettingsRequest,
    responses(
        (status = 200, description = "Updated Obsidian export settings", body = ObsidianSettingsDto),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Connection not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["integrations:write"]))),
    tag = "Integrations",
)]
pub async fn update_obsidian_settings(
    RequireIntegrationsWrite {
        principal: auth_user,
        ..
    }: RequireIntegrationsWrite,
    State(state): State<AppState>,
    Path(id): Path<String>,
    ValidatedJson(body): ValidatedJson<UpdateObsidianSettingsRequest>,
) -> Result<ApiResponse<ObsidianSettingsDto>, ApiError> {
    let parsed_id: IntegrationConnectionId = id.parse().map_err(|_| ApiError::NotFound {
        entity: "integration",
        id: id.clone(),
    })?;
    let ops = state.integration_ops.as_ref().ok_or(ApiError::NotFound {
        entity: "integrations",
        id: id.clone(),
    })?;
    let settings = ops
        .update_obsidian_settings(auth_user.user_id, parsed_id, body.into())
        .await
        .map_err(ApiError::from)?;
    Ok(ApiResponse::new(ObsidianSettingsDto::from(settings)))
}

#[utoipa::path(
    post,
    path = "/api/v1/integrations/{id}/obsidian/preview",
    params(("id" = String, Path, description = "Integration connection ID")),
    request_body = ObsidianPreviewRequest,
    responses(
        (status = 200, description = "Rendered Obsidian preview", body = ObsidianPreviewResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Connection or library entry not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["integrations:read"]))),
    tag = "Integrations",
)]
pub async fn preview_obsidian_export(
    RequireIntegrationsRead {
        principal: auth_user,
        ..
    }: RequireIntegrationsRead,
    State(state): State<AppState>,
    Path(id): Path<String>,
    ValidatedJson(body): ValidatedJson<ObsidianPreviewRequest>,
) -> Result<ApiResponse<ObsidianPreviewResponse>, ApiError> {
    let parsed_id: IntegrationConnectionId = id.parse().map_err(|_| ApiError::NotFound {
        entity: "integration",
        id: id.clone(),
    })?;
    let parsed_library_entry_id: Option<LibraryEntryId> = body
        .library_entry_id
        .map(|raw| {
            raw.parse().map_err(|_| ApiError::NotFound {
                entity: "library_entry",
                id: raw,
            })
        })
        .transpose()?;
    let settings = body.settings.map(Into::into);
    let ops = state.integration_ops.as_ref().ok_or(ApiError::NotFound {
        entity: "integrations",
        id: id.clone(),
    })?;
    let preview = ops
        .preview_obsidian_export(
            auth_user.user_id,
            parsed_id,
            parsed_library_entry_id,
            settings,
        )
        .await
        .map_err(ApiError::from)?;
    Ok(ApiResponse::new(ObsidianPreviewResponse {
        file_path: preview.file_path,
        full_content: preview.full_content,
        append_only_content: preview.append_only_content,
        full_document_text_path: preview.full_document_text_path,
        full_document_text: preview.full_document_text,
    }))
}
