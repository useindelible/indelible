mod chat;
mod save;
pub(crate) mod steps;

use sqlx::PgPool;

use ind_application::AppError;
use ind_application::repos::document_lifecycle::{
    DocumentLifecycle, MaterializeIdentity, MaterializeOutcome, MaterializeRequest,
    MaterializeSideEffects, SaveToLibraryOutcome, SaveToLibraryRequest, StartDocumentChatOutcome,
    StartDocumentChatRequest,
};
use ind_domain::DocumentOriginType;

use super::write_helpers::{apply_domain_events_tx, apply_outbox_tx};
use steps::{
    BacklinkKey, backlink_feed_deliveries_tx, materialize_document_tx,
    upsert_user_document_state_tx,
};

/// Atomic document materialization and adoption. The public `materialize_document` owns a
/// single transaction; see the `DocumentLifecycle` trait docs and
/// docs/document-feed-library-architecture.md for the transaction-boundary contract.
pub struct PgDocumentLifecycle {
    pool: PgPool,
}

impl PgDocumentLifecycle {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl DocumentLifecycle for PgDocumentLifecycle {
    async fn materialize_document(
        &self,
        request: MaterializeRequest,
    ) -> Result<MaterializeOutcome, AppError> {
        let MaterializeRequest {
            identity,
            document_state,
            side_effects,
        } = request;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Repository(Box::new(e)))?;

        let (document, created) = materialize_document_tx(&mut tx, &identity).await?;

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

        let state = match document_state {
            Some(input) => Some(
                upsert_user_document_state_tx(&mut tx, document.user_id, document.id, &input)
                    .await?,
            ),
            None => None,
        };

        // Build side effects from the RESOLVED document so events/outbox reference the real
        // document id even when an existing document was found on conflict.
        let MaterializeSideEffects { events, outbox } = side_effects
            .map(|build| build(&document))
            .unwrap_or_default();
        apply_domain_events_tx(&mut tx, events).await?;
        apply_outbox_tx(&mut tx, &outbox).await?;

        tx.commit()
            .await
            .map_err(|e| AppError::Repository(Box::new(e)))?;

        Ok(MaterializeOutcome {
            document,
            created,
            backlinked_deliveries,
            state,
        })
    }

    async fn save_to_library(
        &self,
        request: SaveToLibraryRequest,
    ) -> Result<SaveToLibraryOutcome, AppError> {
        save::save_to_library(&self.pool, request).await
    }

    async fn start_single_document_chat(
        &self,
        request: StartDocumentChatRequest,
    ) -> Result<StartDocumentChatOutcome, AppError> {
        chat::start_single_document_chat(&self.pool, request).await
    }
}
