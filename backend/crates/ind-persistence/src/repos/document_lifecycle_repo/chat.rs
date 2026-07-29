//! `start_single_document_chat` orchestration: the single public chat-start flow (TASK-234).
//!
//! Owns ONE transaction composing the materialize/back-link `*_tx` helpers (or an existing-document
//! load) plus the `mila_sessions` insert, the content-gated AI outbox, and a `document.engaged`
//! event when the document is not already saved. `materialize_document` alone cannot commit a
//! session row, so chat-start is its own primitive. See docs/document-feed-library-architecture.md
//! (single-document chat attaches to document_id regardless of saved state).

use sqlx::PgPool;

use ind_application::AppError;
use ind_application::event_intents;
use ind_application::repos::document_lifecycle::{
    ChatIdentity, MaterializeIdentity, NewChatSession, StartDocumentChatOutcome,
    StartDocumentChatRequest,
};
use ind_application::repos::lifecycle_outbox::OutboxEntry;
use ind_domain::{DocumentOriginType, DomainError, MilaSession, MilaSessionType, NewDomainEvent};

use super::super::document_repo::tx_writes::select_document_by_id;
use super::super::mila_session_repo::insert_mila_session_tx;
use super::super::write_helpers::{apply_domain_events_tx, apply_outbox_tx};
use super::steps::{
    BacklinkKey, backlink_feed_deliveries_tx, build_engaged_document_ai_outbox_tx,
    document_is_saved_tx, materialize_document_tx,
};

pub(super) async fn start_single_document_chat(
    pool: &PgPool,
    request: StartDocumentChatRequest,
) -> Result<StartDocumentChatOutcome, AppError> {
    let StartDocumentChatRequest {
        chat_identity,
        session,
        enqueue_engaged_ai,
        side_effects,
    } = request;
    let NewChatSession {
        session_id,
        user_id,
        created_at,
    } = session;

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Repository(Box::new(e)))?;

    let (document, document_created, backlinked_deliveries) = match chat_identity {
        ChatIdentity::Existing { document_id } => {
            let row = select_document_by_id(&mut tx, user_id, document_id)
                .await?
                .ok_or_else(|| {
                    AppError::Domain(DomainError::NotFound {
                        entity: "Document",
                        id: document_id.to_string(),
                    })
                })?;
            (row.into_document()?, false, 0)
        }
        ChatIdentity::Materialize(identity) => {
            let identity = identity.as_ref();
            let (document, created) = materialize_document_tx(&mut tx, identity).await?;
            let backlinked = match identity {
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
            (document, created, backlinked)
        }
    };

    let inserted = MilaSession {
        id: session_id,
        user_id,
        document_id: Some(document.id),
        collection_id: None,
        session_type: MilaSessionType::SingleDocument,
        created_at,
        last_active: created_at,
    };
    let session = insert_mila_session_tx(&mut tx, &inserted).await?;

    let mut outbox: Vec<OutboxEntry> = Vec::new();
    if enqueue_engaged_ai {
        outbox.extend(build_engaged_document_ai_outbox_tx(&mut tx, &document).await?);
    }

    // `document.engaged(chatted)` is emitted only when the document is not already saved; a saved
    // document's engagement is already represented by its library entry (AC#5).
    let mut events: Vec<NewDomainEvent> = Vec::new();
    if !document_is_saved_tx(&mut tx, user_id, document.id).await? {
        events.push(event_intents::document_engaged(
            user_id,
            document.id,
            "chatted",
        ));
    }

    if let Some(build) = side_effects {
        let extra = build(&document);
        events.extend(extra.events);
        outbox.extend(extra.outbox);
    }

    apply_domain_events_tx(&mut tx, events).await?;
    apply_outbox_tx(&mut tx, &outbox).await?;

    tx.commit()
        .await
        .map_err(|e| AppError::Repository(Box::new(e)))?;

    Ok(StartDocumentChatOutcome {
        document,
        session,
        document_created,
        backlinked_deliveries,
    })
}
