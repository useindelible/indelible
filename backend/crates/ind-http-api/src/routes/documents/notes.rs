use super::*;
use ind_domain::DocumentNote;

fn note_response(note: DocumentNote) -> DocumentNoteResponse {
    DocumentNoteResponse {
        id: note.id.to_string(),
        body: note.body,
        created_at: note.created_at,
        updated_at: note.updated_at,
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/documents/{document_id}/note",
    params(("document_id" = String, Path, description = "Document id with doc_ prefix")),
    responses(
        (status = 200, description = "Document note", body = DocumentNoteResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Document or note not found"),
        (status = 503, description = "Document reader service not configured"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:read"]))),
    tag = "Documents",
)]
pub async fn get_document_note(
    RequireLibraryRead {
        principal: auth_user,
        ..
    }: RequireLibraryRead,
    State(state): State<AppState>,
    Path(document_id): Path<String>,
) -> Result<axum::Json<DocumentNoteResponse>, ApiError> {
    let ops = require_document_reader_ops(&state)?;
    let document_id = parse_document_id(&document_id)?;
    let note = ops
        .get_note(auth_user.user_id, document_id)
        .await
        .map_err(ApiError::from)?;
    let note = note.ok_or(ApiError::NotFound {
        entity: "DocumentNote",
        id: document_id.to_string(),
    })?;
    Ok(axum::Json(note_response(note)))
}

#[utoipa::path(
    put,
    path = "/api/v1/documents/{document_id}/note",
    params(("document_id" = String, Path, description = "Document id with doc_ prefix")),
    request_body = DocumentUpsertNoteBody,
    responses(
        (status = 200, description = "Document note upserted", body = DocumentNoteResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Document not found"),
        (status = 422, description = "Document not yet rendered"),
        (status = 503, description = "Document reader service not configured"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:write"]))),
    tag = "Documents",
)]
pub async fn upsert_document_note(
    RequireLibraryWrite {
        principal: auth_user,
        ..
    }: RequireLibraryWrite,
    State(state): State<AppState>,
    Path(document_id): Path<String>,
    ValidatedJson(body): ValidatedJson<DocumentUpsertNoteBody>,
) -> Result<axum::Json<DocumentNoteResponse>, ApiError> {
    let ops = require_document_reader_ops(&state)?;
    let document_id = parse_document_id(&document_id)?;
    let note = ops
        .upsert_note(auth_user.user_id, document_id, body.body)
        .await
        .map_err(ApiError::from)?;
    Ok(axum::Json(note_response(note)))
}
