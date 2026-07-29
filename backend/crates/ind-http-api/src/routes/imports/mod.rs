mod dto;

use axum::Router;
use axum::extract::{DefaultBodyLimit, Path, Query, State};
use axum::routing::{delete, get, post};
use bytes::BytesMut;
use ind_application::ports::{ImportUpload, ReadwiseImportUpload};
use ind_domain::ImportJobId;
use serde::Deserialize;

use crate::error::ApiError;
use crate::middleware::AccountAccess;
use crate::response::EmptyResponse;
use crate::state::AppState;

pub use dto::{
    ImportJobCountsDto, ImportJobItemOutcomeDto, ImportJobListResponse, ImportJobStatusResponse,
    ImportUploadResponse, ReadwiseImportReportDto, project_import_status,
};

enum ImportSourceSlug {
    Readwise,
}

impl ImportSourceSlug {
    fn from_path(raw: &str) -> Result<Self, ApiError> {
        match raw {
            "readwise" => Ok(Self::Readwise),
            _ => Err(ApiError::NotFound {
                entity: "import_source",
                id: raw.to_string(),
            }),
        }
    }
}

async fn read_multipart_field(
    field: axum::extract::multipart::Field<'_>,
    total_bytes: &mut usize,
    limit: usize,
) -> Result<ImportUpload, ApiError> {
    let filename = field.file_name().map(|s| s.to_string());
    let content_type = field.content_type().map(|s| s.to_string());

    let mut buf = BytesMut::new();
    let mut stream = field;
    while let Some(chunk) = stream.chunk().await.map_err(|e| ApiError::BadRequest {
        message: format!("error reading upload: {e}"),
    })? {
        *total_bytes += chunk.len();
        if *total_bytes > limit {
            return Err(ApiError::PayloadTooLarge);
        }
        buf.extend_from_slice(&chunk);
    }

    Ok(ImportUpload {
        bytes: buf.to_vec(),
        filename,
        content_type,
    })
}

#[utoipa::path(
    post,
    path = "/api/v1/imports/{slug}",
    params(("slug" = String, Path, description = "Import source slug")),
    responses(
        (status = 202, description = "Import job created", body = ImportUploadResponse),
        (status = 400, description = "Missing file or invalid upload"),
        (status = 401, description = "Authentication required"),
        (status = 413, description = "File too large"),
    ),
    security(("session_cookie" = [])),
    tag = "Imports",
)]
pub async fn upload_import(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(slug): Path<String>,
    mut multipart: axum::extract::Multipart,
) -> Result<(http::StatusCode, crate::extract::Json<ImportUploadResponse>), ApiError> {
    let source = ImportSourceSlug::from_path(&slug)?;
    let ops = state.import_ops.as_ref().ok_or(ApiError::NotFound {
        entity: "imports",
        id: String::new(),
    })?;
    let limit = state.config.max_import_upload_bytes;

    let job = match source {
        ImportSourceSlug::Readwise => {
            let mut rw = ReadwiseImportUpload {
                library_csv: None,
                archive_zip: None,
                feeds_opml: None,
            };
            let mut total = 0usize;
            while let Some(field) =
                multipart
                    .next_field()
                    .await
                    .map_err(|e| ApiError::BadRequest {
                        message: format!("multipart parse error: {e}"),
                    })?
            {
                match field.name().unwrap_or("") {
                    "library_csv" => {
                        rw.library_csv = Some(read_multipart_field(field, &mut total, limit).await?)
                    }
                    "archive_zip" => {
                        rw.archive_zip = Some(read_multipart_field(field, &mut total, limit).await?)
                    }
                    "feeds_opml" => {
                        rw.feeds_opml = Some(read_multipart_field(field, &mut total, limit).await?)
                    }
                    _ => {}
                }
            }
            if rw.library_csv.is_none() && rw.archive_zip.is_none() && rw.feeds_opml.is_none() {
                return Err(ApiError::BadRequest {
                    message: "at least one of library_csv, archive_zip, or feeds_opml is required"
                        .into(),
                });
            }
            ops.upload_readwise(auth_user.user_id, rw)
                .await
                .map_err(ApiError::from)?
        }
    };

    Ok((
        http::StatusCode::ACCEPTED,
        crate::extract::Json(ImportUploadResponse {
            import_job_id: job.id.to_string(),
            status: job.status.as_str().to_string(),
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/imports/{slug}",
    params(("slug" = String, Path, description = "Import job ID")),
    responses(
        (status = 200, description = "Import job status", body = ImportJobStatusResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Import job not found"),
    ),
    security(("session_cookie" = [])),
    tag = "Imports",
)]
pub async fn get_import(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<crate::extract::Json<ImportJobStatusResponse>, ApiError> {
    let id = slug;
    let parsed_id: ImportJobId = id.parse().map_err(|_| ApiError::NotFound {
        entity: "import",
        id: id.clone(),
    })?;
    let ops = state.import_ops.as_ref().ok_or(ApiError::NotFound {
        entity: "imports",
        id: id.clone(),
    })?;

    let output = ops
        .get_status(auth_user.user_id, parsed_id)
        .await
        .map_err(ApiError::from)?;

    Ok(crate::extract::Json(project_import_status(output)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/imports/{slug}/rollback",
    params(("slug" = String, Path, description = "Import job ID")),
    responses(
        (status = 204, description = "Import rolled back"),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Import job not found"),
    ),
    security(("session_cookie" = [])),
    tag = "Imports",
)]
pub async fn rollback_import(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<EmptyResponse, ApiError> {
    let id = slug;
    let parsed_id: ImportJobId = id.parse().map_err(|_| ApiError::NotFound {
        entity: "import",
        id: id.clone(),
    })?;
    let ops = state.import_ops.as_ref().ok_or(ApiError::NotFound {
        entity: "imports",
        id: id.clone(),
    })?;

    ops.rollback(auth_user.user_id, parsed_id)
        .await
        .map_err(ApiError::from)?;

    Ok(EmptyResponse)
}

#[derive(Debug, Deserialize)]
pub struct ListImportsQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    25
}

#[utoipa::path(
    get,
    path = "/api/v1/imports",
    params(
        ("limit" = Option<i64>, Query, description = "Max jobs to return (default 25, max 100)")
    ),
    responses(
        (status = 200, description = "List of recent import jobs", body = ImportJobListResponse),
        (status = 401, description = "Authentication required"),
    ),
    security(("session_cookie" = [])),
    tag = "Imports",
)]
pub async fn list_imports(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Query(query): Query<ListImportsQuery>,
) -> Result<crate::extract::Json<ImportJobListResponse>, ApiError> {
    let ops = state.import_ops.as_ref().ok_or(ApiError::NotFound {
        entity: "imports",
        id: String::new(),
    })?;
    let limit = query.limit.clamp(1, 100);
    let outputs = ops
        .list_recent(auth_user.user_id, limit)
        .await
        .map_err(ApiError::from)?;
    let jobs = outputs.into_iter().map(project_import_status).collect();
    Ok(crate::extract::Json(ImportJobListResponse { jobs }))
}

pub fn import_routes(max_upload_bytes: usize) -> Router<AppState> {
    // Multipart framing (boundary, headers, CRLFs) adds a small overhead on top
    // of the raw file payload. Allow 1 MiB of headroom so the route-level streaming
    // guard (`max_import_upload_bytes`) is the authoritative cap, not the Axum
    // body layer.
    let body_limit = max_upload_bytes.saturating_add(1024 * 1024);

    Router::new()
        .route("/api/v1/imports", get(list_imports))
        .route(
            "/api/v1/imports/{slug}",
            post(upload_import).get(get_import),
        )
        .route("/api/v1/imports/{slug}/rollback", delete(rollback_import))
        .layer(DefaultBodyLimit::max(body_limit))
}
