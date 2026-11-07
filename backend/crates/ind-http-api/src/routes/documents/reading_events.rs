use super::*;

#[utoipa::path(
    post,
    path = "/api/v1/documents/{document_id}/reading-events",
    params(("document_id" = String, Path, description = "Document id with doc_ prefix")),
    request_body = AppendReadingEventsBody,
    responses(
        (status = 202, description = "Events appended; exact duplicates counted as replayed", body = AppendReadingEventsResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Document not found"),
        (status = 409, description = "An event id or client sequence was reused with different content"),
        (status = 422, description = "Validation error"),
        (status = 503, description = "Document reader service not configured"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:write"]))),
    tag = "Documents",
)]
pub async fn append_reading_events(
    RequireLibraryWrite {
        principal: auth_user,
        ..
    }: RequireLibraryWrite,
    State(state): State<AppState>,
    Path(document_id): Path<String>,
    ValidatedJson(body): ValidatedJson<AppendReadingEventsBody>,
) -> Result<
    (
        http::StatusCode,
        crate::extract::Json<AppendReadingEventsResponse>,
    ),
    ApiError,
> {
    let ops = require_document_reader_ops(&state)?;
    let document_id = parse_document_id(&document_id)?;
    let origin = origin_from(&auth_user, body.client_id);
    let events = body
        .events
        .into_iter()
        .map(|event| event.into_domain(&origin))
        .collect();
    let outcome = ops
        .append_reading_events(auth_user.user_id, document_id, events)
        .await
        .map_err(ApiError::from)?;
    Ok((
        http::StatusCode::ACCEPTED,
        crate::extract::Json(AppendReadingEventsResponse {
            accepted: outcome.accepted,
            replayed: outcome.replayed,
        }),
    ))
}
