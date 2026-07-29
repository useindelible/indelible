use axum::extract::{Path, State};

use crate::error::ApiError;
use crate::extract::ValidatedJson;
use crate::middleware::{RequireAiRead, RequireAiWrite};
use crate::state::AppState;

use super::dto::{
    CreateMilaPromptPresetBody, MilaPromptPresetResponse, MilaPromptPresetsResponse,
    UpdateMilaPromptPresetBody, project_mila_prompt_preset, project_mila_prompt_presets,
};
use super::{parse_prompt_preset_id, require_mila_prompt_preset_ops, validation_error};

#[utoipa::path(
    get,
    path = "/api/v1/mila/presets",
    responses((status = 200, description = "Prompt presets grouped by action", body = MilaPromptPresetsResponse)),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["ai:read"]))),
    tag = "Mila",
)]
pub async fn list_prompt_presets(
    RequireAiRead {
        principal: auth_user,
        ..
    }: RequireAiRead,
    State(state): State<AppState>,
) -> Result<axum::Json<MilaPromptPresetsResponse>, ApiError> {
    let groups = require_mila_prompt_preset_ops(&state)?
        .list_prompt_presets(auth_user.user_id)
        .await
        .map_err(ApiError::from)?;
    Ok(axum::Json(project_mila_prompt_presets(groups)))
}

#[utoipa::path(
    post,
    path = "/api/v1/mila/presets",
    request_body = CreateMilaPromptPresetBody,
    responses(
        (status = 201, description = "Created prompt preset", body = MilaPromptPresetResponse),
        (status = 401, description = "Authentication required"),
        (status = 422, description = "Validation error"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["ai:write"]))),
    tag = "Mila",
)]
pub async fn create_prompt_preset(
    RequireAiWrite {
        principal: auth_user,
        ..
    }: RequireAiWrite,
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<CreateMilaPromptPresetBody>,
) -> Result<(http::StatusCode, axum::Json<MilaPromptPresetResponse>), ApiError> {
    let request = body.into_state_request().map_err(validation_error)?;
    let preset = require_mila_prompt_preset_ops(&state)?
        .create_prompt_preset(auth_user.user_id, request)
        .await
        .map_err(ApiError::from)?;
    Ok((
        http::StatusCode::CREATED,
        axum::Json(project_mila_prompt_preset(preset)),
    ))
}

#[utoipa::path(
    patch,
    path = "/api/v1/mila/presets/{preset_id}",
    params(("preset_id" = String, Path, description = "Prompt preset ID")),
    request_body = UpdateMilaPromptPresetBody,
    responses(
        (status = 200, description = "Updated prompt preset", body = MilaPromptPresetResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Prompt preset not found"),
        (status = 422, description = "Validation error"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["ai:write"]))),
    tag = "Mila",
)]
pub async fn update_prompt_preset(
    RequireAiWrite {
        principal: auth_user,
        ..
    }: RequireAiWrite,
    State(state): State<AppState>,
    Path(preset_id): Path<String>,
    ValidatedJson(body): ValidatedJson<UpdateMilaPromptPresetBody>,
) -> Result<axum::Json<MilaPromptPresetResponse>, ApiError> {
    let preset_id = parse_prompt_preset_id(&preset_id)?;
    let preset = require_mila_prompt_preset_ops(&state)?
        .update_prompt_preset(auth_user.user_id, preset_id, body.into_state_request())
        .await
        .map_err(ApiError::from)?;
    Ok(axum::Json(project_mila_prompt_preset(preset)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/mila/presets/{preset_id}",
    params(("preset_id" = String, Path, description = "Prompt preset ID")),
    responses(
        (status = 204, description = "Prompt preset deleted"),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Prompt preset not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["ai:write"]))),
    tag = "Mila",
)]
pub async fn delete_prompt_preset(
    RequireAiWrite {
        principal: auth_user,
        ..
    }: RequireAiWrite,
    State(state): State<AppState>,
    Path(preset_id): Path<String>,
) -> Result<http::StatusCode, ApiError> {
    let preset_id = parse_prompt_preset_id(&preset_id)?;
    require_mila_prompt_preset_ops(&state)?
        .delete_prompt_preset(auth_user.user_id, preset_id)
        .await
        .map_err(ApiError::from)?;
    Ok(http::StatusCode::NO_CONTENT)
}
