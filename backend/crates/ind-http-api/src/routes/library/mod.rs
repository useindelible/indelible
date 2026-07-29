//! Library HTTP surface for the document/feed/library model.
//!
//! Reads go through `library_entries JOIN documents` (never the feed firehose); saves go through
//! the atomic `DocumentLifecycle::save_to_library` lifecycle.

pub(crate) mod dto;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use ind_application::export_summary::DocumentSummarySource;
use ind_application::ports::{
    LibraryOperations, LibraryUploadOperations, SaveUrlRequest, SmartListOperations,
};
use ind_application::repos::{Cursor, Page};
use ind_domain::{
    Document, DocumentType, FeedDeliveryId, LibraryEntry, LibraryEntryId, LibraryEntryWithDocument,
    TriageState,
};

use crate::error::{ApiError, FieldError};
use crate::extract::ValidatedJson;
use crate::middleware::{RequireLibraryRead, RequireLibraryWrite};
use crate::response::{ApiResponse, EmptyResponse, PaginatedResponse};
use crate::state::AppState;

pub(crate) mod core;
pub(crate) mod mutations;
pub(crate) mod upload;

pub use core::{
    count_library, get_library_entry, library_counts, list_library, query_library,
    save_from_delivery, save_url,
};
pub use dto::*;
pub use mutations::{
    delete_library_entry, empty_trash, get_entry_tags, list_trash, purge_entry, restore_entry,
    set_entry_tags, toggle_favorite, toggle_shortlist, triage_entry,
};
pub use upload::upload_file;

fn require_library_ops(state: &AppState) -> Result<&dyn LibraryOperations, ApiError> {
    state
        .library_ops
        .as_deref()
        .ok_or(ApiError::ServiceUnavailable {
            message: "library service not configured".into(),
        })
}

fn require_library_upload_ops(state: &AppState) -> Result<&dyn LibraryUploadOperations, ApiError> {
    state
        .library_upload_ops
        .as_deref()
        .ok_or(ApiError::ServiceUnavailable {
            message: "library upload service not configured".into(),
        })
}

fn require_smart_list_ops(state: &AppState) -> Result<&dyn SmartListOperations, ApiError> {
    state
        .smart_list_ops
        .as_deref()
        .ok_or(ApiError::ServiceUnavailable {
            message: "smart list service not configured".into(),
        })
}

fn parse_entry_id(raw: &str) -> Result<LibraryEntryId, ApiError> {
    raw.parse().map_err(|_| ApiError::NotFound {
        entity: "LibraryEntry",
        id: raw.to_string(),
    })
}

pub(crate) async fn library_entry_response_from_parts(
    state: &AppState,
    entry: LibraryEntry,
    document: Document,
) -> Result<LibraryEntryResponse, ApiError> {
    let summary = if let Some(provider) = state.export_summary_provider.as_ref() {
        provider
            .summary_for_document(document.id, document.excerpt.as_deref())
            .await
            .map_err(ApiError::from)?
    } else {
        normalized_summary(document.excerpt.as_deref())
    };
    Ok(LibraryEntryResponse::from_parts(entry, document).with_summary(summary))
}

pub(crate) async fn library_entry_response(
    state: &AppState,
    joined: LibraryEntryWithDocument,
) -> Result<LibraryEntryResponse, ApiError> {
    // Build from the joined row, not its parts: the parts constructor cannot
    // carry fields that live on the join, such as the ingest failure reason.
    let summary = if let Some(provider) = state.export_summary_provider.as_ref() {
        provider
            .summary_for_document(joined.document.id, joined.document.excerpt.as_deref())
            .await
            .map_err(ApiError::from)?
    } else {
        normalized_summary(joined.document.excerpt.as_deref())
    };
    Ok(LibraryEntryResponse::from_with_document(joined).with_summary(summary))
}

pub(crate) async fn library_entry_responses(
    state: &AppState,
    joined: Vec<LibraryEntryWithDocument>,
) -> Result<Vec<LibraryEntryResponse>, ApiError> {
    let Some(provider) = state.export_summary_provider.as_ref() else {
        return Ok(joined
            .into_iter()
            .map(LibraryEntryResponse::from_with_document)
            .collect());
    };

    let sources: Vec<_> = joined
        .iter()
        .map(|item| DocumentSummarySource {
            document_id: item.document.id,
            excerpt: item.document.excerpt.clone(),
        })
        .collect();
    let mut summaries = provider
        .summaries_for_documents(&sources)
        .await
        .map_err(ApiError::from)?;

    Ok(joined
        .into_iter()
        .map(|item| {
            let summary = summaries
                .remove(&item.document.id)
                .unwrap_or_else(|| normalized_summary(item.document.excerpt.as_deref()));
            LibraryEntryResponse::from_with_document(item).with_summary(summary)
        })
        .collect())
}

fn normalized_summary(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

pub fn library_routes(max_upload_bytes: usize) -> Router<AppState> {
    let body_limit = max_upload_bytes.saturating_add(1024 * 1024);

    Router::new()
        .route("/api/v1/library", get(list_library).post(save_url))
        .route("/api/v1/library/uploads", post(upload_file))
        .route("/api/v1/library/query", post(query_library))
        .route("/api/v1/library/from-delivery", post(save_from_delivery))
        .route("/api/v1/library/count", get(count_library))
        .route("/api/v1/library/counts", get(library_counts))
        .route("/api/v1/library/trash", get(list_trash))
        .route("/api/v1/library/trash/empty", post(empty_trash))
        .route(
            "/api/v1/library/{library_entry_id}",
            get(get_library_entry).delete(delete_library_entry),
        )
        .route(
            "/api/v1/library/{library_entry_id}/restore",
            post(restore_entry),
        )
        .route(
            "/api/v1/library/{library_entry_id}/purge",
            post(purge_entry),
        )
        .route(
            "/api/v1/library/{library_entry_id}/tags",
            get(get_entry_tags).put(set_entry_tags),
        )
        .route(
            "/api/v1/library/{library_entry_id}/triage",
            post(triage_entry),
        )
        .route(
            "/api/v1/library/{library_entry_id}/favorite",
            post(toggle_favorite),
        )
        .route(
            "/api/v1/library/{library_entry_id}/shortlist",
            post(toggle_shortlist),
        )
        .layer(DefaultBodyLimit::max(body_limit))
}
