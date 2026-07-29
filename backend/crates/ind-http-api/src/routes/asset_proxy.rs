use axum::Router;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use http::StatusCode;
use ind_domain::{ArchiveAssetKind, DocumentId, UserId};

use crate::error::ApiError;
use crate::middleware::AssetAccess;
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/assets/{user_id}/avatars/{filename}",
    params(
        ("user_id" = String, Path, description = "User ID with usr_ prefix"),
        ("filename" = String, Path, description = "Avatar filename (e.g. avatar.jpg)"),
    ),
    responses(
        (status = 200, description = "Avatar binary stream", content_type = "application/octet-stream"),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Avatar not found or not owned by caller"),
        (status = 503, description = "Storage not configured"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Asset Proxy",
)]
pub async fn stream_avatar(
    asset_access: AssetAccess,
    State(state): State<AppState>,
    Path((user_id_str, filename)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    let storage = state
        .storage
        .as_ref()
        .ok_or_else(|| ApiError::ServiceUnavailable {
            message: "storage not configured".to_string(),
        })?;

    let owner_id: UserId = user_id_str.parse().map_err(|_| ApiError::NotFound {
        entity: "Avatar",
        id: user_id_str.clone(),
    })?;

    if asset_access.user_id != owner_id {
        return Err(ApiError::NotFound {
            entity: "Avatar",
            id: user_id_str,
        });
    }

    let avatar_key = format!("{owner_id}/avatars/{filename}");
    if !crate::validation::avatar_key_belongs_to_user(&owner_id, &avatar_key) {
        return Err(ApiError::NotFound {
            entity: "Avatar",
            id: avatar_key,
        });
    }

    let object_data = storage
        .get_object(&avatar_key)
        .await
        .map_err(ApiError::from)?;
    build_stream_response(object_data)
}

#[utoipa::path(
    get,
    path = "/api/v1/assets/documents/{document_id}/{asset_kind}",
    params(
        ("document_id" = String, Path, description = "Document ID with doc_ prefix"),
        ("asset_kind" = String, Path, description = "Document asset kind (e.g. thumbnail)"),
    ),
    responses(
        (status = 200, description = "Document asset binary stream", content_type = "application/octet-stream"),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Document or asset not found"),
        (status = 503, description = "Storage or document reader service not configured"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "Asset Proxy",
)]
pub async fn stream_document_asset(
    asset_access: AssetAccess,
    State(state): State<AppState>,
    Path((document_id_str, asset_kind_str)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    let storage = state
        .storage
        .as_ref()
        .ok_or_else(|| ApiError::ServiceUnavailable {
            message: "storage not configured".to_string(),
        })?;
    let ops = state
        .document_reader_ops
        .as_deref()
        .ok_or_else(|| ApiError::ServiceUnavailable {
            message: "document reader service not configured".to_string(),
        })?;

    let document_id: DocumentId = document_id_str.parse().map_err(|_| ApiError::NotFound {
        entity: "Document",
        id: document_id_str.clone(),
    })?;
    let kind: ArchiveAssetKind = asset_kind_str.parse().map_err(|_| ApiError::BadRequest {
        message: format!("unknown asset kind: {asset_kind_str}"),
    })?;

    let asset_with_url = ops
        .get_asset_url(asset_access.user_id, document_id, kind)
        .await
        .map_err(ApiError::from)?;
    let object_data = storage
        .get_object(&asset_with_url.asset.s3_key)
        .await
        .map_err(ApiError::from)?;
    build_stream_response_with_kind(object_data, Some(asset_with_url.asset.asset_kind))
}

fn build_stream_response(
    object_data: ind_application::storage::ObjectData,
) -> Result<Response, ApiError> {
    build_stream_response_with_kind(object_data, None)
}

fn build_stream_response_with_kind(
    object_data: ind_application::storage::ObjectData,
    kind: Option<ArchiveAssetKind>,
) -> Result<Response, ApiError> {
    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header(http::header::CONTENT_TYPE, &object_data.content_type)
        .header(http::header::CONTENT_LENGTH, object_data.content_length)
        .header(http::header::CACHE_CONTROL, "private, max-age=3600");
    for (name, value) in extra_response_headers_for(kind, &object_data.content_type) {
        builder = builder.header(name, value);
    }
    let response = builder
        .body(Body::from_stream(object_data.body))
        .map_err(|error| ApiError::Internal {
            message: format!("failed to build asset response: {error}"),
        })?
        .into_response();
    Ok(response)
}

/// Security headers to attach to asset responses based on asset kind.
/// `OriginalHtml` and an HTML `OriginalUpload` carry unsanitized user/sender
/// HTML; forcing `Content-Disposition: attachment` makes browsers download
/// rather than render them in our origin. `ReadableHtml` is sanitized by ammonia
/// but still gets `X-Content-Type-Options: nosniff` for defense in depth.
fn extra_response_headers_for(
    kind: Option<ArchiveAssetKind>,
    content_type: &str,
) -> Vec<(http::HeaderName, http::HeaderValue)> {
    let Some(kind) = kind else {
        return Vec::new();
    };
    let nosniff = || {
        (
            http::header::HeaderName::from_static("x-content-type-options"),
            http::HeaderValue::from_static("nosniff"),
        )
    };
    let attachment = |filename: &'static str| {
        (
            http::header::CONTENT_DISPOSITION,
            http::HeaderValue::from_static(filename),
        )
    };
    match kind {
        ArchiveAssetKind::OriginalHtml => {
            vec![
                nosniff(),
                attachment("attachment; filename=\"original.html\""),
            ]
        }
        ArchiveAssetKind::OriginalUpload => {
            // The original upload is an archive copy, never rendered in-app (the
            // in-app copy is the sanitized ReadableHtml asset). nosniff always;
            // for an HTML original also force download so raw uploaded HTML can't
            // execute as a document in our origin.
            let mut headers = vec![nosniff()];
            if is_html_content_type(content_type) {
                headers.push(attachment("attachment; filename=\"original_upload.html\""));
            }
            headers
        }
        ArchiveAssetKind::ReadableHtml => vec![nosniff()],
        _ => Vec::new(),
    }
}

fn is_html_content_type(content_type: &str) -> bool {
    content_type
        .split(';')
        .next()
        .map(str::trim)
        .is_some_and(|mime| mime.eq_ignore_ascii_case("text/html"))
}

pub fn asset_proxy_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/assets/{user_id}/avatars/{filename}",
            get(stream_avatar),
        )
        .route(
            "/api/v1/assets/documents/{document_id}/{asset_kind}",
            get(stream_document_asset),
        )
}
