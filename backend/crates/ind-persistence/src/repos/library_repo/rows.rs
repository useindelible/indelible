use chrono::{DateTime, Utc};
use ind_application::AppError;
use ind_domain::{
    ContentSource, Document, DocumentId, DocumentType, DomainError, FeedDeliveryId, LibraryEntry,
    LibraryEntryId, LibraryEntryWithDocument, TriageState, UserId,
};
use uuid::Uuid;

pub(super) fn map_library_error(err: sqlx::Error) -> AppError {
    super::super::map_sqlx_error("library_entry", "library entry conflict", err)
}

fn parse_triage_state(value: &str) -> Result<TriageState, AppError> {
    value
        .parse::<TriageState>()
        .map_err(|_| invariant("triage_state", value))
}

pub(super) fn parse_content_source(value: &str) -> Result<ContentSource, AppError> {
    value
        .parse::<ContentSource>()
        .map_err(|_| invariant("source", value))
}

pub(super) fn parse_document_type(value: &str) -> Result<DocumentType, AppError> {
    value
        .parse::<DocumentType>()
        .map_err(|_| invariant("document_type", value))
}

fn invariant(field: &str, value: &str) -> AppError {
    AppError::Domain(DomainError::InvariantViolation {
        message: format!("unknown {field}: {value}"),
    })
}

pub(super) struct LibraryEntryRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub document_id: Uuid,
    pub saved_at: DateTime<Utc>,
    pub triage_state: String,
    pub is_favorite: bool,
    pub is_shortlisted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub source: String,
    pub source_delivery_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl LibraryEntryRow {
    pub(super) fn into_entry(self) -> Result<LibraryEntry, AppError> {
        Ok(LibraryEntry {
            id: LibraryEntryId::from_uuid(self.id),
            user_id: UserId::from_uuid(self.user_id),
            document_id: DocumentId::from_uuid(self.document_id),
            saved_at: self.saved_at,
            triage_state: parse_triage_state(&self.triage_state)?,
            is_favorite: self.is_favorite,
            is_shortlisted: self.is_shortlisted,
            deleted_at: self.deleted_at,
            source: parse_content_source(&self.source)?,
            source_delivery_id: self.source_delivery_id.map(FeedDeliveryId::from_uuid),
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

/// Column projection that maps onto `LibraryWithDocRow` (le.* plus d.* aliased to `doc_*`).
/// Callers supply their own FROM/JOIN/WHERE; the aliases here must match the struct fields so
/// `FromRow`/`build_query_as` decode correctly. Shared by `library_query` and the collection/tag
/// contents listings (TASK-235).
pub(crate) const LIBRARY_DOC_COLUMNS: &str = "le.id, le.user_id, le.document_id, le.saved_at, le.triage_state, le.is_favorite, \
     le.is_shortlisted, le.deleted_at, le.source, le.source_delivery_id, le.created_at, \
     le.updated_at, d.document_type AS doc_document_type, d.canonical_url AS doc_canonical_url, \
     d.original_url AS doc_original_url, d.content_hash AS doc_content_hash, d.title AS doc_title, \
     d.author AS doc_author, d.excerpt AS doc_excerpt, d.published_at AS doc_published_at, \
     d.language AS doc_language, d.domain AS doc_domain, d.lead_image_url AS doc_lead_image_url, \
     d.thumbnail_url AS doc_thumbnail_url, d.word_count AS doc_word_count, \
     d.reading_time_minutes AS doc_reading_time_minutes, d.created_at AS doc_created_at, \
     d.updated_at AS doc_updated_at";

/// Shared library-entry-with-document projection. Reused by `library_query` (smart-list
/// evaluation, via `QueryBuilder`/`FromRow`) and by collection/tag contents listings, so the
/// `LibraryEntryWithDocument` mapping lives in one place (TASK-235).
#[derive(sqlx::FromRow)]
pub(crate) struct LibraryWithDocRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub document_id: Uuid,
    pub saved_at: DateTime<Utc>,
    pub triage_state: String,
    pub is_favorite: bool,
    pub is_shortlisted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub source: String,
    pub source_delivery_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub doc_document_type: String,
    pub doc_canonical_url: Option<String>,
    pub doc_original_url: Option<String>,
    pub doc_content_hash: Option<String>,
    pub doc_title: String,
    pub doc_author: Option<String>,
    pub doc_excerpt: Option<String>,
    pub doc_published_at: Option<DateTime<Utc>>,
    pub doc_language: Option<String>,
    pub doc_domain: Option<String>,
    pub doc_lead_image_url: Option<String>,
    pub doc_thumbnail_url: Option<String>,
    pub doc_word_count: Option<i32>,
    pub doc_reading_time_minutes: Option<i32>,
    pub doc_created_at: DateTime<Utc>,
    pub doc_updated_at: DateTime<Utc>,
}

/// `LibraryWithDocRow` plus the membership `added_at`, used by collection/tag contents listings to
/// keyset-paginate by when the entry was added to the collection/tag (TASK-235).
#[derive(sqlx::FromRow)]
pub(crate) struct LibraryEntryLinkRow {
    #[sqlx(flatten)]
    pub entry: LibraryWithDocRow,
    pub link_added_at: DateTime<Utc>,
}

impl LibraryWithDocRow {
    pub(crate) fn into_with_document(self) -> Result<LibraryEntryWithDocument, AppError> {
        let document = Document {
            id: DocumentId::from_uuid(self.document_id),
            user_id: UserId::from_uuid(self.user_id),
            document_type: parse_document_type(&self.doc_document_type)?,
            canonical_url: self.doc_canonical_url,
            original_url: self.doc_original_url,
            content_hash: self.doc_content_hash,
            title: self.doc_title,
            author: self.doc_author,
            excerpt: self.doc_excerpt,
            published_at: self.doc_published_at,
            language: self.doc_language,
            domain: self.doc_domain,
            lead_image_url: self.doc_lead_image_url,
            thumbnail_url: self.doc_thumbnail_url,
            word_count: self.doc_word_count,
            reading_time_minutes: self.doc_reading_time_minutes,
            created_at: self.doc_created_at,
            updated_at: self.doc_updated_at,
        };

        let entry = LibraryEntry {
            id: LibraryEntryId::from_uuid(self.id),
            user_id: UserId::from_uuid(self.user_id),
            document_id: DocumentId::from_uuid(self.document_id),
            saved_at: self.saved_at,
            triage_state: parse_triage_state(&self.triage_state)?,
            is_favorite: self.is_favorite,
            is_shortlisted: self.is_shortlisted,
            deleted_at: self.deleted_at,
            source: parse_content_source(&self.source)?,
            source_delivery_id: self.source_delivery_id.map(FeedDeliveryId::from_uuid),
            created_at: self.created_at,
            updated_at: self.updated_at,
        };

        Ok(LibraryEntryWithDocument { entry, document })
    }
}
