mod dto;

use axum::Router;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::{get, post};
use http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use ind_application::ports::ObsidianRefreshRequest;
use ind_domain::LibraryEntryId;

use crate::error::ApiError;
use crate::extract::{Json, ValidatedJson};
use crate::middleware::RequireObsidianSync;
use crate::response::ApiResponse;
use crate::state::AppState;

pub use dto::{
    AckObsidianRunRequest, AckObsidianSubjectDto, CreateObsidianRunRequest,
    ObsidianArtifactDownloadMeta, ObsidianRunStatusResponse, RecordObsidianRenameRequest,
    RecordObsidianRenameResponse, RefreshObsidianSubjectsRequest, RefreshObsidianSubjectsResponse,
};

#[utoipa::path(
    post,
    path = "/api/v1/export/obsidian/runs",
    request_body = CreateObsidianRunRequest,
    responses(
        (status = 202, description = "Obsidian export run created", body = ObsidianRunStatusResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "obsidian:sync permission required"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["obsidian:sync"]))),
    tag = "Export",
)]
pub async fn create_obsidian_run(
    RequireObsidianSync {
        principal: auth_user,
        ..
    }: RequireObsidianSync,
    State(state): State<AppState>,
    Json(body): Json<CreateObsidianRunRequest>,
) -> Result<(http::StatusCode, Json<ObsidianRunStatusResponse>), ApiError> {
    let ops = state.export_ops.as_ref().ok_or(ApiError::NotFound {
        entity: "export",
        id: String::new(),
    })?;
    let input = body
        .try_into()
        .map_err(|message| ApiError::BadRequest { message })?;
    let status = ops
        .create_obsidian_run(auth_user.user_id, input)
        .await
        .map_err(ApiError::from)?;
    Ok((
        http::StatusCode::ACCEPTED,
        Json(ObsidianRunStatusResponse::from(status)),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/export/obsidian/runs/{run_id}",
    params(("run_id" = String, Path, description = "Obsidian export run UUID")),
    responses(
        (status = 200, description = "Obsidian export run status", body = ObsidianRunStatusResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "obsidian:sync permission required"),
        (status = 404, description = "Run not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["obsidian:sync"]))),
    tag = "Export",
)]
pub async fn get_obsidian_run(
    RequireObsidianSync {
        principal: auth_user,
        ..
    }: RequireObsidianSync,
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<ApiResponse<ObsidianRunStatusResponse>, ApiError> {
    let ops = state.export_ops.as_ref().ok_or(ApiError::NotFound {
        entity: "export",
        id: String::new(),
    })?;
    let parsed = uuid::Uuid::parse_str(&run_id).map_err(|_| ApiError::NotFound {
        entity: "obsidian_export_run",
        id: run_id.clone(),
    })?;
    let status = ops
        .get_obsidian_run(auth_user.user_id, parsed)
        .await
        .map_err(ApiError::from)?;
    Ok(ApiResponse::new(ObsidianRunStatusResponse::from(status)))
}

#[utoipa::path(
    get,
    path = "/api/v1/export/obsidian/artifacts/{artifact_id}",
    params(("artifact_id" = String, Path, description = "Obsidian artifact UUID")),
    responses(
        (
            status = 200,
            description = "ZIP artifact bytes",
            content_type = "application/zip",
        ),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "obsidian:sync permission required"),
        (status = 404, description = "Artifact not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["obsidian:sync"]))),
    tag = "Export",
)]
pub async fn download_obsidian_artifact(
    RequireObsidianSync {
        principal: auth_user,
        ..
    }: RequireObsidianSync,
    State(state): State<AppState>,
    Path(artifact_id): Path<String>,
) -> Result<Response, ApiError> {
    let ops = state.export_ops.as_ref().ok_or(ApiError::NotFound {
        entity: "export",
        id: String::new(),
    })?;
    let parsed = uuid::Uuid::parse_str(&artifact_id).map_err(|_| ApiError::NotFound {
        entity: "obsidian_export_artifact",
        id: artifact_id.clone(),
    })?;
    let artifact = ops
        .get_obsidian_artifact(auth_user.user_id, parsed)
        .await
        .map_err(ApiError::from)?;
    let filename = format!("indelible-obsidian-{}.zip", artifact.artifact_id);
    Response::builder()
        .header(CONTENT_TYPE, artifact.content_type)
        .header(
            CONTENT_DISPOSITION,
            format!("attachment; filename=\"{filename}\""),
        )
        .body(Body::from(artifact.bytes))
        .map_err(|e| ApiError::Internal {
            message: e.to_string(),
        })
}

#[utoipa::path(
    post,
    path = "/api/v1/export/obsidian/runs/{run_id}/ack",
    params(("run_id" = String, Path, description = "Obsidian export run UUID")),
    request_body = AckObsidianRunRequest,
    responses(
        (status = 200, description = "Run acknowledgement recorded", body = ObsidianRunStatusResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "obsidian:sync permission required"),
        (status = 404, description = "Run not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["obsidian:sync"]))),
    tag = "Export",
)]
pub async fn ack_obsidian_run(
    RequireObsidianSync {
        principal: auth_user,
        ..
    }: RequireObsidianSync,
    State(state): State<AppState>,
    Path(run_id): Path<String>,
    Json(body): Json<AckObsidianRunRequest>,
) -> Result<ApiResponse<ObsidianRunStatusResponse>, ApiError> {
    let ops = state.export_ops.as_ref().ok_or(ApiError::NotFound {
        entity: "export",
        id: String::new(),
    })?;
    let parsed = uuid::Uuid::parse_str(&run_id).map_err(|_| ApiError::NotFound {
        entity: "obsidian_export_run",
        id: run_id.clone(),
    })?;
    let ack = body
        .try_into()
        .map_err(|message| ApiError::BadRequest { message })?;
    let status = ops
        .ack_obsidian_run(auth_user.user_id, parsed, ack)
        .await
        .map_err(ApiError::from)?;
    Ok(ApiResponse::new(ObsidianRunStatusResponse::from(status)))
}

#[utoipa::path(
    post,
    path = "/api/v1/export/obsidian/refresh",
    request_body = RefreshObsidianSubjectsRequest,
    responses(
        (status = 200, description = "Refresh subjects queued", body = RefreshObsidianSubjectsResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "obsidian:sync permission required"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["obsidian:sync"]))),
    tag = "Export",
)]
pub async fn refresh_obsidian_subjects(
    RequireObsidianSync {
        principal: auth_user,
        ..
    }: RequireObsidianSync,
    State(state): State<AppState>,
    Json(body): Json<RefreshObsidianSubjectsRequest>,
) -> Result<ApiResponse<RefreshObsidianSubjectsResponse>, ApiError> {
    let ops = state.export_ops.as_ref().ok_or(ApiError::NotFound {
        entity: "export",
        id: String::new(),
    })?;
    let subject_ids = body
        .subject_ids
        .into_iter()
        .map(|raw| raw.parse().map_err(|_| raw))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|raw| ApiError::BadRequest {
            message: format!("invalid subject id: {raw}"),
        })?;
    let result = ops
        .refresh_obsidian_subjects(
            auth_user.user_id,
            ObsidianRefreshRequest {
                subject_ids,
                reason: body.reason,
            },
        )
        .await
        .map_err(ApiError::from)?;
    Ok(ApiResponse::new(RefreshObsidianSubjectsResponse {
        queued: result.queued,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/export/obsidian/rename",
    request_body = RecordObsidianRenameRequest,
    responses(
        (status = 200, description = "Renamed export path recorded", body = RecordObsidianRenameResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "obsidian:sync permission required"),
        (status = 404, description = "Connection or subject not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["obsidian:sync"]))),
    tag = "Export",
)]
pub async fn record_obsidian_rename(
    RequireObsidianSync {
        principal: auth_user,
        ..
    }: RequireObsidianSync,
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<RecordObsidianRenameRequest>,
) -> Result<ApiResponse<RecordObsidianRenameResponse>, ApiError> {
    let parsed_subject_id: LibraryEntryId =
        body.subject_id.parse().map_err(|_| ApiError::NotFound {
            entity: "export_subject",
            id: body.subject_id.clone(),
        })?;
    let ops = state.export_ops.as_ref().ok_or(ApiError::NotFound {
        entity: "export",
        id: String::new(),
    })?;
    ops.record_obsidian_path_rename(auth_user.user_id, parsed_subject_id, body.new_path.clone())
        .await
        .map_err(ApiError::from)?;
    Ok(ApiResponse::new(RecordObsidianRenameResponse {
        subject_id: parsed_subject_id.to_string(),
        new_path: body.new_path,
    }))
}

pub fn export_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/export/obsidian/runs", post(create_obsidian_run))
        .route(
            "/api/v1/export/obsidian/runs/{run_id}",
            get(get_obsidian_run),
        )
        .route(
            "/api/v1/export/obsidian/artifacts/{artifact_id}",
            get(download_obsidian_artifact),
        )
        .route(
            "/api/v1/export/obsidian/runs/{run_id}/ack",
            post(ack_obsidian_run),
        )
        .route(
            "/api/v1/export/obsidian/refresh",
            post(refresh_obsidian_subjects),
        )
        .route(
            "/api/v1/export/obsidian/rename",
            post(record_obsidian_rename),
        )
}
