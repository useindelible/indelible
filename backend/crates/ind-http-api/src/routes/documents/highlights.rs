use super::*;

#[utoipa::path(
    post,
    path = "/api/v1/documents/{document_id}/highlights",
    params(("document_id" = String, Path, description = "Document id with doc_ prefix")),
    request_body = CreateHighlightBody,
    responses(
        (status = 201, description = "Highlight created", body = HighlightResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Document not found"),
        (status = 422, description = "Document not yet rendered, or validation error"),
        (status = 503, description = "Document reader service not configured"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Documents",
)]
pub async fn create_document_highlight(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(document_id): Path<String>,
    ValidatedJson(body): ValidatedJson<CreateHighlightBody>,
) -> Result<(http::StatusCode, crate::extract::Json<HighlightResponse>), ApiError> {
    let ops = require_document_reader_ops(&state)?;
    let document_id = parse_document_id(&document_id)?;
    let highlight = ops
        .create_highlight(
            auth_user.user_id,
            document_id,
            body.color,
            body.text_content,
            Some(body.locator.into()),
            body.source_locator.map(Into::into),
        )
        .await
        .map_err(ApiError::from)?;
    Ok((
        http::StatusCode::CREATED,
        crate::extract::Json(HighlightResponse::from_domain(highlight)),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/documents/{document_id}/highlights",
    params(("document_id" = String, Path, description = "Document id with doc_ prefix")),
    responses(
        (status = 200, description = "Highlights with notes for the document", body = HighlightListResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Document not found"),
        (status = 503, description = "Document reader service not configured"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Documents",
)]
pub async fn list_document_highlights(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(document_id): Path<String>,
) -> Result<crate::extract::Json<HighlightListResponse>, ApiError> {
    let ops = require_document_reader_ops(&state)?;
    let document_id = parse_document_id(&document_id)?;
    let highlights = ops
        .list_highlights(auth_user.user_id, document_id)
        .await
        .map_err(ApiError::from)?;
    let count = highlights.len();
    let items: Vec<HighlightWithNoteResponse> = highlights
        .into_iter()
        .map(HighlightWithNoteResponse::from_domain)
        .collect();
    Ok(crate::extract::Json(HighlightListResponse {
        highlights: items,
        count,
    }))
}
