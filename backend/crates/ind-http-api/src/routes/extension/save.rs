use super::entries::{document_reader_url, parse_extension_document_type, parse_published_at};
use super::*;

/// Quick-save: URL + optional title, server does all fetching.
#[utoipa::path(
    post,
    path = "/api/v1/extension/quick-save",
    request_body = QuickSaveRequest,
    responses(
        (status = 202, description = "Save accepted", body = ExtensionSaveResponse),
        (status = 400, description = "Extension save not configured"),
        (status = 401, description = "Authentication required"),
        (status = 422, description = "Validation error"),
    ),
    security(("api_token" = [])),
    tag = "Extension",
)]
pub async fn extension_quick_save(
    State(state): State<AppState>,
    RequireExtensionAccess(auth_user): RequireExtensionAccess,
    ValidatedJson(body): ValidatedJson<QuickSaveRequest>,
) -> Result<(http::StatusCode, axum::Json<ExtensionSaveResponse>), ApiError> {
    let save_ops = state
        .extension_save_ops
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest {
            message: "extension save not configured".to_string(),
        })?;

    let result = save_ops
        .quick_save(
            auth_user.user_id,
            QuickSaveInput {
                url: body.url,
                title: body.title,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok((
        http::StatusCode::ACCEPTED,
        axum::Json(ExtensionSaveResponse {
            library_entry_id: result.library_entry_id.to_string(),
            status: result.status.to_string(),
            reader_url: document_reader_url(&state.config, &result.document_id),
        }),
    ))
}

/// Reader-save: URL + pre-extracted readable HTML + metadata.
#[utoipa::path(
    post,
    path = "/api/v1/extension/reader-save",
    request_body = ReaderSaveRequest,
    responses(
        (status = 202, description = "Save accepted", body = ExtensionSaveResponse),
        (status = 400, description = "Extension save not configured"),
        (status = 401, description = "Authentication required"),
        (status = 422, description = "Validation error"),
    ),
    security(("api_token" = [])),
    tag = "Extension",
)]
pub async fn extension_reader_save(
    State(state): State<AppState>,
    RequireExtensionAccess(auth_user): RequireExtensionAccess,
    ValidatedJson(body): ValidatedJson<ReaderSaveRequest>,
) -> Result<(http::StatusCode, axum::Json<ExtensionSaveResponse>), ApiError> {
    let save_ops = state
        .extension_save_ops
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest {
            message: "extension save not configured".to_string(),
        })?;

    let item_type = parse_extension_document_type(body.item_type.as_deref())?;

    tracing::info!(
        url = %body.url,
        canonical_url = ?body.canonical_url,
        title = ?body.title,
        item_type = ?item_type,
        "extension canonical_url received from link rel=canonical or og:url"
    );

    let result = save_ops
        .reader_save(
            auth_user.user_id,
            ReaderSaveInput {
                url: body.url,
                canonical_url: body.canonical_url,
                title: body.title,
                author: body.author,
                excerpt: body.excerpt,
                reader_html: body.reader_html,
                language: body.language,
                lead_image_url: body.lead_image_url,
                item_type,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok((
        http::StatusCode::ACCEPTED,
        axum::Json(ExtensionSaveResponse {
            library_entry_id: result.library_entry_id.to_string(),
            status: result.status.to_string(),
            reader_url: document_reader_url(&state.config, &result.document_id),
        }),
    ))
}

/// Full-archive: URL + monolith(base64) + reader HTML + optional thumbnail(base64).
#[utoipa::path(
    post,
    path = "/api/v1/extension/full-archive",
    request_body = FullArchiveRequest,
    responses(
        (status = 202, description = "Archive accepted", body = ExtensionSaveResponse),
        (status = 400, description = "Extension save not configured"),
        (status = 401, description = "Authentication required"),
        (status = 422, description = "Validation error"),
    ),
    security(("api_token" = [])),
    tag = "Extension",
)]
pub async fn extension_full_archive(
    State(state): State<AppState>,
    RequireExtensionAccess(auth_user): RequireExtensionAccess,
    ValidatedJson(body): ValidatedJson<FullArchiveRequest>,
) -> Result<(http::StatusCode, axum::Json<ExtensionSaveResponse>), ApiError> {
    let save_ops = state
        .extension_save_ops
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest {
            message: "extension save not configured".to_string(),
        })?;

    let reader_html = body.reader_html.filter(|s| !s.is_empty());
    let item_type = parse_extension_document_type(body.item_type.as_deref())?;

    tracing::info!(
        url = %body.url,
        canonical_url = ?body.canonical_url,
        title = ?body.title,
        excerpt = ?body.excerpt,
        author = ?body.author,
        language = ?body.language,
        published_at = ?body.published_at,
        item_type = ?item_type,
        has_reader_html = reader_html.is_some(),
        "extension canonical_url received from link rel=canonical or og:url"
    );

    let result = save_ops
        .full_archive(
            auth_user.user_id,
            FullArchiveInput {
                url: body.url,
                canonical_url: body.canonical_url,
                title: body.title,
                reader_html,
                html_base64: body.html_base64,
                lead_image_url: body.lead_image_url,
                excerpt: body.excerpt,
                author: body.author,
                language: body.language,
                published_at: body.published_at.as_deref().and_then(parse_published_at),
                item_type,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok((
        http::StatusCode::ACCEPTED,
        axum::Json(ExtensionSaveResponse {
            library_entry_id: result.library_entry_id.to_string(),
            status: result.status.to_string(),
            reader_url: document_reader_url(&state.config, &result.document_id),
        }),
    ))
}
