//! Library entry entities for the document/feed/library architecture.
//!
//! Source of truth: docs/document-feed-library-architecture.md (library_entries).
//! A `LibraryEntry` is one user's saved relationship to a `Document`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    ContentSource, Document, DocumentId, FeedDeliveryId, LibraryEntryId, TriageState, UserId,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryEntry {
    pub id: LibraryEntryId,
    pub user_id: UserId,
    pub document_id: DocumentId,
    pub saved_at: DateTime<Utc>,
    pub triage_state: TriageState,
    pub is_favorite: bool,
    pub is_shortlisted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub source: ContentSource,
    pub source_delivery_id: Option<FeedDeliveryId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Library reads always join `documents` (AC #4): a saved entry always references a
/// live document, so the document is non-optional here.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryEntryWithDocument {
    pub entry: LibraryEntry,
    pub document: Document,
}
