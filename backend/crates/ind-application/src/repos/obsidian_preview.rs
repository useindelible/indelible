use chrono::{DateTime, Utc};

use crate::error::AppError;
use ind_domain::{DocumentId, HighlightId, ItemType, LibraryEntryId, UserId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObsidianPreviewHighlight {
    pub id: HighlightId,
    pub text: String,
    pub color: String,
    pub created_at: DateTime<Utc>,
    pub note: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObsidianPreviewDocument {
    pub document_id: DocumentId,
    pub library_entry_id: LibraryEntryId,
    pub title: String,
    pub url: Option<String>,
    pub author: Option<String>,
    pub item_type: ItemType,
    pub lead_image_url: Option<String>,
    pub excerpt: Option<String>,
    pub tags: Vec<String>,
    pub highlights: Vec<ObsidianPreviewHighlight>,
}

#[async_trait::async_trait]
pub trait ObsidianPreviewRepository: Send + Sync {
    async fn load_document(
        &self,
        user_id: UserId,
        library_entry_id: LibraryEntryId,
    ) -> Result<Option<ObsidianPreviewDocument>, AppError>;
}
