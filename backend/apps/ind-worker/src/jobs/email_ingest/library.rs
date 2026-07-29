use bytes::Bytes;
use chrono::Utc;
use ind_application::AppError;
use ind_application::handlers::feed_identity::{document_type_for, domain_from_url};
use ind_application::handlers::provided_content::{StagedProvidedContent, stage_provided_content};
use ind_application::repos::document_lifecycle::{
    MaterializeIdentity, MaterializeOrigin, MaterializeSideEffects, SaveToLibraryOutcome,
    SaveToLibraryRequest,
};
use ind_application::repos::event::MutationSideEffects;
use ind_application::repos::lifecycle_outbox::{OutboxEntry, search_reindex_document_outbox};
use ind_domain::{
    ArchiveAssetKind, ContentSource, DocumentId, DocumentOriginType, DocumentType, LibraryEntryId,
    NewOriginDocument, NewUrlDocument, TagSource, UserId, deterministic_origin_id,
    parse_from_header,
};
use ind_ingest::{CanonicalizationConfig, canonicalize_url};
use ind_integrations::email::{ContentMode, extract_primary_url, prepare_email_for_reader};
use tracing::info;

use super::assets::email_text_body_to_reader_html;
use crate::context::EmailJobDeps;

/// Stable `document_origins.origin_id` key for an inbound email (TASK-236). Prefer the
/// normalized RFC5322 Message-ID; fall back to a provider-scoped id so two providers cannot
/// collide for the same user. Used with `deterministic_origin_id(EmailMessage, user_id, key)`.
pub(super) fn email_origin_key(
    message_id: Option<&str>,
    provider: &str,
    provider_email_id: &str,
) -> String {
    match message_id.map(str::trim).filter(|m| !m.is_empty()) {
        Some(mid) => format!("message-id:{}", normalize_message_id(mid)),
        None => format!("provider:{provider}:email:{provider_email_id}"),
    }
}

/// Normalize a Message-ID so casing/bracket/whitespace variants dedupe to one origin id.
pub(super) fn normalize_message_id(message_id: &str) -> String {
    message_id
        .trim()
        .trim_start_matches('<')
        .trim_end_matches('>')
        .trim()
        .to_lowercase()
}

pub(super) struct EmailLibraryInput<'a> {
    pub(super) user_id: UserId,
    pub(super) subject: &'a str,
    pub(super) author: Option<&'a str>,
    pub(super) from_address: &'a str,
    pub(super) content_html: Option<&'a str>,
    pub(super) text_body: Option<&'a str>,
    pub(super) excerpt: Option<&'a str>,
    pub(super) language: Option<&'a str>,
    pub(super) origin_key: &'a str,
    pub(super) sender_id: Option<ind_domain::EmailSenderId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum LibraryEmailIngestAction {
    EmailBody,
    ExtractedUrl { url: String },
    EmailBodyNoLinkFound,
}

pub(super) fn decide_library_email_ingest_action(
    mode: ContentMode,
    html_body: Option<&str>,
    text_body: Option<&str>,
) -> LibraryEmailIngestAction {
    match mode {
        ContentMode::ModeA => LibraryEmailIngestAction::EmailBody,
        ContentMode::ModeB => match extract_primary_url(html_body, text_body) {
            Some(url) => LibraryEmailIngestAction::ExtractedUrl { url },
            None => LibraryEmailIngestAction::EmailBodyNoLinkFound,
        },
    }
}

/// Save an email body to the Library as a `documents` row keyed through `document_origins`
/// (no canonical URL). Idempotency is the deterministic email origin id, so re-ingesting the
/// same message resolves the same document and active library entry via `save_to_library`.
/// Readable HTML is attached document-keyed (from the HTML body, or synthesized from a text
/// body) so the saved document is reader/search/embed ready. (TASK-236)
pub(super) async fn save_email_as_document(
    ctx: &EmailJobDeps,
    input: EmailLibraryInput<'_>,
) -> Result<SaveToLibraryOutcome, AppError> {
    let now = Utc::now();
    let domain = parse_from_header(input.from_address)
        .0
        .domain()
        .map(|d| d.to_string());

    let document = NewOriginDocument {
        id: DocumentId::new(),
        user_id: input.user_id,
        document_type: DocumentType::Email,
        content_hash: None,
        original_url: None,
        title: input.subject.to_string(),
        author: input.author.map(String::from),
        excerpt: input.excerpt.map(String::from),
        published_at: Some(now),
        language: input.language.map(String::from),
        domain,
        lead_image_url: None,
        thumbnail_url: None,
        sender_id: input.sender_id,
    };

    let origin = MaterializeOrigin {
        origin_type: DocumentOriginType::EmailMessage,
        origin_id: deterministic_origin_id(
            DocumentOriginType::EmailMessage,
            input.user_id,
            input.origin_key,
        ),
    };

    // Require storage before the save: an outage must fail the email save before commit (the
    // email.ingest job retries) so no document is left without its readable asset.
    let storage = ctx.object_storage.as_ref().ok_or_else(|| {
        AppError::Repository(
            "object storage is not configured; cannot attach readable document asset".into(),
        )
    })?;

    let staged_original = match input.content_html {
        Some(html) => Some(
            stage_provided_content(
                storage,
                input.user_id,
                ArchiveAssetKind::OriginalHtml,
                "text/html",
                Bytes::from(html.to_owned()),
            )
            .await?,
        ),
        None => None,
    };

    let reader_html = match input.content_html {
        Some(html) => Some(prepare_email_for_reader(html)),
        None => input.text_body.map(email_text_body_to_reader_html),
    };
    let staged_readable: Option<StagedProvidedContent> = match reader_html {
        Some(reader_html) => Some(
            stage_provided_content(
                storage,
                input.user_id,
                ArchiveAssetKind::ReadableHtml,
                "text/html",
                Bytes::from(reader_html),
            )
            .await?,
        ),
        None => None,
    };

    let user_id = input.user_id;
    let side_effects = Box::new(
        move |ctx: &ind_application::repos::document_lifecycle::SaveContext| {
            let mut outbox: Vec<OutboxEntry> = Vec::new();
            if let Some(original) = staged_original.as_ref() {
                outbox.push(original.outbox(ctx.document.id, user_id));
            }
            if let Some(readable) = staged_readable.as_ref() {
                outbox.push(readable.outbox(ctx.document.id, user_id));
            }
            MaterializeSideEffects {
                events: Vec::new(),
                outbox,
            }
        },
    );

    let outcome = ctx
        .lifecycle
        .save_to_library(SaveToLibraryRequest {
            identity: MaterializeIdentity::Origin { document, origin },
            source: ContentSource::Email,
            source_delivery_id: None,
            hide_deliveries: false,
            enqueue_engaged_ai: false,
            restore_policy: Default::default(),
            side_effects: Some(side_effects),
        })
        .await?;

    info!(
        document_id = %outcome.document.id,
        user_id = %input.user_id,
        "saved email body to library as document"
    );

    Ok(outcome)
}

/// Save a URL extracted from an email to the Library through the document lifecycle. URL-backed
/// content dedupes on `(user_id, canonical_url)`; readable preparation/embedding is enqueued by
/// the content-gated AI builder (enqueue_engaged_ai: true). (TASK-236)
pub(super) async fn enqueue_url_save(
    ctx: &EmailJobDeps,
    user_id: UserId,
    url: &str,
    title: &str,
) -> Result<SaveToLibraryOutcome, AppError> {
    let canonical = match canonicalize_url(url, &CanonicalizationConfig::default()) {
        Ok(c) => c.into_string(),
        Err(_) => url.to_string(),
    };
    let document_type = document_type_for(ind_application::dispatch::infer_item_type_for_url(url));

    let document = NewUrlDocument {
        id: DocumentId::new(),
        user_id,
        document_type,
        canonical_url: canonical,
        original_url: Some(url.to_string()),
        content_hash: None,
        title: title.to_string(),
        author: None,
        excerpt: None,
        published_at: None,
        language: None,
        domain: domain_from_url(url),
        lead_image_url: None,
        thumbnail_url: None,
    };

    let outcome = ctx
        .lifecycle
        .save_to_library(SaveToLibraryRequest {
            identity: MaterializeIdentity::Url {
                document,
                origin: None,
            },
            source: ContentSource::Email,
            source_delivery_id: None,
            hide_deliveries: true,
            enqueue_engaged_ai: true,
            restore_policy: Default::default(),
            side_effects: Some(Box::new(|ctx| MaterializeSideEffects {
                events: Vec::new(),
                outbox: vec![search_reindex_document_outbox(ctx.document.id, Utc::now())],
            })),
        })
        .await?;

    info!(
        document_id = %outcome.document.id,
        user_id = %user_id,
        url = %url,
        "saved URL from email to library"
    );

    Ok(outcome)
}

const NO_LINK_FOUND_TAG: &str = "no-link-found";

/// Tag a saved library entry whose Mode-B email had no extractable link. The tag attaches to the
/// library entry (TASK-235 library-entry-keyed tags), not the legacy item.
pub(super) async fn apply_no_link_found_tag(
    ctx: &EmailJobDeps,
    user_id: UserId,
    library_entry_id: LibraryEntryId,
) -> Result<(), AppError> {
    let tag = ctx
        .tag_repo
        .find_or_create_by_name(user_id, NO_LINK_FOUND_TAG)
        .await?;
    ctx.tag_repo
        .replace_for_library_entry_with_source(
            user_id,
            library_entry_id,
            &[tag.id],
            TagSource::Import,
            MutationSideEffects::none(),
        )
        .await?;
    Ok(())
}
