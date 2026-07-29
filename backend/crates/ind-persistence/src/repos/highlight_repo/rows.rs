use chrono::{DateTime, Utc};
use uuid::Uuid;

use ind_application::AppError;
use ind_domain::{
    DocumentId, Highlight, HighlightId, HighlightLocator, HighlightNote, HighlightNoteId,
    HighlightSourceLocator, Tag, TagId, UserId,
};

#[derive(sqlx::FromRow)]
pub(super) struct HighlightRow {
    pub(super) id: Uuid,
    pub(super) document_id: Uuid,
    pub(super) user_id: Uuid,
    pub(super) color: String,
    pub(super) text_content: String,
    pub(super) locator: Option<serde_json::Value>,
    pub(super) source_locator: Option<serde_json::Value>,
    pub(super) created_at: DateTime<Utc>,
    pub(super) updated_at: DateTime<Utc>,
}

impl TryFrom<HighlightRow> for Highlight {
    type Error = AppError;

    fn try_from(row: HighlightRow) -> Result<Self, Self::Error> {
        let locator: Option<HighlightLocator> = row
            .locator
            .map(serde_json::from_value)
            .transpose()
            .map_err(|e| AppError::Repository(Box::new(e)))?;
        let source_locator: Option<HighlightSourceLocator> = row
            .source_locator
            .map(serde_json::from_value)
            .transpose()
            .map_err(|e| AppError::Repository(Box::new(e)))?;
        Ok(Highlight {
            id: HighlightId::from_uuid(row.id),
            document_id: DocumentId::from_uuid(row.document_id),
            user_id: UserId::from_uuid(row.user_id),
            color: row.color,
            text_content: row.text_content,
            locator,
            source_locator,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(sqlx::FromRow)]
pub(super) struct HighlightNoteRow {
    pub(super) id: Uuid,
    pub(super) highlight_id: Uuid,
    pub(super) body: String,
    pub(super) created_at: DateTime<Utc>,
    pub(super) updated_at: DateTime<Utc>,
}

impl From<HighlightNoteRow> for HighlightNote {
    fn from(row: HighlightNoteRow) -> Self {
        HighlightNote {
            id: HighlightNoteId::from_uuid(row.id),
            highlight_id: HighlightId::from_uuid(row.highlight_id),
            body: row.body,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
pub(super) struct TagRow {
    pub(super) id: Uuid,
    pub(super) user_id: Uuid,
    pub(super) name: String,
    pub(super) color: Option<String>,
    pub(super) parent_id: Option<Uuid>,
    pub(super) created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
pub(super) struct HighlightTagRow {
    pub(super) highlight_id: Uuid,
    pub(super) id: Uuid,
    pub(super) user_id: Uuid,
    pub(super) name: String,
    pub(super) color: Option<String>,
    pub(super) parent_id: Option<Uuid>,
    pub(super) created_at: DateTime<Utc>,
}

impl From<TagRow> for Tag {
    fn from(row: TagRow) -> Self {
        Tag {
            id: TagId::from_uuid(row.id),
            user_id: UserId::from_uuid(row.user_id),
            name: row.name,
            color: row.color,
            parent_id: row.parent_id.map(TagId::from_uuid),
            created_at: row.created_at,
        }
    }
}
