//! Document model entities for the document/feed/library architecture.
//!
//! Source of truth: docs/document-feed-library-architecture.md. A `Document` is
//! one user's prepared/engaged canonical content identity.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use uuid::Uuid;

use crate::{ContentSource, DocumentId, EmailSenderId, UserId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    Article,
    Book,
    Email,
    Pdf,
    Tweet,
    Video,
    Podcast,
}

impl DocumentType {
    pub const NAMES: &'static [&'static str] = &[
        "article", "book", "email", "pdf", "tweet", "video", "podcast",
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Article => "article",
            Self::Book => "book",
            Self::Email => "email",
            Self::Pdf => "pdf",
            Self::Tweet => "tweet",
            Self::Video => "video",
            Self::Podcast => "podcast",
        }
    }
}

impl fmt::Display for DocumentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for DocumentType {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "article" => Ok(Self::Article),
            "book" => Ok(Self::Book),
            "email" => Ok(Self::Email),
            "pdf" => Ok(Self::Pdf),
            "tweet" => Ok(Self::Tweet),
            "video" => Ok(Self::Video),
            "podcast" => Ok(Self::Podcast),
            other => Err(format!("invalid document type: {other}")),
        }
    }
}

/// External origin kind adopted into a materialized document. `origin_id` references
/// the corresponding source table (see docs/document-feed-library-architecture.md,
/// document_origins). This is the no-URL idempotency layer for materialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentOriginType {
    FeedSourceEntry,
    ReadwiseImportItem,
    EmailMessage,
    ManualUpload,
}

impl DocumentOriginType {
    pub const NAMES: &'static [&'static str] = &[
        "feed_source_entry",
        "readwise_import_item",
        "email_message",
        "manual_upload",
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::FeedSourceEntry => "feed_source_entry",
            Self::ReadwiseImportItem => "readwise_import_item",
            Self::EmailMessage => "email_message",
            Self::ManualUpload => "manual_upload",
        }
    }
}

impl fmt::Display for DocumentOriginType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for DocumentOriginType {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "feed_source_entry" => Ok(Self::FeedSourceEntry),
            "readwise_import_item" => Ok(Self::ReadwiseImportItem),
            "email_message" => Ok(Self::EmailMessage),
            "manual_upload" => Ok(Self::ManualUpload),
            other => Err(format!("invalid document origin type: {other}")),
        }
    }
}

/// Deterministic, user-scoped `document_origins.origin_id` for no-URL content.
/// Stable across retries so re-ingesting the same email/import maps to one document through
/// `document_origins(user_id, origin_type, origin_id)`. A UUIDv5 over a namespaced name keeps
/// it collision-resistant without inventing a fake canonical URL or a separate identity table.
/// `key` must be the origin's own stable identity (e.g. a normalized RFC5322 Message-ID, a
/// provider-scoped email id, or a Readwise external id).
pub fn deterministic_origin_id(
    origin_type: DocumentOriginType,
    user_id: UserId,
    key: &str,
) -> Uuid {
    let name = format!(
        "indelible:{}:{}:{}",
        origin_type.as_str(),
        user_id.into_uuid(),
        key
    );
    Uuid::new_v5(&Uuid::NAMESPACE_URL, name.as_bytes())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: DocumentId,
    pub user_id: UserId,
    pub document_type: DocumentType,
    pub canonical_url: Option<String>,
    pub original_url: Option<String>,
    pub content_hash: Option<String>,
    pub title: String,
    pub author: Option<String>,
    pub excerpt: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub language: Option<String>,
    pub domain: Option<String>,
    pub lead_image_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub word_count: Option<i32>,
    pub reading_time_minutes: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Rounds up at 238 WPM, matching the renderer's readability path so every content type
/// (article, email, EPUB, PDF) derives reading time from one constant.
pub fn reading_time_minutes_from_words(word_count: i32) -> i32 {
    if word_count <= 0 {
        return 0;
    }
    ((word_count as f32) / 238.0).ceil() as i32
}

/// Composed provenance read model for a document. Deliberately NOT an enum-as-type on `documents`:
/// provenance is derived from `library_entries`, durable capability rows, and `document_origins`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentProvenance {
    pub document_id: DocumentId,
    pub is_saved: bool,
    pub library_source: Option<ContentSource>,
    pub origins: Vec<DocumentOriginType>,
    pub has_highlights: bool,
    pub has_note: bool,
    pub has_mila_session: bool,
}

impl DocumentProvenance {
    /// Whether the document is engaged enough for engagement-gated AI paths. Saving or any durable
    /// authored engagement (chat/highlight/note) counts; reading progress alone does not.
    pub fn is_engaged_for_ai(&self) -> bool {
        self.is_saved || self.has_highlights || self.has_note || self.has_mila_session
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDocumentState {
    pub user_id: UserId,
    pub document_id: DocumentId,
    pub progress_percent: Option<i32>,
    pub max_progress_percent: Option<i32>,
    // scroll_position is renderer/client-specific reader position data with no single
    // stable shape (EPUB CFI, percentage, pixel offset); covered by the documented
    // Value-field allowlist.
    pub scroll_position: Option<serde_json::Value>,
    pub chapter_locator: Option<String>,
    pub chapter_offset: Option<i32>,
    pub last_read_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub first_opened_at: Option<DateTime<Utc>>,
    pub last_opened_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// URL-backed materialization input. `canonical_url` is mandatory by construction, so
/// a no-URL document cannot be created through the URL-backed repository path.
#[derive(Debug, Clone)]
pub struct NewUrlDocument {
    pub id: DocumentId,
    pub user_id: UserId,
    pub document_type: DocumentType,
    pub canonical_url: String,
    pub original_url: Option<String>,
    pub content_hash: Option<String>,
    pub title: String,
    pub author: Option<String>,
    pub excerpt: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub language: Option<String>,
    pub domain: Option<String>,
    pub lead_image_url: Option<String>,
    pub thumbnail_url: Option<String>,
}

/// No-URL materialization input. There is intentionally no `canonical_url` field;
/// identity is `content_hash` when present, else the `document_origins` row.
#[derive(Debug, Clone)]
pub struct NewOriginDocument {
    pub id: DocumentId,
    pub user_id: UserId,
    pub document_type: DocumentType,
    pub content_hash: Option<String>,
    pub original_url: Option<String>,
    pub title: String,
    pub author: Option<String>,
    pub excerpt: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub language: Option<String>,
    pub domain: Option<String>,
    pub lead_image_url: Option<String>,
    pub thumbnail_url: Option<String>,
    /// Email-sender linkage. Set only for `Email` documents (an email has exactly one
    /// sender); `None` for every other origin-backed source.
    pub sender_id: Option<EmailSenderId>,
}
