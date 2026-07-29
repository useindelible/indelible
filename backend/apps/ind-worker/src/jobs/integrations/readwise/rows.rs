use std::collections::HashMap;

use chrono::Utc;
use ind_application::AppError;
use ind_application::handlers::feed_identity::document_type_for;
use ind_application::repos::document_lifecycle::{
    MaterializeIdentity, MaterializeOrigin, MaterializeSideEffects, SaveContext, SaveSideEffectsFn,
    SaveToLibraryOutcome, SaveToLibraryRequest,
};
use ind_application::repos::event::MutationSideEffects;
use ind_application::repos::lifecycle_outbox::youtube_ingest_document_outbox;
use ind_domain::{
    CanonicalizationConfig, ContentSource, DocumentId, DocumentOriginType, NewOriginDocument,
    NewUrlDocument, TagSource, TriageState, UserId, canonicalize_url, deterministic_origin_id,
};
use tracing::warn;

use super::assets::attach_zip_asset_to_document;
use super::parse::{
    detect_item_type, ext_to_item_type, extract_domain, location_to_triage,
    parse_python_list_with_errors, parse_readwise_date,
};
use super::types::{ProcessCsvOutcome, ProcessRowResult, ReadwiseCsvRow, RowDiagnostics, ZipEntry};
use crate::context::IntegrationJobDeps;
use crate::jobs::ai::enqueue_document_embed_if_engaged;
use crate::jobs::search::enqueue_search_reindex_document;

/// Deterministic `document_origins` key for a no-URL Readwise row (TASK-236). Idempotency derives
/// from the Readwise id, replacing the legacy `items.external_id` dedup.
fn readwise_origin_id(user_id: UserId, readwise_id: &str) -> MaterializeOrigin {
    MaterializeOrigin {
        origin_type: DocumentOriginType::ReadwiseImportItem,
        origin_id: deterministic_origin_id(
            DocumentOriginType::ReadwiseImportItem,
            user_id,
            &format!("readwise:{readwise_id}"),
        ),
    }
}

/// Returns Imported for a fresh save, Duplicate when the content already has an active library
/// entry (TASK-236). URL rows dedupe on canonical URL; no-URL/book rows dedupe via
/// `document_origins`. Provided ZIP snapshots are attached document-keyed; URL rows without a
/// snapshot are prepared by the content-gated AI pipeline.
pub(super) async fn process_csv_row(
    ctx: &IntegrationJobDeps,
    storage: &dyn ind_application::storage::ObjectStorage,
    user_id: UserId,
    row: &ReadwiseCsvRow,
    zip_map: &HashMap<String, ZipEntry>,
    matched_zip_ids: &mut std::collections::HashSet<String>,
) -> Result<ProcessCsvOutcome, ProcessRowResult> {
    let zip_entry = zip_map.get(&row.id);

    // Claim this row's ZIP entry up front so dedup early-returns below don't leave it to the
    // orphan-ZIP pass, which would create a phantom article-typed document from the HTML snapshot.
    if zip_entry.is_some() {
        matched_zip_ids.insert(row.id.clone());
    }

    let is_private_url = row
        .url
        .as_deref()
        .map(|u| u.starts_with("private://"))
        .unwrap_or(false);

    // Private rows are uploaded books/PDFs whose content lives only in the ZIP. Without a matching
    // ZIP asset there is nothing to import.
    if is_private_url && zip_entry.is_none() {
        return Ok(ProcessCsvOutcome::SkippedPrivate);
    }

    let url = if is_private_url {
        None
    } else {
        row.url.clone()
    };
    let item_type = detect_item_type(url.as_deref(), zip_entry);
    let document_type = document_type_for(item_type);
    let (triage_state, is_shortlisted) = location_to_triage(&row.location);
    let parsed_tags = parse_python_list_with_errors(&row.document_tags);
    // Readwise's saved_date is parsed for validation/consistency with the legacy path; the library
    // entry's saved_at is set by `save_to_library` at import time.
    let _ = parse_readwise_date(&row.saved_date);

    // YouTube HTML snapshots are useless (just the watch page), so those route through normal URL
    // preparation rather than attaching the snapshot.
    let is_youtube = url
        .as_deref()
        .is_some_and(ind_application::dispatch::is_youtube_url);
    let attach_entry = if is_youtube { None } else { zip_entry };
    // TASK-241: per-row provenance recorded on import_job_items. zip_path is the snapshot the
    // document content came from (none for YouTube, whose readable asset is the transcript);
    // tag_parse_errors preserves malformed-tag diagnostics instead of silently dropping them.
    let diagnostics = RowDiagnostics {
        zip_path: attach_entry.map(|entry| entry.path.clone()),
        zip_only: false,
        tag_parse_errors: parsed_tags.errors.clone(),
    }
    .to_value();
    let readwise_origin = readwise_origin_id(user_id, &row.id);
    let preexisting_origin_document_id = ctx
        .document_repo
        .find_by_origin(
            user_id,
            readwise_origin.origin_type,
            readwise_origin.origin_id,
        )
        .await
        .map_err(ProcessRowResult::Failed)?
        .map(|document| document.id);

    let (identity, enqueue_engaged_ai) = match url.as_deref() {
        Some(u) => {
            let canonical = canonicalize_url(u, &CanonicalizationConfig::default())
                .map(|c| c.into_string())
                .unwrap_or_else(|_| u.to_string());
            let document = NewUrlDocument {
                id: DocumentId::new(),
                user_id,
                document_type,
                canonical_url: canonical,
                original_url: Some(u.to_string()),
                content_hash: None,
                title: row.title.clone(),
                author: None,
                excerpt: None,
                published_at: None,
                language: None,
                domain: extract_domain(u),
                lead_image_url: None,
                thumbnail_url: None,
            };
            // A provided snapshot is attached as the readable asset (no render); a YouTube URL is
            // ingested by `document.youtube_ingest` (transcript-enriched), so it must not trigger
            // the generic readable-render prepare pipeline either. Otherwise the content-gated AI
            // pipeline prepares readable content from the URL.
            (
                MaterializeIdentity::Url {
                    document,
                    origin: Some(readwise_origin),
                },
                attach_entry.is_none() && !is_youtube,
            )
        }
        None => {
            let document = NewOriginDocument {
                id: DocumentId::new(),
                user_id,
                document_type,
                content_hash: None,
                original_url: None,
                title: row.title.clone(),
                author: None,
                excerpt: None,
                published_at: None,
                language: None,
                domain: None,
                lead_image_url: None,
                thumbnail_url: None,
                sender_id: None,
            };
            (
                MaterializeIdentity::Origin {
                    document,
                    origin: readwise_origin,
                },
                false,
            )
        }
    };

    let side_effects: Option<SaveSideEffectsFn> = if is_youtube {
        url.clone().map(|youtube_url| {
            let origin_document_for_enqueue = preexisting_origin_document_id;
            Box::new(move |ctx: &SaveContext<'_>| {
                let should_enqueue = !ctx.already_active
                    || origin_document_for_enqueue.is_some_and(|id| id == ctx.document.id);
                let outbox = should_enqueue
                    .then(|| {
                        youtube_ingest_document_outbox(
                            ctx.document.id,
                            user_id,
                            youtube_url.clone(),
                            Utc::now(),
                        )
                    })
                    .into_iter()
                    .collect();
                MaterializeSideEffects {
                    events: Vec::new(),
                    outbox,
                }
            }) as SaveSideEffectsFn
        })
    } else {
        None
    };

    let outcome = ctx
        .lifecycle
        .save_to_library(SaveToLibraryRequest {
            identity,
            source: ContentSource::Import,
            source_delivery_id: None,
            hide_deliveries: false,
            enqueue_engaged_ai,
            restore_policy: Default::default(),
            side_effects,
        })
        .await
        .map_err(ProcessRowResult::Failed)?;

    if outcome.already_active
        && preexisting_origin_document_id.is_some_and(|id| id != outcome.document.id)
    {
        return Err(ProcessRowResult::Failed(AppError::Domain(
            ind_domain::DomainError::InvariantViolation {
                message: format!(
                    "Readwise origin {} mapped to a different document than the active save",
                    readwise_origin.origin_id
                ),
            },
        )));
    }
    let same_origin_retry = outcome.already_active
        && preexisting_origin_document_id.is_some_and(|id| id == outcome.document.id);
    if outcome.already_active && !same_origin_retry {
        return Ok(ProcessCsvOutcome::Duplicate {
            search_reindex_jobs: 0,
            embedding_jobs: 0,
            diagnostics,
        });
    }

    let (search_reindex_jobs, embedding_jobs) = attach_provided_content(
        ctx,
        storage,
        user_id,
        &outcome,
        attach_entry,
        !same_origin_retry,
    )
    .await
    .map_err(ProcessRowResult::Failed)?;
    apply_tags(ctx, user_id, &outcome, &parsed_tags.tags)
        .await
        .map_err(ProcessRowResult::Failed)?;
    apply_triage(ctx, user_id, &outcome, triage_state, is_shortlisted)
        .await
        .map_err(ProcessRowResult::Failed)?;
    apply_reading_progress(ctx, user_id, &outcome, row)
        .await
        .map_err(ProcessRowResult::Failed)?;

    if same_origin_retry {
        Ok(ProcessCsvOutcome::Duplicate {
            search_reindex_jobs,
            embedding_jobs,
            diagnostics,
        })
    } else {
        Ok(ProcessCsvOutcome::Imported {
            search_reindex_jobs,
            embedding_jobs,
            diagnostics,
        })
    }
}

pub(super) async fn process_zip_only_entry(
    ctx: &IntegrationJobDeps,
    storage: &dyn ind_application::storage::ObjectStorage,
    user_id: UserId,
    ulid: &str,
    entry: &ZipEntry,
) -> Result<ProcessCsvOutcome, AppError> {
    let document_type = document_type_for(ext_to_item_type(&entry.extension));
    let document = NewOriginDocument {
        id: DocumentId::new(),
        user_id,
        document_type,
        content_hash: None,
        original_url: None,
        title: entry.title.clone(),
        author: None,
        excerpt: None,
        published_at: None,
        language: None,
        domain: None,
        lead_image_url: None,
        thumbnail_url: None,
        sender_id: None,
    };

    let outcome = ctx
        .lifecycle
        .save_to_library(SaveToLibraryRequest {
            identity: MaterializeIdentity::Origin {
                document,
                origin: readwise_origin_id(user_id, ulid),
            },
            source: ContentSource::Import,
            source_delivery_id: None,
            hide_deliveries: false,
            enqueue_engaged_ai: false,
            restore_policy: Default::default(),
            side_effects: None,
        })
        .await?;

    // TASK-241: a ZIP-only entry came from this ZIP path and has no CSV tags to parse.
    let diagnostics = RowDiagnostics {
        zip_path: Some(entry.path.clone()),
        zip_only: true,
        tag_parse_errors: Vec::new(),
    }
    .to_value();

    if outcome.already_active {
        return Ok(ProcessCsvOutcome::Duplicate {
            search_reindex_jobs: 0,
            embedding_jobs: 0,
            diagnostics,
        });
    }

    let (search_reindex_jobs, embedding_jobs) =
        attach_provided_content(ctx, storage, user_id, &outcome, Some(entry), true).await?;
    Ok(ProcessCsvOutcome::Imported {
        search_reindex_jobs,
        embedding_jobs,
        diagnostics,
    })
}

/// Attach a provided ZIP snapshot as document-keyed asset(s) and, when it is readable HTML, make
/// the document searchable + embedded. On attach failure the freshly-saved library entry is
/// rolled back so a failed import leaves nothing behind.
async fn attach_provided_content(
    ctx: &IntegrationJobDeps,
    storage: &dyn ind_application::storage::ObjectStorage,
    user_id: UserId,
    outcome: &SaveToLibraryOutcome,
    entry: Option<&ZipEntry>,
    soft_delete_on_failure: bool,
) -> Result<(u32, u32), AppError> {
    let Some(entry) = entry else {
        return Ok((0, 0));
    };
    let document_id = outcome.document.id;
    let readable =
        match attach_zip_asset_to_document(ctx, storage, user_id, document_id, entry).await {
            Ok(readable) => readable,
            Err(e) => {
                if soft_delete_on_failure {
                    ctx.library_repo
                        .soft_delete(outcome.entry.id, user_id, MutationSideEffects::none())
                        .await
                        .ok();
                }
                return Err(e);
            }
        };
    if readable {
        enqueue_search_reindex_document(ctx, document_id).await?;
        enqueue_document_embed_if_engaged(ctx, user_id, document_id).await?;
        return Ok((1, 1));
    }
    Ok((0, 0))
}

async fn apply_tags(
    ctx: &IntegrationJobDeps,
    user_id: UserId,
    outcome: &SaveToLibraryOutcome,
    tags: &[String],
) -> Result<(), AppError> {
    if tags.is_empty() {
        return Ok(());
    }
    let mut tag_ids = Vec::new();
    for name in tags {
        match ctx.tag_repo.find_or_create_by_name(user_id, name).await {
            Ok(tag) => tag_ids.push(tag.id),
            Err(e) => warn!(tag = %name, error = %e, "failed to find/create tag"),
        }
    }
    if !tag_ids.is_empty() {
        ctx.tag_repo
            .replace_for_library_entry_with_source(
                user_id,
                outcome.entry.id,
                &tag_ids,
                TagSource::Import,
                MutationSideEffects::none(),
            )
            .await?;
    }
    Ok(())
}

async fn apply_triage(
    ctx: &IntegrationJobDeps,
    user_id: UserId,
    outcome: &SaveToLibraryOutcome,
    triage_state: TriageState,
    is_shortlisted: bool,
) -> Result<(), AppError> {
    // Fresh entries default to inbox / not-shortlisted, so only the non-default cases need a write.
    if triage_state != TriageState::Inbox {
        ctx.library_repo
            .set_triage_state(
                outcome.entry.id,
                user_id,
                triage_state,
                MutationSideEffects::none(),
            )
            .await?;
    }
    if is_shortlisted {
        ctx.library_repo
            .toggle_shortlist(outcome.entry.id, user_id, MutationSideEffects::none())
            .await?;
    }
    Ok(())
}

async fn apply_reading_progress(
    ctx: &IntegrationJobDeps,
    user_id: UserId,
    outcome: &SaveToLibraryOutcome,
    row: &ReadwiseCsvRow,
) -> Result<(), AppError> {
    if row.reading_progress <= 0.0 && !row.seen {
        return Ok(());
    }
    let progress_percent = (row.reading_progress * 100.0).round() as i32;
    ctx.user_document_state_repo
        .record_progress(user_id, outcome.document.id, progress_percent, None, None)
        .await?;
    Ok(())
}
