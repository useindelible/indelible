//! Document reader read-model, content serving, and document-keyed authored capabilities
//! (highlights, the single note, reading progress). Highlights and notes require the document
//! to have its completed format-specific canonical asset; progress writes
//! `user_document_state` without requiring a Library entry.

use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use ind_domain::{
    ArchiveAssetKind, ArchiveAssetStatus, Document, DocumentAsset, DocumentId, DocumentNote,
    DocumentType, DomainError, EventOrigin, Highlight, HighlightId, NewHighlight, NewReadingEvent,
    ReadingPosition, UserDocumentState, UserId,
};

use futures::future::BoxFuture;

use crate::error::AppError;
use crate::event_intents::document_highlighted;
use crate::handlers::highlight::HighlightWithNote;
use crate::handlers::highlight::validation::{
    validate_color, validate_highlight_locators_for_document,
};
use crate::ports::{
    CreateHighlightRequest, DocumentReaderOperations, DocumentReaderView, DocumentReprocessOutput,
    HighlightCreation,
};
use crate::repos::document::DocumentRepository;
use crate::repos::document_asset::DocumentAssetRepository;
use crate::repos::document_note::DocumentNoteRepository;
use crate::repos::document_reprocess::DocumentReprocessRepository;
use crate::repos::event::MutationSideEffects;
use crate::repos::highlight::{HighlightRepository, HighlightWrite};
use crate::repos::library::LibraryRepository;
use crate::repos::lifecycle_outbox::search_reindex_document_outbox;
use crate::repos::user_document_state::{AppendOutcome, UserDocumentStateRepository};

const REPROCESS_COOLDOWN: Duration = Duration::from_secs(5 * 60);

pub struct DocumentReaderService {
    document_repo: Arc<dyn DocumentRepository>,
    state_repo: Arc<dyn UserDocumentStateRepository>,
    library_repo: Arc<dyn LibraryRepository>,
    highlight_repo: Arc<dyn HighlightRepository>,
    note_repo: Arc<dyn DocumentNoteRepository>,
    asset_repo: Arc<dyn DocumentAssetRepository>,
    reprocess_repo: Arc<dyn DocumentReprocessRepository>,
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
    ) -> Self {
        Self {
            document_repo,
            state_repo,
            library_repo,
            highlight_repo,
            note_repo,
            asset_repo,
            reprocess_repo,
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

    /// Durable annotations require the completed canonical asset for their document format so
    /// locators never anchor to an ephemeral or cross-format representation.
    async fn require_annotation_source(&self, document: &Document) -> Result<(), AppError> {
        let required = match document.document_type {
            DocumentType::Book => ArchiveAssetKind::Epub,
            DocumentType::Pdf => ArchiveAssetKind::Pdf,
            _ => ArchiveAssetKind::ReadableHtml,
        };
        if self
            .asset_repo
            .has_successful_asset(document.id, required)
            .await?
        {
            Ok(())
        } else {
            Err(AppError::Domain(DomainError::Validation {
                field: "document_id".into(),
                message: "document has no completed reader content yet; prepare it before \
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

    pub async fn get_completed_asset(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        kind: ArchiveAssetKind,
    ) -> Result<DocumentAsset, AppError> {
        self.require_document(user_id, document_id).await?;
        // Only serve a completed asset: exposing a pending/failed row would hand the client a
        // URL whose bytes may not exist yet.
        self.asset_repo
            .find_by_document_and_kind(document_id, kind)
            .await?
            .filter(|asset| asset.status == ArchiveAssetStatus::Completed)
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "DocumentAsset",
                    id: format!("{document_id}/{kind}"),
                })
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
        request: CreateHighlightRequest,
    ) -> Result<HighlightCreation, AppError> {
        let document = self.require_document(user_id, document_id).await?;
        self.require_annotation_source(&document).await?;
        validate_color(&request.color)?;
        validate_highlight_locators_for_document(
            document.document_type,
            request.locator.as_ref(),
            request.source_locator.as_ref(),
        )?;

        let new_highlight = NewHighlight {
            id: request.requested_id.unwrap_or_else(HighlightId::new),
            document_id,
            user_id,
            color: request.color,
            text_content: request.text_content,
            locator: request.locator,
            source_locator: request.source_locator,
        };
        let now = Utc::now();
        let effects = MutationSideEffects {
            events: vec![document_highlighted(user_id, document_id, new_highlight.id)],
            outbox: vec![search_reindex_document_outbox(document_id, now)],
        };

        match self
            .highlight_repo
            .create_for_document(&new_highlight, effects)
            .await?
        {
            HighlightWrite::Inserted(highlight) => Ok(HighlightCreation {
                highlight: *highlight,
                created: true,
            }),
            HighlightWrite::IdTaken => {
                match self
                    .highlight_repo
                    .get_by_id(new_highlight.id, user_id)
                    .await?
                {
                    Some(existing) => replay_or_conflict(existing, &new_highlight),
                    None => Err(highlight_id_conflict(new_highlight.id)),
                }
            }
        }
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
        let document = self.require_document(user_id, document_id).await?;
        self.require_annotation_source(&document).await?;
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
        position: Option<ReadingPosition>,
        origin: EventOrigin,
    ) -> Result<UserDocumentState, AppError> {
        self.require_document(user_id, document_id).await?;
        self.state_repo
            .record_progress(user_id, document_id, progress_percent, position, origin)
            .await
    }

    pub async fn append_reading_events(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        events: Vec<NewReadingEvent>,
    ) -> Result<AppendOutcome, AppError> {
        self.require_document(user_id, document_id).await?;
        self.state_repo
            .append_reading_events(user_id, document_id, &events)
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

    fn get_completed_asset(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        kind: ArchiveAssetKind,
    ) -> BoxFuture<'_, Result<DocumentAsset, AppError>> {
        Box::pin(self.get_completed_asset(user_id, document_id, kind))
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
        request: CreateHighlightRequest,
    ) -> BoxFuture<'_, Result<HighlightCreation, AppError>> {
        Box::pin(self.create_highlight(user_id, document_id, request))
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
        position: Option<ReadingPosition>,
        origin: EventOrigin,
    ) -> BoxFuture<'_, Result<UserDocumentState, AppError>> {
        Box::pin(self.update_progress(user_id, document_id, progress_percent, position, origin))
    }

    fn append_reading_events(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        events: Vec<NewReadingEvent>,
    ) -> BoxFuture<'_, Result<AppendOutcome, AppError>> {
        Box::pin(self.append_reading_events(user_id, document_id, events))
    }
}

/// `created_at` and `updated_at` are stamped server-side, so comparing them would make every
/// retry read as divergent; only client-supplied content decides replay versus conflict.
fn replay_or_conflict(
    existing: Highlight,
    requested: &NewHighlight,
) -> Result<HighlightCreation, AppError> {
    let same = existing.document_id == requested.document_id
        && existing.color == requested.color
        && existing.text_content == requested.text_content
        && existing.locator == requested.locator
        && existing.source_locator == requested.source_locator;
    if same {
        Ok(HighlightCreation {
            highlight: existing,
            created: false,
        })
    } else {
        Err(highlight_id_conflict(requested.id))
    }
}

fn highlight_id_conflict(id: HighlightId) -> AppError {
    AppError::Domain(DomainError::Conflict {
        entity: "Highlight",
        message: format!("highlight {id} already exists with different content"),
    })
}
