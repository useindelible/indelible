use async_trait::async_trait;

use crate::AppError;
use ind_domain::{DocumentId, PreparedItemContent};

#[async_trait]
pub trait PreparedContentProvider: Send + Sync {
    /// Load prepared content for a document (TASK-233 durable search indexing).
    async fn load_for_document(
        &self,
        document_id: DocumentId,
    ) -> Result<Option<PreparedItemContent>, AppError>;

    /// Plain readable text for a document keyed by `document_id` (TASK-233 durable search body).
    /// Used as a fallback by the search indexer for net-new feed-prepared documents whose
    /// `load_for_document` resolves to nothing but whose rendered `readable_html` lives in
    /// `archive_assets(document_id)`. The default returns `None`; adapters with
    /// document-addressable assets override it.
    async fn load_readable_text_for_document(
        &self,
        document_id: DocumentId,
    ) -> Result<Option<String>, AppError> {
        let _ = document_id;
        Ok(None)
    }
}
