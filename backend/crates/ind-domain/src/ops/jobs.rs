use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    ArchiveAssetKind, CollectionId, DocumentId, EmailSenderId, FeedDeliveryId, FeedSourceEntryId,
    FeedSourceId, ImportJobId, IntegrationConnectionId, JobOutboxId, LibraryEntryId, UserId,
};

/// Readable-content preparation for a feed-discovered document. Carries the document
/// identity, owner, and the URL to render. The worker writes the rendered output to
/// `archive_assets(document_id)`; the URL is the canonical URL resolved at materialize
/// time (the feed's inline content is never used).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrepareDocumentJob {
    pub document_id: DocumentId,
    pub user_id: UserId,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericJobEnvelope {
    pub outbox_id: JobOutboxId,
    pub job_type: String,
    pub payload: serde_json::Value,
    /// Carries the original outbox dedupe_key so the worker can derive a
    /// stable recovery_key for the universal recovery ledger. Optional and
    /// `#[serde(default)]` so envelopes serialized before TASK-193 still
    /// deserialize cleanly.
    #[serde(default)]
    pub dedupe_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoutubeIngestDocumentJob {
    pub document_id: DocumentId,
    pub user_id: UserId,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReprocessDocumentJob {
    pub document_id: DocumentId,
    pub user_id: UserId,
}

/// Attach browser/email-provided content as a document-keyed asset, then drive search reindex and
/// the content-gated embed. The bytes are never carried in the payload; `storage_key` references
/// the object already uploaded to a content-addressed staging key before the save transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachProvidedContentJob {
    pub document_id: DocumentId,
    pub user_id: UserId,
    pub asset_kind: ArchiveAssetKind,
    pub storage_key: String,
    pub storage_bucket: String,
    pub content_type: String,
    pub size_bytes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedDocumentJob {
    pub document_id: DocumentId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnsureArticleTocJob {
    pub document_id: DocumentId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummarizeDocumentJob {
    pub document_id: DocumentId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestTagsDocumentJob {
    pub document_id: DocumentId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractEntitiesDocumentJob {
    pub document_id: DocumentId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchReindexDocumentJob {
    pub document_id: DocumentId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchReindexAllJob {
    pub page_size: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_version: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedPollJob {
    pub source_id: FeedSourceId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedAutosaveJob {
    pub feed_delivery_id: FeedDeliveryId,
    pub source_entry_id: FeedSourceEntryId,
    pub user_id: UserId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collection_id: Option<CollectionId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailIngestJob {
    pub provider: String,
    pub provider_email_id: String,
    pub raw_payload: Vec<u8>,
    pub user_id: UserId,
    pub destination: String,
    pub ingest_log_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailUnsubscribeJob {
    pub user_id: UserId,
    pub sender_id: EmailSenderId,
}

/// Removes a purged account's object-storage keys. Enqueued inside the account
/// purge transaction (`job_outbox` has no foreign key to `users`, so the row
/// survives the delete) and processed with retries by the worker.
///
/// `prefixes` are derived from the user id and cover every current key scheme;
/// `residual_keys` are harvested from asset rows before they cascade away and
/// cover legacy shapes no prefix matches.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountStoragePurgeJob {
    pub user_id: UserId,
    pub prefixes: Vec<String>,
    pub residual_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotionExportDocumentJob {
    pub connection_id: IntegrationConnectionId,
    pub user_id: UserId,
    pub library_entry_id: LibraryEntryId,
    pub document_id: DocumentId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replaced_page_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsidianSyncConnectionJob {
    pub connection_id: IntegrationConnectionId,
    pub user_id: UserId,
    pub requested_by_user: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotionSyncConnectionJob {
    pub connection_id: IntegrationConnectionId,
    pub user_id: UserId,
    pub requested_by_user: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadwiseImportJob {
    pub import_job_id: ImportJobId,
    pub user_id: UserId,
    pub csv_key: Option<String>,
    pub archive_zip_key: Option<String>,
    pub opml_key: Option<String>,
}
