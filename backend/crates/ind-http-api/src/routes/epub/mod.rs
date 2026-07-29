mod dto;

use axum::body::Body;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use futures::TryStreamExt;
use http::{HeaderValue, StatusCode};
use ind_domain::ArchiveAssetKind;

pub use dto::{EpubMetadata, EpubTocEntry, EpubTocResponse};

use crate::error::ApiError;
use crate::middleware::RequireLibraryRead;
use crate::routes::documents::parse_document_id;
use crate::state::AppState;

/// Get EPUB table of contents and metadata.
///
/// Reads `epub_toc.json` from S3 via the Epub asset record.
#[utoipa::path(
    get,
    path = "/api/v1/documents/{document_id}/epub/toc",
    params(
        ("document_id" = String, Path, description = "Document id with doc_ prefix"),
    ),
    responses(
        (status = 200, description = "EPUB TOC and metadata", body = EpubTocResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Document or EPUB asset not found"),
        (status = 503, description = "Storage not configured"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:read"]))),
    tag = "EPUB",
)]
pub async fn get_epub_toc(
    RequireLibraryRead {
        principal: auth, ..
    }: RequireLibraryRead,
    State(state): State<AppState>,
    Path(document_id): Path<String>,
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

    let parsed_document_id = parse_document_id(&document_id)?;

    let asset = ops
        .get_completed_asset(auth.user_id, parsed_document_id, ArchiveAssetKind::Epub)
        .await
        .map_err(ApiError::from)?;

    let object_data = storage
        .get_object(&asset.s3_key)
        .await
        .map_err(ApiError::from)?;

    let bytes = object_data
        .body
        .try_fold(Vec::new(), |mut acc, chunk| async move {
            acc.extend_from_slice(&chunk);
            Ok(acc)
        })
        .await
        .map_err(|err| ApiError::Internal {
            message: format!("failed to read EPUB TOC object: {err}"),
        })?;
    let toc =
        serde_json::from_slice::<EpubTocResponse>(&bytes).map_err(|err| ApiError::Internal {
            message: format!("malformed EPUB TOC JSON: {err}"),
        })?;

    let mut response = Json(toc).into_response();
    response.headers_mut().insert(
        http::header::CACHE_CONTROL,
        HeaderValue::from_static("private, max-age=3600"),
    );
    Ok(response)
}

/// Get a single EPUB chapter by index.
///
/// Reads `epub_ch_{index}.html` from S3. The S3 key prefix is derived
/// from the Epub asset record's key by stripping the `epub_toc.json` suffix.
#[utoipa::path(
    get,
    path = "/api/v1/documents/{document_id}/epub/chapters/{chapter_index}",
    params(
        ("document_id" = String, Path, description = "Document id with doc_ prefix"),
        ("chapter_index" = u32, Path, description = "0-based chapter index"),
    ),
    responses(
        (status = 200, description = "Chapter HTML content", content_type = "text/html"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Document, EPUB asset, or chapter not found"),
        (status = 503, description = "Storage not configured"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:read"]))),
    tag = "EPUB",
)]
pub async fn get_epub_chapter(
    RequireLibraryRead {
        principal: auth, ..
    }: RequireLibraryRead,
    State(state): State<AppState>,
    Path((document_id, chapter_index)): Path<(String, u32)>,
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

    let parsed_document_id = parse_document_id(&document_id)?;

    let asset = ops
        .get_completed_asset(auth.user_id, parsed_document_id, ArchiveAssetKind::Epub)
        .await
        .map_err(ApiError::from)?;

    let prefix = asset
        .s3_key
        .strip_suffix("epub_toc.json")
        .ok_or_else(|| ApiError::Internal {
            message: "unexpected epub asset key format".to_string(),
        })?;
    let chapter_key = format!("{prefix}epub_ch_{chapter_index}.html");

    let object_data = storage
        .get_object(&chapter_key)
        .await
        .map_err(|_| ApiError::NotFound {
            entity: "Chapter",
            id: chapter_index.to_string(),
        })?;

    #[expect(
        clippy::expect_used,
        reason = "all headers are static literals and Body::from_stream is infallible, so the response always builds"
    )]
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(http::header::CONTENT_TYPE, "text/html; charset=utf-8")
        .header(http::header::CACHE_CONTROL, "private, max-age=3600")
        .body(Body::from_stream(object_data.body))
        .expect("valid response")
        .into_response();
    Ok(response)
}

pub fn epub_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/documents/{document_id}/epub/toc",
            get(get_epub_toc),
        )
        .route(
            "/api/v1/documents/{document_id}/epub/chapters/{chapter_index}",
            get(get_epub_chapter),
        )
}
