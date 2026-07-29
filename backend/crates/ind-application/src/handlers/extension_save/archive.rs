use bytes::Bytes;
use chrono::Utc;

use crate::content_hash::compute_content_hash;
use crate::error::AppError;
use crate::handlers::provided_content::{StagedProvidedContent, stage_provided_content};
use crate::repos::document_lifecycle::{
    MaterializeIdentity, MaterializeSideEffects, SaveSideEffectsFn, SaveToLibraryRequest,
};
use crate::repos::lifecycle_outbox::{
    OutboxEntry, feed_prepare_document_outbox, search_reindex_document_outbox,
    youtube_ingest_document_outbox,
};
use ind_domain::{ArchiveAssetKind, ContentSource, UserId};

use super::utils::{base64_decode, resolved_canonical_url};
use super::{ExtensionSaveService, FullArchiveInput, SaveResult};

impl ExtensionSaveService {
    /// Full-archive save: the browser provides a monolith full-page capture and (usually) readable
    /// HTML. When readable HTML is present it is attached and the embed + reindex are enqueued
    /// (provided-content path); when it is absent, content-gated AI is enabled so the document
    /// preparation pipeline renders readable content from the URL. Monolith/pdf/screenshot archival
    /// follows the user's preferences.
    ///
    /// A YouTube URL is the exception (TASK-240): the browser-provided readable HTML (the watch
    /// page) is discarded and `document.youtube_ingest` is enqueued atomically with the save to
    /// produce the transcript-enriched readable asset. The monolith is only archived when enabled.
    pub async fn full_archive(
        &self,
        user_id: UserId,
        input: FullArchiveInput,
    ) -> Result<SaveResult, AppError> {
        super::utils::validate_lead_image_url(&input.lead_image_url)?;
        let canonical_url = resolved_canonical_url(&input.url, input.canonical_url.as_deref());
        let is_youtube = crate::dispatch::is_youtube_url(&input.url);
        let has_reader_html = input.reader_html.is_some();
        let archival = self
            .user_preferences_repo
            .get_archival(user_id)
            .await?
            .unwrap_or_default();
        let wants_monolith = archival.archive_formats.monolith;
        let wants_derived = archival.archive_formats.pdf || archival.archive_formats.screenshot;

        // The URL is rendered server-side when readable content must be produced
        // (no provided reader HTML) or when derived pdf/screenshot is requested.
        // Guard those cases at save time (SSRF); the renderer also pre-flights.
        let will_server_render = !is_youtube && (!has_reader_html || wants_derived);
        if will_server_render {
            self.url_guard.check_url(&input.url).await.map_err(|e| {
                AppError::Domain(ind_domain::DomainError::Validation {
                    field: "url".into(),
                    message: e.message().to_string(),
                })
            })?;
        }
        // Do not hash the discarded watch-page HTML for YouTube saves.
        let content_hash = if is_youtube {
            None
        } else {
            input.reader_html.as_deref().map(compute_content_hash)
        };
        let document = Self::build_url_document(
            user_id,
            &input.url,
            canonical_url,
            input.title,
            input.author,
            input.excerpt,
            input.language,
            input.lead_image_url,
            input.published_at,
            content_hash,
            input.item_type,
        );

        // The readable HTML is skipped for YouTube saves (watch page is discarded, ingest provides
        // the readable asset) and for reader-less archives (the engaged-AI prepare pipeline renders
        // it). The monolith is non-readable, so its attach job no-ops reindex/embed.
        let staged_monolith = if wants_monolith {
            let monolith_bytes = base64_decode(&input.html_base64)?;
            Some(
                stage_provided_content(
                    &self.object_storage,
                    user_id,
                    ArchiveAssetKind::Monolith,
                    "text/html",
                    monolith_bytes,
                )
                .await?,
            )
        } else {
            None
        };

        let staged_reader: Option<StagedProvidedContent> = if is_youtube {
            None
        } else if let Some(reader_html) = input.reader_html.as_deref() {
            Some(
                stage_provided_content(
                    &self.object_storage,
                    user_id,
                    ArchiveAssetKind::ReadableHtml,
                    "text/html",
                    Bytes::from(reader_html.to_owned()),
                )
                .await?,
            )
        } else {
            None
        };

        let youtube_url = is_youtube.then(|| input.url.clone());
        let prepare_url =
            (!is_youtube && has_reader_html && wants_derived).then(|| input.url.clone());
        let outbox_monolith = staged_monolith.clone();
        let outbox_reader = staged_reader.clone();
        let side_effects: SaveSideEffectsFn = Box::new(move |ctx| {
            let mut outbox: Vec<OutboxEntry> = Vec::new();
            if let Some(monolith) = outbox_monolith.as_ref() {
                outbox.push(monolith.outbox(ctx.document.id, user_id));
            }
            if let Some(reader) = outbox_reader.as_ref() {
                outbox.push(reader.outbox(ctx.document.id, user_id));
            }
            if let Some(url) = youtube_url.as_ref() {
                outbox.push(youtube_ingest_document_outbox(
                    ctx.document.id,
                    user_id,
                    url.clone(),
                    Utc::now(),
                ));
            } else if outbox_reader.is_none() {
                // Reader-less archive: the prepare pipeline (engaged AI) renders readable content,
                // but the monolith attach already reindexes the document; keep search current.
                outbox.push(search_reindex_document_outbox(ctx.document.id, Utc::now()));
            }
            if let Some(url) = prepare_url.as_ref() {
                outbox.push(feed_prepare_document_outbox(
                    ctx.document.id,
                    user_id,
                    url.clone(),
                    Utc::now(),
                ));
            }
            MaterializeSideEffects {
                events: Vec::new(),
                outbox,
            }
        });

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
                // YouTube routes to ingest (above), so it never needs the generic prepare pipeline.
                enqueue_engaged_ai: !has_reader_html && !is_youtube,
                restore_policy: Default::default(),
                side_effects: Some(side_effects),
            })
            .await?;

        if let Some(staged) = staged_monolith.as_ref() {
            self.attach_staged_document_asset(outcome.document.id, staged)
                .await?;
        }
        if let Some(staged) = staged_reader.as_ref() {
            self.attach_staged_document_asset(outcome.document.id, staged)
                .await?;
        }

        Ok(Self::save_result(&outcome))
    }
}
