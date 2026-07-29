//! Document reader HTTP surface for the document/feed/library model.
//!
//! Reads and capability writes are keyed by `document_id`. `GET /documents/{id}` returns the
//! reader read-model (distinguishing prepared-but-unsaved from saved); the readable bytes are
//! served via a presigned URL from `/documents/{id}/assets/{kind}`. Highlights and the single
//! note require completed readable content (422 otherwise); progress writes `user_document_state`
//! without requiring a Library entry. The canonical-reader-open flow itself is the Feed
//! `POST /feeds/deliveries/{id}/prepare` (TASK-231); this surface is what the reader loads after.
//! See docs/document-feed-library-architecture.md (Document Reader; API Shape).

pub(crate) mod dto;

pub(crate) mod entities;
pub(crate) mod highlights;
pub(crate) mod notes;
pub(crate) mod progress;
pub(crate) mod reader;
pub(crate) mod toc;

use axum::Router;
use axum::extract::{Path, State};
use axum::routing::{get, patch, post};
use ind_application::ports::DocumentReaderOperations;

use crate::error::ApiError;
use crate::extract::ValidatedJson;
use crate::middleware::{RequireDocumentAssetRead, RequireLibraryRead, RequireLibraryWrite};
use crate::response::{ApiResponse, EmptyResponse};
use crate::state::AppState;

pub(crate) use dto::{
    DocumentAssetResponse, DocumentNoteResponse, DocumentReaderAssetResponse,
    DocumentReaderResponse, DocumentReprocessResponse, DocumentUpsertNoteBody,
    UpdateDocumentProgressBody, parse_document_id,
};
pub use entities::list_document_entities;
pub use highlights::{create_document_highlight, list_document_highlights};
pub use notes::{get_document_note, upsert_document_note};
pub use progress::update_document_progress;
pub use reader::{get_document_asset, get_document_reader, reprocess_document};

// Reused DTOs so the document surface speaks the same shapes as the legacy item surface.
pub(crate) use crate::routes::highlights::dto::{
    CreateHighlightBody, HighlightListResponse, HighlightResponse, HighlightWithNoteResponse,
};

fn require_document_reader_ops(
    state: &AppState,
) -> Result<&dyn DocumentReaderOperations, ApiError> {
    state
        .document_reader_ops
        .as_deref()
        .ok_or(ApiError::ServiceUnavailable {
            message: "document reader service not configured".into(),
        })
}

pub fn document_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/documents/{document_id}", get(get_document_reader))
        .route(
            "/api/v1/documents/{document_id}/reprocess",
            post(reprocess_document),
        )
        .route(
            "/api/v1/documents/{document_id}/assets/{asset_kind}",
            get(get_document_asset),
        )
        .route(
            "/api/v1/documents/{document_id}/toc",
            get(toc::get_article_toc),
        )
        .route(
            "/api/v1/documents/{document_id}/entities",
            get(list_document_entities),
        )
        .route(
            "/api/v1/documents/{document_id}/highlights",
            get(list_document_highlights).post(create_document_highlight),
        )
        .route(
            "/api/v1/documents/{document_id}/note",
            get(get_document_note).put(upsert_document_note),
        )
        .route(
            "/api/v1/documents/{document_id}/progress",
            patch(update_document_progress),
        )
}
