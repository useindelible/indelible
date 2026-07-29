use axum::extract::{Path, Query, State};

use crate::error::ApiError;
use crate::extract::ValidatedJson;
use crate::middleware::{RequireAiRead, RequireAiWrite};
use crate::state::AppState;

use super::dto::{
    CreateMilaSessionBody, ListSessionsParams, MilaConversationResponse, MilaSessionListResponse,
    MilaSessionResponse, project_mila_conversation, project_mila_session,
    project_mila_session_preview,
};
use super::{parse_session_id, require_mila_session_ops, validation_error};

#[utoipa::path(
    get,
    path = "/api/v1/mila/sessions",
    params(ListSessionsParams),
    responses(
        (status = 200, description = "User's Mila sessions ordered by last_active desc", body = MilaSessionListResponse),
        (status = 401, description = "Authentication required"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["ai:read"]))),
    tag = "Mila",
)]
pub async fn list_sessions(
    RequireAiRead {
        principal: auth_user,
        ..
    }: RequireAiRead,
    State(state): State<AppState>,
    Query(params): Query<ListSessionsParams>,
) -> Result<axum::Json<MilaSessionListResponse>, ApiError> {
    let sessions = require_mila_session_ops(&state)?
        .list_sessions(auth_user.user_id, params.limit)
        .await
        .map_err(ApiError::from)?;
    Ok(axum::Json(MilaSessionListResponse {
        sessions: sessions
            .into_iter()
            .map(project_mila_session_preview)
            .collect(),
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/mila/sessions",
    request_body = CreateMilaSessionBody,
    responses(
        (status = 201, description = "Created Mila chat session", body = MilaSessionResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Item or collection not found"),
        (status = 422, description = "Validation error"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["ai:write"]))),
    tag = "Mila",
)]
pub async fn create_session(
    RequireAiWrite {
        principal: auth_user,
        ..
    }: RequireAiWrite,
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<CreateMilaSessionBody>,
) -> Result<(http::StatusCode, axum::Json<MilaSessionResponse>), ApiError> {
    let request = body.into_state_request().map_err(validation_error)?;
    let session = require_mila_session_ops(&state)?
        .create_session(auth_user.user_id, request)
        .await
        .map_err(ApiError::from)?;
    Ok((
        http::StatusCode::CREATED,
        axum::Json(project_mila_session(session)),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/mila/sessions/{session_id}/messages",
    params(("session_id" = String, Path, description = "Mila session ID")),
    responses(
        (status = 200, description = "Chronological Mila conversation history", body = MilaConversationResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Session not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["ai:read"]))),
    tag = "Mila",
)]
pub async fn get_session_messages(
    RequireAiRead {
        principal: auth_user,
        ..
    }: RequireAiRead,
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<axum::Json<MilaConversationResponse>, ApiError> {
    let session_id = parse_session_id(&session_id)?;
    let conversation = require_mila_session_ops(&state)?
        .get_session_messages(auth_user.user_id, session_id)
        .await
        .map_err(ApiError::from)?;
    Ok(axum::Json(project_mila_conversation(conversation)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/mila/sessions/{session_id}",
    params(("session_id" = String, Path, description = "Mila session ID")),
    responses(
        (status = 204, description = "Session deleted"),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Session not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["ai:write"]))),
    tag = "Mila",
)]
pub async fn delete_session(
    RequireAiWrite {
        principal: auth_user,
        ..
    }: RequireAiWrite,
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<http::StatusCode, ApiError> {
    let session_id = parse_session_id(&session_id)?;
    require_mila_session_ops(&state)?
        .delete_session(auth_user.user_id, session_id)
        .await
        .map_err(ApiError::from)?;
    Ok(http::StatusCode::NO_CONTENT)
}
