//! Article table-of-contents lifecycle: derive the outline from stored readable
//! HTML and commit it against the readable asset's version stamp.
//!
//! Every object written here goes to a unique immutable content-addressed key —
//! never a fixed name (a fixed key would let a stale worker overwrite the
//! winner's bytes after losing the row-level compare-and-swap), and nothing is
//! ever deleted (content-addressed keys can be shared by concurrent writers, so
//! a loser's delete could destroy the winner's referenced object). Orphans are
//! accepted debris for a later sweep.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use ind_domain::{ArchiveAssetKind, ArchiveAssetStatus, DocumentId, NewDocumentAsset, UserId};
use ind_html::{ArticleToc, derive_article_toc, prepare_reader_html};

use crate::error::AppError;
use crate::repos::document::DocumentRepository;
use crate::repos::document_asset::{DocumentAssetRepository, PreparedReadableLocation};
use crate::storage::{ObjectStorage, get_object_string};

pub const ARTICLE_TOC_PAYLOAD_VERSION: u32 = 1;

/// The persisted `article_toc` payload: the derived outline plus the readable
/// version stamp it was derived from. A payload whose stamp no longer matches
/// the readable asset row is stale.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoredArticleToc {
    pub version: u32,
    pub source: ArticleTocSource,
    #[serde(flatten)]
    pub toc: ArticleToc,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArticleTocSource {
    pub readable_created_at: DateTime<Utc>,
}

#[derive(Debug)]
pub enum EnsureOutcome {
    Committed(StoredArticleToc),
    /// A reprocess refreshed the readable asset between observation and commit;
    /// the newer content's own ingest recomputes its ToC. Staged objects are
    /// left in place.
    LostRace,
    NoReadableHtml,
}

/// Content-addressed key for a prepared (anchor-injected) readable HTML copy.
/// Mirrors the provided-content staging convention: the directory is the hash
/// of the bytes it holds, so the key is immutable and collision-free.
pub fn prepared_readable_key(user_id: UserId, prepared_html: &str) -> String {
    let digest = hex::encode(Sha256::digest(prepared_html.as_bytes()));
    format!(
        "documents/prepared/{}/{digest}/readable_html.html",
        user_id.into_uuid()
    )
}

/// Content-addressed key for a serialized [`StoredArticleToc`] payload.
pub fn article_toc_key(user_id: UserId, payload: &[u8]) -> String {
    let digest = hex::encode(Sha256::digest(payload));
    format!(
        "documents/toc/{}/{digest}/article_toc.json",
        user_id.into_uuid()
    )
}

/// Derive and persist the ToC for a document's current readable HTML.
///
/// Legacy content (stored before anchor preparation existed) is re-prepared and
/// uploaded to a new immutable key; the readable asset row's pointer is swapped
/// under a compare-and-swap guarded by the observed `created_at`. Already
/// prepared content skips the swap. Both paths commit the ToC asset row against
/// the same observed stamp, so a concurrent reprocess always wins.
pub async fn ensure_article_toc(
    storage: &dyn ObjectStorage,
    assets: &dyn DocumentAssetRepository,
    documents: &dyn DocumentRepository,
    document_id: DocumentId,
) -> Result<EnsureOutcome, AppError> {
    let Some(document) = documents.find_by_id_global(document_id).await? else {
        return Ok(EnsureOutcome::NoReadableHtml);
    };
    let Some(readable) = assets
        .find_by_document_and_kind(document_id, ArchiveAssetKind::ReadableHtml)
        .await?
    else {
        return Ok(EnsureOutcome::NoReadableHtml);
    };
    if readable.status != ArchiveAssetStatus::Completed {
        return Ok(EnsureOutcome::NoReadableHtml);
    }

    let html = get_object_string(storage, &readable.s3_key).await?;
    let prepared = prepare_reader_html(&html).map_err(|err| AppError::Repository(Box::new(err)))?;

    let new_location = if prepared == html {
        None
    } else {
        let key = prepared_readable_key(document.user_id, &prepared);
        let upload = storage
            .upload(&key, "text/html", prepared.clone().into_bytes().into())
            .await?;
        Some(PreparedReadableLocation {
            s3_key: upload.key,
            size_bytes: upload.size_bytes,
        })
    };

    let toc = derive_article_toc(&prepared, &document.title);
    let stored = StoredArticleToc {
        version: ARTICLE_TOC_PAYLOAD_VERSION,
        source: ArticleTocSource {
            readable_created_at: readable.created_at,
        },
        toc,
    };
    let payload = serde_json::to_vec(&stored).map_err(|err| AppError::Repository(Box::new(err)))?;
    let toc_key = article_toc_key(document.user_id, &payload);
    let toc_upload = storage
        .upload(&toc_key, "application/json", payload.into())
        .await?;

    let committed = assets
        .commit_article_toc(
            document_id,
            readable.created_at,
            new_location,
            NewDocumentAsset {
                document_id,
                asset_kind: ArchiveAssetKind::ArticleToc,
                s3_key: toc_upload.key,
                s3_bucket: toc_upload.bucket,
                content_type: "application/json".to_string(),
                size_bytes: toc_upload.size_bytes,
                status: ArchiveAssetStatus::Completed,
                failed_reason: None,
            },
        )
        .await?;

    Ok(if committed {
        EnsureOutcome::Committed(stored)
    } else {
        EnsureOutcome::LostRace
    })
}

/// Best-effort ToC derivation for ingest arms: the outline is derived reader
/// metadata, so a failure here must never fail the content save that carries
/// it — log and move on (mirrors the reading-metrics convention).
pub async fn apply_article_toc(
    storage: Option<&dyn ObjectStorage>,
    assets: &dyn DocumentAssetRepository,
    documents: &dyn DocumentRepository,
    document_id: DocumentId,
) {
    let Some(storage) = storage else {
        return;
    };
    match ensure_article_toc(storage, assets, documents, document_id).await {
        Ok(EnsureOutcome::Committed(_)) | Ok(EnsureOutcome::NoReadableHtml) => {}
        Ok(EnsureOutcome::LostRace) => {
            tracing::info!(%document_id, "article toc commit lost a reprocess race; skipping");
        }
        Err(err) => {
            tracing::warn!(error = %err, %document_id, "failed to derive article toc");
        }
    }
}

pub fn toc_ensure_dedupe_key(document_id: DocumentId) -> String {
    format!(
        "{}:{document_id}",
        ind_domain::job_types::DOCUMENT_TOC_ENSURE
    )
}

/// Enqueue one deduped `document.toc.ensure` job. Safe to call on every ToC
/// read that finds a missing/stale outline: the outbox dedupe key collapses
/// request storms into a single pending job per document.
pub async fn enqueue_toc_ensure(
    outbox: &dyn crate::repos::outbox::JobOutboxRepository,
    document_id: DocumentId,
    now: DateTime<Utc>,
) -> Result<(), AppError> {
    let payload = serde_json::to_value(ind_domain::EnsureArticleTocJob { document_id })
        .map_err(|err| AppError::Repository(Box::new(err)))?;
    outbox
        .enqueue(
            ind_domain::job_types::DOCUMENT_TOC_ENSURE,
            payload,
            Some(toc_ensure_dedupe_key(document_id)),
            now,
        )
        .await?;
    Ok(())
}

/// Concrete [`crate::ports::ArticleTocOperations`]: resolves the stored outline
/// against the current readable version, enqueueing one deduped ensure job on a
/// miss or staleness.
pub struct ArticleTocReadService {
    documents: std::sync::Arc<dyn DocumentRepository>,
    assets: std::sync::Arc<dyn DocumentAssetRepository>,
    storage: std::sync::Arc<dyn ObjectStorage>,
    outbox: std::sync::Arc<dyn crate::repos::outbox::JobOutboxRepository>,
}

impl ArticleTocReadService {
    pub fn new(
        documents: std::sync::Arc<dyn DocumentRepository>,
        assets: std::sync::Arc<dyn DocumentAssetRepository>,
        storage: std::sync::Arc<dyn ObjectStorage>,
        outbox: std::sync::Arc<dyn crate::repos::outbox::JobOutboxRepository>,
    ) -> Self {
        Self {
            documents,
            assets,
            storage,
            outbox,
        }
    }
}

#[async_trait::async_trait]
impl crate::ports::ArticleTocOperations for ArticleTocReadService {
    async fn get_or_request(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<crate::ports::ArticleTocReadOutput, AppError> {
        use crate::ports::ArticleTocReadOutput;

        if self
            .documents
            .find_by_id(user_id, document_id)
            .await?
            .is_none()
        {
            return Err(AppError::Domain(ind_domain::DomainError::NotFound {
                entity: "document",
                id: document_id.to_string(),
            }));
        }

        let readable = self
            .assets
            .find_by_document_and_kind(document_id, ArchiveAssetKind::ReadableHtml)
            .await?
            .filter(|asset| asset.status == ArchiveAssetStatus::Completed);
        let Some(readable) = readable else {
            // Not readable yet: ingest owns producing both the content and its
            // ToC, so there is nothing to enqueue here.
            return Ok(ArticleTocReadOutput::Pending);
        };

        if let Some(toc_row) = self
            .assets
            .find_by_document_and_kind(document_id, ArchiveAssetKind::ArticleToc)
            .await?
        {
            match get_object_string(self.storage.as_ref(), &toc_row.s3_key).await {
                Ok(payload) => match serde_json::from_str::<StoredArticleToc>(&payload) {
                    Ok(stored) if stored.source.readable_created_at == readable.created_at => {
                        return Ok(ArticleTocReadOutput::Available(stored));
                    }
                    Ok(_) => {} // stale — fall through to re-enqueue
                    Err(err) => {
                        tracing::warn!(
                            error = %err, %document_id,
                            "stored article toc payload unparseable; re-deriving"
                        );
                    }
                },
                Err(err) => {
                    tracing::warn!(
                        error = %err, %document_id,
                        "stored article toc object unreadable; re-deriving"
                    );
                }
            }
        }

        enqueue_toc_ensure(self.outbox.as_ref(), document_id, Utc::now()).await?;
        Ok(ArticleTocReadOutput::Pending)
    }
}
