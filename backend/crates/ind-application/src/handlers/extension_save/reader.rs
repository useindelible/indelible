use bytes::Bytes;
use chrono::Utc;

use crate::content_hash::compute_content_hash;
use crate::error::AppError;
use crate::handlers::provided_content::stage_provided_content;
use crate::repos::document_lifecycle::{
    MaterializeIdentity, MaterializeSideEffects, SaveSideEffectsFn, SaveToLibraryRequest,
};
use crate::repos::lifecycle_outbox::youtube_ingest_document_outbox;
use ind_domain::{ArchiveAssetKind, ContentSource, UserId};

use super::utils::resolved_canonical_url;
use super::{ExtensionSaveService, ReaderSaveInput, SaveResult};

impl ExtensionSaveService {
    /// Provided-content save: the browser already extracted readable HTML. The save does NOT
    /// enable content-gated AI (no redundant server render); instead the readable HTML is attached
    /// as a document-keyed asset and the embed + search reindex are enqueued once it exists.
    ///
    /// A YouTube URL is the exception (TASK-240): the browser-provided HTML is just the watch page,
    /// so it is discarded and `document.youtube_ingest` is enqueued atomically with the save to
    /// produce the transcript-enriched readable asset instead.
    pub async fn reader_save(
        &self,
        user_id: UserId,
        input: ReaderSaveInput,
    ) -> Result<SaveResult, AppError> {
        super::utils::validate_lead_image_url(&input.lead_image_url)?;
        let canonical_url = resolved_canonical_url(&input.url, input.canonical_url.as_deref());
        let is_youtube = crate::dispatch::is_youtube_url(&input.url);
        // Do not hash the discarded watch-page HTML for YouTube saves.
        let content_hash = (!is_youtube).then(|| compute_content_hash(&input.reader_html));
        let document = Self::build_url_document(
            user_id,
            &input.url,
            canonical_url,
            input.title,
            input.author,
            input.excerpt,
            input.language,
            input.lead_image_url,
            None,
            content_hash,
            input.item_type,
        );

        let url = input.url.clone();
        let staged_readable = if is_youtube {
            None
        } else {
            Some(
                stage_provided_content(
                    &self.object_storage,
                    user_id,
                    ArchiveAssetKind::ReadableHtml,
                    "text/html",
                    Bytes::from(input.reader_html.clone()),
                )
                .await?,
            )
        };
        let side_effects: Option<SaveSideEffectsFn> = if is_youtube {
            // YouTube: the readable asset comes from the ingest job; skip attaching the watch-page
            // HTML. The ingest job enqueues its own reindex/embed on completion.
            Some(Box::new(move |ctx| MaterializeSideEffects {
                events: Vec::new(),
                outbox: vec![youtube_ingest_document_outbox(
                    ctx.document.id,
                    user_id,
                    url.clone(),
                    Utc::now(),
                )],
            }))
        } else {
            #[expect(
                clippy::expect_used,
                reason = "staged_readable is set to Some on the non-YouTube branch a few lines above, the same condition that selects this else branch"
            )]
            let staged = staged_readable
                .clone()
                .expect("non-YouTube reader save stages readable HTML");
            Some(Box::new(move |ctx| MaterializeSideEffects {
                events: Vec::new(),
                outbox: vec![staged.outbox(ctx.document.id, user_id)],
            }))
        };

        let outcome = self
            .lifecycle
            .save_to_library(SaveToLibraryRequest {
                identity: MaterializeIdentity::Url {
                    document,
                    origin: None,
                },
                source: ContentSource::Extension,
                source_delivery_id: None,
                hide_deliveries: true,
                enqueue_engaged_ai: false,
                restore_policy: Default::default(),
                side_effects,
            })
            .await?;

        if let Some(staged) = staged_readable.as_ref() {
            self.attach_staged_document_asset(outcome.document.id, staged)
                .await?;
        }

        Ok(Self::save_result(&outcome))
    }
}
