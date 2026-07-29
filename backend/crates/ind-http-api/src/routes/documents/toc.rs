use axum::Json;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use http::HeaderValue;

use ind_application::ports::ArticleTocReadOutput;
use ind_html::ArticleTocStatus;

use crate::error::ApiError;
use crate::middleware::RequireLibraryRead;
use crate::routes::documents::dto::{
    ArticleTocEntryResponse, ArticleTocResponse, ArticleTocResponseStatus,
};
use crate::routes::documents::parse_document_id;
use crate::state::AppState;

/// Get the article table of contents.
///
/// Returns the derived outline for the document's readable content. `pending`
/// means the outline is being derived (or the content is not readable yet);
/// clients poll until `ready` or `none`.
#[utoipa::path(
    get,
    path = "/api/v1/documents/{document_id}/toc",
    params(
        ("document_id" = String, Path, description = "Document id with doc_ prefix"),
    ),
    responses(
        (status = 200, description = "Outline, or a pending/none status", body = ArticleTocResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Document not found"),
        (status = 503, description = "Table of contents service not configured"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    extensions(("x-indelible-permissions" = json!(["library:read"]))),
    tag = "Documents",
)]
pub async fn get_article_toc(
    RequireLibraryRead {
        principal: auth, ..
    }: RequireLibraryRead,
    State(state): State<AppState>,
    Path(document_id): Path<String>,
) -> Result<Response, ApiError> {
    let ops = state
        .article_toc_ops
        .as_deref()
        .ok_or_else(|| ApiError::ServiceUnavailable {
            message: "table of contents service not configured".into(),
        })?;
    let document_id = parse_document_id(&document_id)?;

    let output = ops
        .get_or_request(auth.user_id, document_id)
        .await
        .map_err(ApiError::from)?;

    let (body, cache_control) = match output {
        ArticleTocReadOutput::Available(stored) => {
            let status = match stored.toc.status {
                ArticleTocStatus::Ready => ArticleTocResponseStatus::Ready,
                ArticleTocStatus::None => ArticleTocResponseStatus::None,
            };
            let response = ArticleTocResponse {
                status,
                truncated: stored.toc.truncated,
                entries: stored
                    .toc
                    .entries
                    .into_iter()
                    .map(|entry| ArticleTocEntryResponse {
                        source_heading_index: entry.source_heading_index,
                        id: entry.id,
                        title: entry.title,
                        depth: entry.depth,
                        word_count: entry.word_count,
                    })
                    .collect(),
            };
            (response, "private, max-age=3600")
        }
        ArticleTocReadOutput::Pending => (
            ArticleTocResponse {
                status: ArticleTocResponseStatus::Pending,
                truncated: false,
                entries: Vec::new(),
            },
            "no-store",
        ),
    };

    let mut response = Json(body).into_response();
    response.headers_mut().insert(
        http::header::CACHE_CONTROL,
        HeaderValue::from_static(cache_control),
    );
    Ok(response)
}
