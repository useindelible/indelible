use super::*;
use ind_domain::ArchiveAssetKind;

#[utoipa::path(
    get,
    path = "/api/v1/documents/{document_id}",
    params(("document_id" = String, Path, description = "Document id with doc_ prefix")),
    responses(
        (status = 200, description = "Reader read-model", body = DocumentReaderResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Document not found"),
        (status = 503, description = "Document reader service not configured"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Documents",
)]
pub async fn get_document_reader(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(document_id): Path<String>,
) -> Result<ApiResponse<DocumentReaderResponse>, ApiError> {
    let ops = require_document_reader_ops(&state)?;
    let document_id = parse_document_id(&document_id)?;
    let view = ops
        .get_reader(auth_user.user_id, document_id)
        .await
        .map_err(ApiError::from)?;
    let summary = if let Some(provider) = state.export_summary_provider.as_ref() {
        provider
            .summary_for_document(view.document.id, view.document.excerpt.as_deref())
            .await
            .map_err(ApiError::from)?
    } else {
        view.document
            .excerpt
            .as_deref()
            .map(str::trim)
            .filter(|excerpt| !excerpt.is_empty())
            .map(ToOwned::to_owned)
    };
    Ok(ApiResponse::new(DocumentReaderResponse::from_view(
        view,
        state.config.base_url.trim_end_matches('/'),
        summary,
    )))
}

#[utoipa::path(
    get,
    path = "/api/v1/documents/{document_id}/assets/{asset_kind}",
    params(
        ("document_id" = String, Path, description = "Document id with doc_ prefix"),
        ("asset_kind" = String, Path, description = "Asset kind (e.g. readable_html)"),
    ),
    responses(
        (status = 200, description = "Asset metadata with presigned download URL", body = DocumentAssetResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Document or asset not found"),
        (status = 503, description = "Document reader service not configured"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Documents",
)]
pub async fn get_document_asset(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path((document_id, asset_kind)): Path<(String, String)>,
) -> Result<ApiResponse<DocumentAssetResponse>, ApiError> {
    let ops = require_document_reader_ops(&state)?;
    let document_id = parse_document_id(&document_id)?;
    let kind: ArchiveAssetKind = asset_kind.parse().map_err(|_| ApiError::BadRequest {
        message: format!("unknown asset kind: {asset_kind}"),
    })?;
    let result = ops
        .get_asset_url(auth_user.user_id, document_id, kind)
        .await
        .map_err(ApiError::from)?;
    Ok(ApiResponse::new(DocumentAssetResponse::from(result)))
}

#[utoipa::path(
    post,
    path = "/api/v1/documents/{document_id}/reprocess",
    params(("document_id" = String, Path, description = "Document id with doc_ prefix")),
    responses(
        (status = 200, description = "Reprocess job queued", body = DocumentReprocessResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Document not found"),
        (status = 422, description = "Document has no reprocessable source"),
        (status = 503, description = "Document reader service not configured"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Documents",
)]
pub async fn reprocess_document(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(document_id): Path<String>,
) -> Result<ApiResponse<DocumentReprocessResponse>, ApiError> {
    let ops = require_document_reader_ops(&state)?;
    let document_id = parse_document_id(&document_id)?;
    let result = ops
        .reprocess_document(auth_user.user_id, document_id)
        .await
        .map_err(ApiError::from)?;
    Ok(ApiResponse::new(DocumentReprocessResponse {
        queued: result.queued,
        job_type: result.job_type,
        retry_after_seconds: result.retry_after_seconds,
    }))
}
