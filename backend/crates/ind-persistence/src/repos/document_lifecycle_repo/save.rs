//! `save_to_library` orchestration: the single public save flow.
//!
//! Owns ONE transaction composing the TASK-228 `*_tx` helpers plus the library-entry
//! insert/restore and optional delivery hiding. The intrinsic `library_entry.saved` domain event
//! is built here from the RESOLVED
//! `(document, entry)` so it always keys off the real ids. Readable-content preparation is NOT
//! enqueued here: the outbox relay dispatches any pending row immediately and the worker skips
//! unknown job types as success, so a `document_prepare` row would be silently consumed before
//! the Phase 6 (TASK-231) consumer exists. Phase 6 owns both the enqueue (per save policy) and
//! the worker. See docs/document-feed-library-architecture.md (User saves a feed-delivered
//! document; Materialization and adoption must be atomic).

use sqlx::PgPool;

use ind_application::AppError;
use ind_application::event_intents;
use ind_application::repos::document_lifecycle::{
    MaterializeIdentity, SaveContext, SaveToLibraryOutcome, SaveToLibraryRequest,
};
use ind_application::repos::lifecycle_outbox::OutboxEntry;
use ind_domain::DocumentOriginType;

use super::super::feed_delivery_repo::tx_writes::hide_deliveries_for_document_tx;
use super::super::library_repo::tx_writes::insert_or_restore_library_entry_tx;
use super::super::write_helpers::{apply_domain_events_tx, apply_outbox_tx};
use super::steps::{
    BacklinkKey, backlink_feed_deliveries_tx, build_engaged_document_ai_outbox_tx,
    materialize_document_tx,
};

pub(super) async fn save_to_library(
    pool: &PgPool,
    request: SaveToLibraryRequest,
) -> Result<SaveToLibraryOutcome, AppError> {
    let SaveToLibraryRequest {
        identity,
        source,
        source_delivery_id,
        hide_deliveries,
        enqueue_engaged_ai,
        restore_policy,
        side_effects,
    } = request;

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Repository(Box::new(e)))?;

    let (document, document_created) = materialize_document_tx(&mut tx, &identity).await?;

    let backlinked_deliveries = match &identity {
        MaterializeIdentity::Url { document: doc, .. } => {
            backlink_feed_deliveries_tx(
                &mut tx,
                document.user_id,
                document.id,
                BacklinkKey::CanonicalUrl(&doc.canonical_url),
            )
            .await?
        }
        MaterializeIdentity::Origin { origin, .. } => match origin.origin_type {
            DocumentOriginType::FeedSourceEntry => {
                backlink_feed_deliveries_tx(
                    &mut tx,
                    document.user_id,
                    document.id,
                    BacklinkKey::SourceEntry(origin.origin_id),
                )
                .await?
            }
            _ => 0,
        },
    };

    let upsert = insert_or_restore_library_entry_tx(
        &mut tx,
        document.user_id,
        document.id,
        source,
        source_delivery_id,
        restore_policy,
    )
    .await?;

    let hidden_deliveries = if hide_deliveries && !upsert.skipped_restore {
        hide_deliveries_for_document_tx(&mut tx, document.user_id, document.id).await?
    } else {
        0
    };

    // Intrinsic save event, built from the resolved (document, entry). Any outbox rows come
    // only from a caller-provided side-effect builder (Phase 6 attaches preparation jobs once
    // a consumer exists).
    let mut events = if upsert.skipped_restore {
        Vec::new()
    } else {
        vec![event_intents::library_entry_saved(
            document.user_id,
            upsert.entry.id,
            document.id,
            source,
        )]
    };
    let mut outbox: Vec<OutboxEntry> = Vec::new();

    // Content-gated AI enablement (TASK-234, Codex P1). Saving an already-prepared document embeds
    // immediately; saving an unprepared one enqueues preparation, which embeds on completion
    // because the document is now saved. This closes the read-ahead-then-save gap.
    if enqueue_engaged_ai && !upsert.skipped_restore {
        outbox.extend(build_engaged_document_ai_outbox_tx(&mut tx, &document).await?);
    }

    if let Some(build) = side_effects
        && !upsert.skipped_restore
    {
        let ctx = SaveContext {
            document: &document,
            entry: &upsert.entry,
            document_created,
            restored: upsert.restored,
            already_active: upsert.already_active,
        };
        let extra = build(&ctx);
        events.extend(extra.events);
        outbox.extend(extra.outbox);
    }

    apply_domain_events_tx(&mut tx, events).await?;
    apply_outbox_tx(&mut tx, &outbox).await?;

    tx.commit()
        .await
        .map_err(|e| AppError::Repository(Box::new(e)))?;

    Ok(SaveToLibraryOutcome {
        document,
        document_created,
        entry: upsert.entry,
        restored: upsert.restored,
        skipped_restore: upsert.skipped_restore,
        already_active: upsert.already_active,
        backlinked_deliveries,
        hidden_deliveries,
    })
}
