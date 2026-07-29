//! Document reader read-model, content serving, and document-keyed authored capabilities
//! (highlights, the single note, reading progress). Highlights and notes require the document
//! to have a completed readable asset (canonical rendered content); progress writes
//! `user_document_state` without requiring a Library entry. See
//! docs/document-feed-library-architecture.md (Document Reader; User highlights or notes an
//! unsaved feed delivery; Reading progress).

use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use ind_domain::{
    ArchiveAssetKind, ArchiveAssetStatus, Document, DocumentId, DocumentNote, DomainError,
    Highlight, HighlightId, HighlightLocator, HighlightSourceLocator, NewHighlight,
    UserDocumentState, UserId,
};

use futures::future::BoxFuture;

use crate::error::AppError;
use crate::event_intents::document_highlighted;
use crate::handlers::highlight::HighlightWithNote;
use crate::handlers::highlight::validation::{
    validate_color, validate_highlight_locators_for_document,
};
use crate::ports::{
    DocumentAssetWithUrl, DocumentReaderOperations, DocumentReaderView, DocumentReprocessOutput,
};
use crate::repos::document::DocumentRepository;
use crate::repos::document_asset::DocumentAssetRepository;
use crate::repos::document_note::DocumentNoteRepository;
use crate::repos::document_reprocess::DocumentReprocessRepository;
use crate::repos::event::MutationSideEffects;
use crate::repos::highlight::HighlightRepository;
use crate::repos::library::LibraryRepository;
use crate::repos::lifecycle_outbox::search_reindex_document_outbox;
use crate::repos::user_document_state::UserDocumentStateRepository;
use crate::storage::ObjectStorage;

const PRESIGNED_URL_EXPIRY: Duration = Duration::from_secs(3600);
const REPROCESS_COOLDOWN: Duration = Duration::from_secs(5 * 60);

pub struct DocumentReaderService {
    document_repo: Arc<dyn DocumentRepository>,
    state_repo: Arc<dyn UserDocumentStateRepository>,
    library_repo: Arc<dyn LibraryRepository>,
    highlight_repo: Arc<dyn HighlightRepository>,
    note_repo: Arc<dyn DocumentNoteRepository>,
    asset_repo: Arc<dyn DocumentAssetRepository>,
    reprocess_repo: Arc<dyn DocumentReprocessRepository>,
    storage: Arc<dyn ObjectStorage>,
}

impl DocumentReaderService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        document_repo: Arc<dyn DocumentRepository>,
        state_repo: Arc<dyn UserDocumentStateRepository>,
        library_repo: Arc<dyn LibraryRepository>,
        highlight_repo: Arc<dyn HighlightRepository>,
        note_repo: Arc<dyn DocumentNoteRepository>,
        asset_repo: Arc<dyn DocumentAssetRepository>,
        reprocess_repo: Arc<dyn DocumentReprocessRepository>,
        storage: Arc<dyn ObjectStorage>,
    ) -> Self {
        Self {
            document_repo,
            state_repo,
            library_repo,
            highlight_repo,
            note_repo,
            asset_repo,
            reprocess_repo,
            storage,
        }
    }

    async fn require_document(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<Document, AppError> {
        self.document_repo
            .find_by_id(user_id, document_id)
            .await?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "Document",
                    id: document_id.to_string(),
                })
            })
    }

    /// Durable annotations require canonical rendered content. Reject (422) when the document has
    /// no completed `readable_html` asset so locators never anchor to ephemeral feed preview HTML.
    async fn require_readable(&self, document_id: DocumentId) -> Result<(), AppError> {
        if self
            .asset_repo
            .has_successful_asset(document_id, ArchiveAssetKind::ReadableHtml)
            .await?
        {
            Ok(())
        } else {
            Err(AppError::Domain(DomainError::Validation {
                field: "document_id".into(),
                message: "document has no rendered readable content yet; prepare it before \
                          highlighting or noting"
                    .into(),
            }))
        }
    }

    pub async fn get_reader(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<DocumentReaderView, AppError> {
        let document = self.require_document(user_id, document_id).await?;
        // Reader availability outranks open-tracking: a failed open record must not fail the GET.
        if let Err(err) = self
            .state_repo
            .record_document_opened(user_id, document_id)
            .await
        {
            tracing::warn!(%document_id, error = %err, "failed to record document open");
        }
        let state = self.state_repo.find(user_id, document_id).await?;
        let library_entry_id = self
            .library_repo
            .find_active_by_document(user_id, document_id)
            .await?
            .map(|entry| entry.id);
        let assets = self.asset_repo.find_by_document(document_id).await?;
        Ok(DocumentReaderView {
            document,
            state,
            library_entry_id,
            assets,
        })
    }

    pub async fn get_asset_url(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        kind: ArchiveAssetKind,
    ) -> Result<DocumentAssetWithUrl, AppError> {
        self.require_document(user_id, document_id).await?;
        // Only serve a completed asset: presigning a pending/failed row would hand the client a
        // URL whose bytes may not exist yet.
        let asset = self
            .asset_repo
            .find_by_document_and_kind(document_id, kind)
            .await?
            .filter(|asset| asset.status == ArchiveAssetStatus::Completed)
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "DocumentAsset",
                    id: format!("{document_id}/{kind}"),
                })
            })?;
        let download_url = self
            .storage
            .presigned_url(&asset.s3_key, PRESIGNED_URL_EXPIRY)
            .await?;
        Ok(DocumentAssetWithUrl {
            asset,
            download_url,
        })
    }

    pub async fn reprocess_document(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<DocumentReprocessOutput, AppError> {
        let document = self.require_document(user_id, document_id).await?;
        let source_url = document
            .canonical_url
            .clone()
            .or_else(|| document.original_url.clone());
        if source_url.is_none() {
            let assets = self.asset_repo.find_by_document(document_id).await?;
            let has_upload_source = assets.iter().any(|asset| {
                asset.asset_kind == ArchiveAssetKind::OriginalUpload
                    && asset.status == ArchiveAssetStatus::Completed
                    && matches!(
                        asset.content_type.as_str(),
                        "application/pdf" | "application/epub+zip"
                    )
            });
            if !has_upload_source {
                return Err(AppError::Domain(DomainError::Validation {
                    field: "document_id".into(),
                    message: "document has no URL or uploaded PDF/EPUB source to reprocess".into(),
                }));
            }
        }

        let admission = self
            .reprocess_repo
            .admit(
                ind_domain::ReprocessDocumentJob {
                    document_id,
                    user_id,
                },
                Utc::now(),
                REPROCESS_COOLDOWN,
            )
            .await?;

        Ok(DocumentReprocessOutput {
            queued: admission.queued,
            job_type: ind_domain::job_types::DOCUMENT_REPROCESS.into(),
            retry_after_seconds: admission.retry_after_seconds,
        })
    }

    pub async fn create_highlight(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        color: String,
        text_content: String,
        locator: Option<HighlightLocator>,
        source_locator: Option<HighlightSourceLocator>,
    ) -> Result<Highlight, AppError> {
        let document = self.require_document(user_id, document_id).await?;
        self.require_readable(document_id).await?;
        validate_color(&color)?;
        validate_highlight_locators_for_document(
            document.document_type,
            locator.as_ref(),
            source_locator.as_ref(),
        )?;

        let new_highlight = NewHighlight {
            id: HighlightId::new(),
            document_id,
            user_id,
            color,
            text_content,
            locator,
            source_locator,
        };
        let now = Utc::now();
        let effects = MutationSideEffects {
            events: vec![document_highlighted(user_id, document_id, new_highlight.id)],
            outbox: vec![search_reindex_document_outbox(document_id, now)],
        };
        self.highlight_repo
            .create_for_document(&new_highlight, effects)
            .await
    }

    pub async fn list_highlights(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<Vec<HighlightWithNote>, AppError> {
        self.require_document(user_id, document_id).await?;
        let highlights = self
            .highlight_repo
            .list_by_document(document_id, user_id)
            .await?;
        let highlight_ids: Vec<HighlightId> = highlights.iter().map(|hl| hl.id).collect();
        let mut tags_by_highlight = self
            .highlight_repo
            .list_tags_for_highlights(&highlight_ids, user_id)
            .await?;

        let mut result = Vec::with_capacity(highlights.len());
        for highlight in highlights {
            let note = self.highlight_repo.get_note(highlight.id, user_id).await?;
            let tags = tags_by_highlight.remove(&highlight.id).unwrap_or_default();
            result.push(HighlightWithNote {
                highlight,
                note,
                tags,
            });
        }
        Ok(result)
    }

    pub async fn get_note(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<Option<DocumentNote>, AppError> {
        self.require_document(user_id, document_id).await?;
        self.note_repo.find_by_document(user_id, document_id).await
    }

    pub async fn upsert_note(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        body: String,
    ) -> Result<DocumentNote, AppError> {
        self.require_document(user_id, document_id).await?;
        self.require_readable(document_id).await?;
        let now = Utc::now();
        self.note_repo
            .upsert_for_document(
                user_id,
                document_id,
                &body,
                vec![search_reindex_document_outbox(document_id, now)],
            )
            .await
    }

    pub async fn update_progress(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        progress_percent: i32,
        chapter_locator: Option<String>,
        chapter_offset: Option<i32>,
    ) -> Result<UserDocumentState, AppError> {
        self.require_document(user_id, document_id).await?;
        self.state_repo
            .record_progress(
                user_id,
                document_id,
                progress_percent,
                chapter_locator,
                chapter_offset,
            )
            .await
    }
}

impl DocumentReaderOperations for DocumentReaderService {
    fn get_reader(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> BoxFuture<'_, Result<DocumentReaderView, AppError>> {
        Box::pin(self.get_reader(user_id, document_id))
    }

    fn get_asset_url(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        kind: ArchiveAssetKind,
    ) -> BoxFuture<'_, Result<DocumentAssetWithUrl, AppError>> {
        Box::pin(self.get_asset_url(user_id, document_id, kind))
    }

    fn reprocess_document(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> BoxFuture<'_, Result<DocumentReprocessOutput, AppError>> {
        Box::pin(self.reprocess_document(user_id, document_id))
    }

    fn create_highlight(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        color: String,
        text_content: String,
        locator: Option<HighlightLocator>,
        source_locator: Option<HighlightSourceLocator>,
    ) -> BoxFuture<'_, Result<Highlight, AppError>> {
        Box::pin(self.create_highlight(
            user_id,
            document_id,
            color,
            text_content,
            locator,
            source_locator,
        ))
    }

    fn list_highlights(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> BoxFuture<'_, Result<Vec<HighlightWithNote>, AppError>> {
        Box::pin(self.list_highlights(user_id, document_id))
    }

    fn get_note(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> BoxFuture<'_, Result<Option<DocumentNote>, AppError>> {
        Box::pin(self.get_note(user_id, document_id))
    }

    fn upsert_note(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        body: String,
    ) -> BoxFuture<'_, Result<DocumentNote, AppError>> {
        Box::pin(self.upsert_note(user_id, document_id, body))
    }

    fn update_progress(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        progress_percent: i32,
        chapter_locator: Option<String>,
        chapter_offset: Option<i32>,
    ) -> BoxFuture<'_, Result<UserDocumentState, AppError>> {
        Box::pin(self.update_progress(
            user_id,
            document_id,
            progress_percent,
            chapter_locator,
            chapter_offset,
        ))
    }
}
