use axum::extract::Multipart;
use ind_application::ports::UploadFileRequest;
use serde::Serialize;

use crate::extract::read_multipart_field_bytes;

use super::*;

#[utoipa::path(
    post,
    path = "/api/v1/library/uploads",
    request_body(content_type = "multipart/form-data", content = inline(LibraryUploadSchema)),
    responses(
        (status = 200, description = "Uploaded document saved to Library", body = LibraryEntryResponse),
        (status = 400, description = "Missing file or invalid upload"),
        (status = 401, description = "Authentication required"),
        (status = 413, description = "File too large"),
        (status = 422, description = "Validation error"),
        (status = 503, description = "Storage not configured"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:write"]))),
    tag = "Library",
)]
pub async fn upload_file(
    RequireLibraryWrite {
        principal: auth_user,
        ..
    }: RequireLibraryWrite,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<ApiResponse<LibraryEntryResponse>, ApiError> {
    let ops = require_library_upload_ops(&state)?;
    let limit = state.config.max_upload_bytes;
    let mut total = 0usize;
    let mut file: Option<UploadFileRequest> = None;
    let mut title: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| ApiError::BadRequest {
            message: format!("multipart parse error: {err}"),
        })?
    {
        match field.name().unwrap_or("") {
            "file" => {
                let filename = field.file_name().unwrap_or("upload").to_string();
                let content_type = field
                    .content_type()
                    .unwrap_or("application/octet-stream")
                    .to_string();
                let data = read_multipart_field_bytes(field, &mut total, limit).await?;
                file = Some(UploadFileRequest {
                    filename,
                    content_type,
                    data,
                    title_override: None,
                    max_bytes: limit,
                    asset_base_url: state.config.base_url.trim_end_matches('/').to_string(),
                });
            }
            "title" => {
                let bytes = read_multipart_field_bytes(field, &mut total, limit).await?;
                let value =
                    String::from_utf8(bytes.to_vec()).map_err(|_| ApiError::ValidationError {
                        errors: vec![FieldError {
                            field: "title".into(),
                            message: "must be valid UTF-8".into(),
                        }],
                    })?;
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    title = Some(trimmed.to_string());
                }
            }
            _ => {}
        }
    }

    let mut req = file.ok_or_else(|| ApiError::BadRequest {
        message: "missing 'file' field in multipart upload".into(),
    })?;
    req.title_override = title;

    let outcome = ops
        .upload_file(auth_user.user_id, req)
        .await
        .map_err(ApiError::from)?;

    Ok(ApiResponse::new(
        library_entry_response_from_parts(&state, outcome.entry, outcome.document).await?,
    ))
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UploadLimitsResponse {
    /// Largest accepted upload in bytes. The server aborts mid-stream once the
    /// running total crosses it, so clients should check before sending.
    pub max_upload_bytes: u64,
}

#[utoipa::path(
    get,
    path = "/api/v1/library/uploads/limits",
    responses(
        (status = 200, description = "Upload size limits", body = UploadLimitsResponse),
        (status = 401, description = "Authentication required"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:write"]))),
    tag = "Library",
)]
pub async fn upload_limits(
    _auth: RequireLibraryWrite,
    State(state): State<AppState>,
) -> ApiResponse<UploadLimitsResponse> {
    ApiResponse::new(UploadLimitsResponse {
        max_upload_bytes: state.config.max_upload_bytes as u64,
    })
}
