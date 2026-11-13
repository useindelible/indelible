use super::*;
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use ind_application::repos::document_note::NoteWrite;

use super::dto::{note_conflict_response, note_response};

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
        (status = 409, description = "The note changed since the base the client saw", body = DocumentNoteConflictResponse, content_type = "application/problem+json"),
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
) -> Result<Response, ApiError> {
    let ops = require_document_reader_ops(&state)?;
    let document_id = parse_document_id(&document_id)?;
    let base = if body.force {
        None
    } else {
        body.base_updated_at
    };
    let written = ops
        .upsert_note(auth_user.user_id, document_id, body.body, base)
        .await
        .map_err(ApiError::from)?;
    Ok(match written {
        NoteWrite::Written(note) => axum::Json(note_response(note)).into_response(),
        NoteWrite::Conflict(current) => (
            StatusCode::CONFLICT,
            [(http::header::CONTENT_TYPE, "application/problem+json")],
            axum::Json(note_conflict_response(current)),
        )
            .into_response(),
    })
}
