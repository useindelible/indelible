use chrono::{DateTime, Utc};
use ind_application::AppError;
use ind_domain::{Document, DocumentId, DocumentOriginType, DocumentType, DomainError, UserId};
use uuid::Uuid;

pub(super) fn map_document_error(err: sqlx::Error) -> AppError {
    super::super::map_sqlx_error("Document", "document conflict", err)
}

pub(super) fn map_origin_error(err: sqlx::Error) -> AppError {
    super::super::map_sqlx_error("document_origin", "document origin conflict", err)
}

pub(super) fn parse_document_type(value: &str) -> Result<DocumentType, AppError> {
    value.parse::<DocumentType>().map_err(|_| {
        AppError::Domain(DomainError::InvariantViolation {
            message: format!("unknown document_type: {value}"),
        })
    })
}

pub(super) fn origin_type_to_str(origin_type: DocumentOriginType) -> &'static str {
    origin_type.as_str()
}

pub(crate) struct DocumentRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub document_type: String,
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

impl DocumentRow {
    pub(crate) fn into_document(self) -> Result<Document, AppError> {
        Ok(Document {
            id: DocumentId::from_uuid(self.id),
            user_id: UserId::from_uuid(self.user_id),
            document_type: parse_document_type(&self.document_type)?,
            canonical_url: self.canonical_url,
            original_url: self.original_url,
            content_hash: self.content_hash,
            title: self.title,
            author: self.author,
            excerpt: self.excerpt,
            published_at: self.published_at,
            language: self.language,
            domain: self.domain,
            lead_image_url: self.lead_image_url,
            thumbnail_url: self.thumbnail_url,
            word_count: self.word_count,
            reading_time_minutes: self.reading_time_minutes,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}
