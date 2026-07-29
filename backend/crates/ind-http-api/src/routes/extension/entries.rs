use super::library_alias::{
    library_context_response, library_entry_for_alias, patch_library_context_response,
};
use super::*;

pub(super) fn parse_extension_document_type(s: Option<&str>) -> Result<Option<ItemType>, ApiError> {
    let Some(s) = s else { return Ok(None) };
    s.parse::<ItemType>()
        .map(Some)
        .map_err(|_| ApiError::ValidationError {
            errors: vec![crate::error::FieldError {
                field: "item_type".into(),
                message: format!("unknown item type: {s}"),
            }],
        })
}

pub(super) fn parse_extension_entry_id(s: &str) -> Result<ind_domain::LibraryEntryId, ApiError> {
    s.parse::<ind_domain::LibraryEntryId>()
        .map_err(|_| ApiError::NotFound {
            entity: "SavedEntry",
            id: s.to_string(),
        })
}

pub(super) fn document_reader_url(
    config: &crate::state::AppConfig,
    document_id: &ind_domain::DocumentId,
) -> String {
    format!(
        "{}/reader/{}",
        config.frontend_url.trim_end_matches('/'),
        document_id
    )
}

#[utoipa::path(
    get,
    path = "/api/v1/extension/check-url",
    params(ExtensionCheckUrlParams),
    responses(
        (status = 200, description = "URL existence check result", body = ExtensionUrlCheckResponse),
        (status = 401, description = "Authentication required"),
        (status = 503, description = "Library service not configured"),
    ),
    security(("api_token" = [])),
    tag = "Extension",
)]
pub async fn extension_check_url(
    State(state): State<AppState>,
    RequireExtensionAccess(auth_user): RequireExtensionAccess,
    Query(params): Query<ExtensionCheckUrlParams>,
) -> Result<ApiResponse<ExtensionUrlCheckResponse>, ApiError> {
    let config = ind_ingest::CanonicalizationConfig::default();
    let canonical = match ind_ingest::canonicalize_url(&params.url, &config) {
        Ok(c) => c.into_string(),
        Err(_) => params.url.clone(),
    };

    let library_ops = state
        .library_ops
        .as_ref()
        .ok_or(ApiError::ServiceUnavailable {
            message: "library service not configured".into(),
        })?;

    let result = library_ops
        .check_url(auth_user.user_id, &canonical)
        .await
        .map_err(ApiError::from)?;

    match result {
        Some(found) => Ok(ApiResponse::new(ExtensionUrlCheckResponse {
            exists: true,
            library_entry_id: Some(found.entry.id.to_string()),
            document_id: Some(found.document.id.to_string()),
            reader_url: Some(document_reader_url(&state.config, &found.document.id)),
            title: Some(found.document.title),
            saved_at: Some(found.entry.saved_at),
            triage_state: Some(found.entry.triage_state.as_str().to_string()),
        })),
        None => Ok(ApiResponse::new(ExtensionUrlCheckResponse {
            exists: false,
            library_entry_id: None,
            document_id: None,
            reader_url: None,
            title: None,
            saved_at: None,
            triage_state: None,
        })),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/extension/entries/{library_entry_id}/highlights",
    params(("library_entry_id" = String, Path, description = "Saved entry ID")),
    responses(
        (status = 200, description = "Highlights for the saved entry", body = HighlightListResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Saved entry not found"),
        (status = 503, description = "Document reader service not configured"),
    ),
    security(("api_token" = [])),
    tag = "Extension",
)]
pub async fn extension_list_highlights(
    State(state): State<AppState>,
    RequireExtensionAccess(auth_user): RequireExtensionAccess,
    Path(library_entry_id): Path<String>,
) -> Result<crate::extract::Json<HighlightListResponse>, ApiError> {
    let entry_id = parse_extension_entry_id(&library_entry_id)?;
    let joined = library_entry_for_alias(&state, auth_user.user_id, entry_id).await?;
    let document_reader_ops =
        state
            .document_reader_ops
            .as_ref()
            .ok_or(ApiError::ServiceUnavailable {
                message: "document reader service not configured".into(),
            })?;

    let highlights = document_reader_ops
        .list_highlights(auth_user.user_id, joined.document.id)
        .await
        .map_err(ApiError::from)?;

    let count = highlights.len();
    let highlight_entries = highlights
        .into_iter()
        .map(HighlightWithNoteResponse::from_domain)
        .collect();

    Ok(crate::extract::Json(HighlightListResponse {
        highlights: highlight_entries,
        count,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/extension/entries/{library_entry_id}/highlights",
    params(("library_entry_id" = String, Path, description = "Saved entry ID")),
    request_body = ExtensionCreateHighlightBody,
    responses(
        (status = 201, description = "Created highlight", body = HighlightResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Saved entry not found"),
        (status = 422, description = "Validation error"),
        (status = 503, description = "Document reader service not configured"),
    ),
    security(("api_token" = [])),
    tag = "Extension",
)]
pub async fn extension_create_highlight(
    State(state): State<AppState>,
    RequireExtensionAccess(auth_user): RequireExtensionAccess,
    Path(library_entry_id): Path<String>,
    axum::Json(body): axum::Json<ExtensionCreateHighlightBody>,
) -> Result<(http::StatusCode, crate::extract::Json<HighlightResponse>), ApiError> {
    let entry_id = parse_extension_entry_id(&library_entry_id)?;
    let joined = library_entry_for_alias(&state, auth_user.user_id, entry_id).await?;

    let mut errors = Vec::new();
    if body.color.trim().is_empty() {
        errors.push(crate::error::FieldError {
            field: "color".into(),
            message: "must not be empty".into(),
        });
    }
    if body.text_content.trim().is_empty() {
        errors.push(crate::error::FieldError {
            field: "text_content".into(),
            message: "must not be empty".into(),
        });
    }
    if body.locator.is_none() && body.source_locator.is_none() {
        errors.push(crate::error::FieldError {
            field: "locator".into(),
            message: "at least one of locator or source_locator is required".into(),
        });
    }
    if !errors.is_empty() {
        return Err(ApiError::ValidationError { errors });
    }

    let document_reader_ops =
        state
            .document_reader_ops
            .as_ref()
            .ok_or(ApiError::ServiceUnavailable {
                message: "document reader service not configured".into(),
            })?;
    let highlight = document_reader_ops
        .create_highlight(
            auth_user.user_id,
            joined.document.id,
            body.color,
            body.text_content,
            body.locator.map(Into::into),
            body.source_locator.map(Into::into),
        )
        .await
        .map_err(ApiError::from)?;

    Ok((
        http::StatusCode::CREATED,
        crate::extract::Json(HighlightResponse::from_domain(highlight)),
    ))
}

/// Serve a document-owned asset for a saved extension entry.
#[utoipa::path(
    get,
    path = "/api/v1/extension/entries/{library_entry_id}/assets/{asset_kind}",
    params(
        ("library_entry_id" = String, Path, description = "Saved entry ID"),
        ("asset_kind" = String, Path, description = "Archive asset kind"),
    ),
    responses(
        (status = 200, description = "Presigned asset URL", body = crate::routes::documents::DocumentAssetResponse),
        (status = 400, description = "Unknown asset kind"),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Saved entry or asset not found"),
        (status = 503, description = "Document reader service not configured"),
    ),
    security(("api_token" = [])),
    tag = "Extension",
)]
pub async fn extension_get_entry_asset(
    State(state): State<AppState>,
    RequireExtensionAccess(auth_user): RequireExtensionAccess,
    Path((library_entry_id, asset_kind)): Path<(String, String)>,
) -> Result<ApiResponse<crate::routes::documents::DocumentAssetResponse>, ApiError> {
    let entry_id = parse_extension_entry_id(&library_entry_id)?;
    let kind: ind_domain::ArchiveAssetKind =
        asset_kind.parse().map_err(|_| ApiError::BadRequest {
            message: format!("unknown asset kind: {asset_kind}"),
        })?;
    let document_reader_ops =
        state
            .document_reader_ops
            .as_ref()
            .ok_or(ApiError::ServiceUnavailable {
                message: "document reader service not configured".into(),
            })?;
    let joined = library_entry_for_alias(&state, auth_user.user_id, entry_id).await?;
    let result = document_reader_ops
        .get_asset_url(auth_user.user_id, joined.document.id, kind)
        .await
        .map_err(ApiError::from)?;
    Ok(ApiResponse::new(
        crate::routes::documents::DocumentAssetResponse::from(result),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/extension/entries/{library_entry_id}",
    params(("library_entry_id" = String, Path, description = "Saved entry ID")),
    responses(
        (status = 200, description = "Saved entry context", body = ExtensionSavedEntryResponse),
        (status = 400, description = "Extension entry ops not configured"),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Saved entry not found"),
    ),
    security(("api_token" = [])),
    tag = "Extension",
)]
pub async fn extension_get_entry(
    State(state): State<AppState>,
    RequireExtensionAccess(auth_user): RequireExtensionAccess,
    Path(library_entry_id): Path<String>,
) -> Result<ApiResponse<ExtensionSavedEntryResponse>, ApiError> {
    let entry_id = parse_extension_entry_id(&library_entry_id)?;

    tracing::debug!(library_entry_id = %entry_id, "get extension saved entry context");

    let resp = library_context_response(&state, auth_user.user_id, entry_id).await?;
    Ok(ApiResponse::new(resp))
}

#[utoipa::path(
    patch,
    path = "/api/v1/extension/entries/{library_entry_id}",
    params(("library_entry_id" = String, Path, description = "Saved entry ID")),
    request_body = PatchExtensionEntryBody,
    responses(
        (status = 200, description = "Updated saved entry context", body = ExtensionSavedEntryResponse),
        (status = 400, description = "Extension entry ops not configured"),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Saved entry not found"),
        (status = 422, description = "Validation error"),
    ),
    security(("api_token" = [])),
    tag = "Extension",
)]
pub async fn extension_patch_entry(
    State(state): State<AppState>,
    RequireExtensionAccess(auth_user): RequireExtensionAccess,
    Path(library_entry_id): Path<String>,
    axum::Json(body): axum::Json<PatchExtensionEntryBody>,
) -> Result<ApiResponse<ExtensionSavedEntryResponse>, ApiError> {
    let entry_id = parse_extension_entry_id(&library_entry_id)?;

    let triage_state = body
        .triage_state
        .as_deref()
        .map(str::parse::<ind_domain::TriageState>);

    if let Some(Err(_)) = triage_state {
        return Err(ApiError::ValidationError {
            errors: vec![crate::error::FieldError {
                field: "triage_state".into(),
                message: format!(
                    "must be one of: {}",
                    ind_domain::TriageState::NAMES.join(", ")
                ),
            }],
        });
    }

    let req = PatchExtensionEntryRequest {
        triage_state: triage_state.transpose().ok().flatten(),
        is_favorite: body.is_favorite,
    };

    tracing::debug!(library_entry_id = %entry_id, "patch extension saved entry");

    let resp = patch_library_context_response(&state, auth_user.user_id, entry_id, req).await?;
    Ok(ApiResponse::new(resp))
}

// Readability may return date-only ("2024-03-26") or full RFC 3339 strings.
// We try RFC 3339 first; for date-only strings we append midnight UTC.
pub(super) fn parse_published_at(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    use chrono::{DateTime, NaiveDate, Utc};
    if let Ok(dt) = s.parse::<DateTime<Utc>>() {
        return Some(dt);
    }
    if let Ok(date) = s.parse::<NaiveDate>() {
        return Some(date.and_hms_opt(0, 0, 0)?.and_utc());
    }
    None
}

#[utoipa::path(
    put,
    path = "/api/v1/extension/entries/{library_entry_id}/note",
    params(("library_entry_id" = String, Path, description = "Saved entry ID")),
    request_body = ExtensionUpsertNoteBody,
    responses(
        (status = 200, description = "Upserted note, null when cleared", body = Option<ExtensionNoteResponse>),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Saved entry not found"),
        (status = 503, description = "Document reader service not configured"),
    ),
    security(("api_token" = [])),
    tag = "Extension",
)]
pub async fn extension_upsert_note(
    State(state): State<AppState>,
    RequireExtensionAccess(auth_user): RequireExtensionAccess,
    Path(library_entry_id): Path<String>,
    axum::Json(body): axum::Json<ExtensionUpsertNoteBody>,
) -> Result<axum::Json<Option<ExtensionNoteResponse>>, ApiError> {
    let entry_id = parse_extension_entry_id(&library_entry_id)?;
    let joined = library_entry_for_alias(&state, auth_user.user_id, entry_id).await?;

    tracing::debug!(library_entry_id = %entry_id, "upsert extension saved-entry note");

    let note_body = body.body;
    let note = if note_body.is_empty() {
        None
    } else {
        let document_reader_ops =
            state
                .document_reader_ops
                .as_ref()
                .ok_or(ApiError::ServiceUnavailable {
                    message: "document reader service not configured".into(),
                })?;
        Some(
            document_reader_ops
                .upsert_note(auth_user.user_id, joined.document.id, note_body)
                .await
                .map_err(ApiError::from)?,
        )
    };

    Ok(axum::Json(note.map(|n| ExtensionNoteResponse {
        id: n.id.to_string(),
        body: n.body,
        created_at: n.created_at,
        updated_at: n.updated_at,
    })))
}

#[utoipa::path(
    put,
    path = "/api/v1/extension/entries/{library_entry_id}/tags",
    params(("library_entry_id" = String, Path, description = "Saved entry ID")),
    request_body = ExtensionReplaceTagsBody,
    responses(
        (status = 200, description = "Replaced tag set", body = ExtensionReplaceTagsResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Saved entry not found"),
        (status = 422, description = "Validation error"),
        (status = 503, description = "Tag service not configured"),
    ),
    security(("api_token" = [])),
    tag = "Extension",
)]
pub async fn extension_replace_tags(
    State(state): State<AppState>,
    RequireExtensionAccess(auth_user): RequireExtensionAccess,
    Path(library_entry_id): Path<String>,
    axum::Json(body): axum::Json<ExtensionReplaceTagsBody>,
) -> Result<axum::Json<ExtensionReplaceTagsResponse>, ApiError> {
    let entry_id = parse_extension_entry_id(&library_entry_id)?;

    if body.tags.len() > 20 {
        return Err(ApiError::ValidationError {
            errors: vec![crate::error::FieldError {
                field: "tags".into(),
                message: "maximum 20 tags per saved entry".into(),
            }],
        });
    }

    let normalized: Vec<String> = body
        .tags
        .into_iter()
        .map(|t| t.trim().to_lowercase())
        .filter(|t| !t.is_empty())
        .collect();

    tracing::debug!(library_entry_id = %entry_id, tag_count = normalized.len(), "replace extension saved-entry tags");

    let joined = library_entry_for_alias(&state, auth_user.user_id, entry_id).await?;
    let tag_ops = state.tag_ops.as_ref().ok_or(ApiError::ServiceUnavailable {
        message: "tag service not configured".into(),
    })?;
    let tags = tag_ops
        .set_library_entry_tags(auth_user.user_id, entry_id, joined.document.id, normalized)
        .await
        .map_err(ApiError::from)?;

    let mut tag_names: Vec<String> = tags.into_iter().map(|t| t.name).collect();
    tag_names.sort();

    Ok(axum::Json(ExtensionReplaceTagsResponse { tags: tag_names }))
}
