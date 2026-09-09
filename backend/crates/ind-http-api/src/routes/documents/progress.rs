use super::*;

#[utoipa::path(
    patch,
    path = "/api/v1/documents/{document_id}/progress",
    params(("document_id" = String, Path, description = "Document id with doc_ prefix")),
    request_body = UpdateDocumentProgressBody,
    responses(
        (status = 204, description = "Progress recorded"),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Document not found"),
        (status = 422, description = "Validation error"),
        (status = 503, description = "Document reader service not configured"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:write"]))),
    tag = "Documents",
)]
pub async fn update_document_progress(
    RequireLibraryWrite {
        principal: auth_user,
        ..
    }: RequireLibraryWrite,
    State(state): State<AppState>,
    Path(document_id): Path<String>,
    ValidatedJson(body): ValidatedJson<UpdateDocumentProgressBody>,
) -> Result<EmptyResponse, ApiError> {
    let ops = require_document_reader_ops(&state)?;
    let document_id = parse_document_id(&document_id)?;
    ops.update_progress(
        auth_user.user_id,
        document_id,
        body.progress_percent.round() as i32,
        body.chapter_locator,
        body.chapter_offset,
    )
    .await
    .map_err(ApiError::from)?;
    Ok(EmptyResponse)
}

#[utoipa::path(
    post,
    path = "/api/v1/documents/{document_id}/mark-unread",
    params(("document_id" = String, Path, description = "Document id with doc_ prefix")),
    responses(
        (status = 204, description = "Read status reset; the chapter position is kept"),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Document not found"),
        (status = 503, description = "Document reader service not configured"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:write"]))),
    tag = "Documents",
)]
pub async fn mark_document_unread(
    RequireLibraryWrite {
        principal: auth_user,
        ..
    }: RequireLibraryWrite,
    State(state): State<AppState>,
    Path(document_id): Path<String>,
) -> Result<EmptyResponse, ApiError> {
    let ops = require_document_reader_ops(&state)?;
    let document_id = parse_document_id(&document_id)?;
    ops.mark_unread(auth_user.user_id, document_id)
        .await
        .map_err(ApiError::from)?;
    Ok(EmptyResponse)
}
