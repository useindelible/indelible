use chrono::{DateTime, Utc};
use ind_application::repos::library::LibraryScopeCounts;
use ind_domain::{Document, DocumentType, LibraryEntry, LibraryEntryWithDocument, TriageState};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::FieldError;
use crate::extract::Validate;

/// Save a URL/manual/API item into the Library.
#[derive(Debug, Deserialize, ToSchema)]
pub struct SaveUrlBody {
    pub url: String,
    pub title: Option<String>,
    /// One of: article, book, email, pdf, tweet, video, podcast. Inferred from the URL when
    /// omitted.
    pub item_type: Option<String>,
}

impl Validate for SaveUrlBody {
    fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut errors = Vec::new();
        if self.url.trim().is_empty() {
            errors.push(FieldError {
                field: "url".into(),
                message: "must not be empty".into(),
            });
        }
        if let Some(ref t) = self.item_type
            && t.parse::<DocumentType>().is_err()
        {
            errors.push(FieldError {
                field: "item_type".into(),
                message: format!("must be one of: {}", DocumentType::NAMES.join(", ")),
            });
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Save a feed delivery into the Library.
#[derive(Debug, Deserialize, ToSchema)]
pub struct SaveFromDeliveryBody {
    pub delivery_id: String,
}

impl Validate for SaveFromDeliveryBody {
    fn validate(&self) -> Result<(), Vec<FieldError>> {
        if self.delivery_id.trim().is_empty() {
            return Err(vec![FieldError {
                field: "delivery_id".into(),
                message: "must not be empty".into(),
            }]);
        }
        Ok(())
    }
}

/// Set the triage state of a library entry.
#[derive(Debug, Deserialize, ToSchema)]
pub struct LibraryTriageBody {
    /// One of: inbox, later, archive.
    pub triage_state: String,
}

impl Validate for LibraryTriageBody {
    fn validate(&self) -> Result<(), Vec<FieldError>> {
        if self.triage_state.parse::<TriageState>().is_err() {
            return Err(vec![FieldError {
                field: "triage_state".into(),
                message: "must be one of: inbox, later, archive".into(),
            }]);
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct ListLibraryParams {
    /// Filter by triage state: inbox, later, archive.
    pub triage_state: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

/// Ad-hoc library query: the same filter-expression vocabulary as smart lists
/// (item_type/triage_state/domain/tags/...), evaluated without persisting a list.
/// A null/omitted expression returns the unfiltered library.
#[derive(Debug, Deserialize, ToSchema)]
pub struct LibraryQueryBody {
    #[serde(default)]
    #[schema(value_type = Option<crate::routes::smart_lists::dto::FilterExpressionNode>)]
    pub filter_expression: Option<ind_domain::FilterNode>,
    #[serde(default)]
    pub cursor: Option<String>,
    #[serde(default)]
    pub limit: Option<u32>,
}

impl Validate for LibraryQueryBody {
    fn validate(&self) -> Result<(), Vec<FieldError>> {
        if let Some(node) = &self.filter_expression
            && let Err(message) = node.validate()
        {
            return Err(vec![FieldError {
                field: "filter_expression".into(),
                message,
            }]);
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LibraryCountResponse {
    pub saved_count: i64,
}

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct LibraryCountsParams {
    /// Narrow the counts to one triage state: inbox, later, archive.
    pub triage_state: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LibraryItemTypeCountResponse {
    /// One of: article, book, email, pdf, tweet, video, podcast.
    pub item_type: String,
    pub count: i64,
}

/// Read-state and item-type breakdown of one library scope. Item types with no saved entries
/// are omitted, so the list is never a fixed-length vector of zeroes.
#[derive(Debug, Serialize, ToSchema)]
pub struct LibraryScopeCountsResponse {
    pub total: i64,
    pub unread: i64,
    pub reading: i64,
    pub done: i64,
    pub by_item_type: Vec<LibraryItemTypeCountResponse>,
}

impl From<LibraryScopeCounts> for LibraryScopeCountsResponse {
    fn from(counts: LibraryScopeCounts) -> Self {
        Self {
            total: counts.total(),
            unread: counts.unread,
            reading: counts.reading,
            done: counts.done,
            by_item_type: counts
                .by_item_type
                .into_iter()
                .map(|entry| LibraryItemTypeCountResponse {
                    item_type: entry.item_type.as_str().to_string(),
                    count: entry.count,
                })
                .collect(),
        }
    }
}

#[derive(Debug, ToSchema)]
pub struct LibraryUploadSchema {
    pub file: String,
    pub title: Option<String>,
}

/// Replace the Library tag set on a saved entry. Tag names are normalized server-side
/// (trimmed, lowercased); unknown names create tags.
#[derive(Debug, Deserialize, ToSchema)]
pub struct LibraryEntryTagsBody {
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LibraryEntryTagsResponse {
    pub tags: Vec<String>,
}

/// Result of emptying the Library trash: how many entries were permanently purged.
#[derive(Debug, Serialize, ToSchema)]
pub struct EmptyTrashResponse {
    pub purged: u64,
}

/// A saved Library entry joined with its document. Exposes both `library_entry_id` and
/// `document_id` per docs/document-feed-library-architecture.md (API Shape).
#[derive(Debug, Serialize, ToSchema)]
pub struct LibraryEntryResponse {
    pub object: &'static str,
    pub library_entry_id: String,
    pub document_id: String,
    pub document_type: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,
    /// Resolved short summary. Returns the stored AI summary when present,
    /// falling back to `excerpt`. `null` when neither is available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub published_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead_image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub word_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reading_time_minutes: Option<i32>,
    pub triage_state: String,
    pub is_favorite: bool,
    pub is_shortlisted: bool,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_delivery_id: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    pub saved_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

impl LibraryEntryResponse {
    pub(crate) fn from_parts(entry: LibraryEntry, document: Document) -> Self {
        Self {
            object: "library_entry",
            library_entry_id: entry.id.to_string(),
            document_id: document.id.to_string(),
            document_type: document.document_type.to_string(),
            title: document.title,
            url: document
                .original_url
                .or_else(|| document.canonical_url.clone()),
            canonical_url: document.canonical_url,
            domain: document.domain,
            author: document.author,
            summary: document
                .excerpt
                .as_deref()
                .map(str::trim)
                .filter(|excerpt| !excerpt.is_empty())
                .map(ToOwned::to_owned),
            excerpt: document.excerpt,
            published_at: document.published_at,
            language: document.language,
            lead_image_url: document.lead_image_url,
            thumbnail_url: document.thumbnail_url,
            word_count: document.word_count,
            reading_time_minutes: document.reading_time_minutes,
            triage_state: triage_state_str(entry.triage_state).to_string(),
            is_favorite: entry.is_favorite,
            is_shortlisted: entry.is_shortlisted,
            source: entry.source.as_str().to_string(),
            source_delivery_id: entry.source_delivery_id.map(|id| id.to_string()),
            saved_at: entry.saved_at,
            created_at: entry.created_at,
            updated_at: entry.updated_at,
        }
    }

    pub(crate) fn from_with_document(joined: LibraryEntryWithDocument) -> Self {
        Self::from_parts(joined.entry, joined.document)
    }

    pub(crate) fn with_summary(mut self, summary: Option<String>) -> Self {
        self.summary = summary;
        self
    }
}

fn triage_state_str(state: TriageState) -> &'static str {
    match state {
        TriageState::Inbox => "inbox",
        TriageState::Later => "later",
        TriageState::Archive => "archive",
    }
}
