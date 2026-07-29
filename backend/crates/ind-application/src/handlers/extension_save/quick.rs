use chrono::Utc;

use crate::error::AppError;
use crate::repos::document_lifecycle::{
    MaterializeIdentity, MaterializeSideEffects, SaveToLibraryRequest,
};
use crate::repos::lifecycle_outbox::search_reindex_document_outbox;
use ind_domain::{ContentSource, UserId};

use super::utils::resolved_canonical_url;
use super::{ExtensionSaveService, QuickSaveInput, SaveResult};

impl ExtensionSaveService {
    /// URL-only save: no browser-provided content. The save enables content-gated AI so the
    /// document preparation pipeline renders readable content from the URL; an early metadata
    /// reindex makes the saved document searchable before preparation completes.
    pub async fn quick_save(
        &self,
        user_id: UserId,
        input: QuickSaveInput,
    ) -> Result<SaveResult, AppError> {
        // quick_save renders the page server-side; reject private/internal
        // targets before queuing the render (SSRF). Defense in depth with the
        // renderer pre-flight.
        self.url_guard.check_url(&input.url).await.map_err(|e| {
            AppError::Domain(ind_domain::DomainError::Validation {
                field: "url".into(),
                message: e.message().to_string(),
            })
        })?;

        let canonical_url = resolved_canonical_url(&input.url, None);
        let document = Self::build_url_document(
            user_id,
            &input.url,
            canonical_url,
            input.title,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

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
                enqueue_engaged_ai: true,
                restore_policy: Default::default(),
                side_effects: Some(Box::new(|ctx| MaterializeSideEffects {
                    events: Vec::new(),
                    outbox: vec![search_reindex_document_outbox(ctx.document.id, Utc::now())],
                })),
            })
            .await?;

        Ok(Self::save_result(&outcome))
    }
}
