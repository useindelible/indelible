use chrono::{DateTime, Utc};

use crate::handlers::feed_identity::document_type_for;
use crate::repos::document_lifecycle::SaveToLibraryOutcome;
use ind_domain::{DocumentId, ItemType, NewUrlDocument, UserId};

use super::ExtensionSaveService;
use super::utils::extract_domain;

impl ExtensionSaveService {
    /// Build the URL-identity document for an extension save. The document type follows the same
    /// URL inference the legacy item save used; an explicit caller `item_type` only overrides when
    /// the URL itself does not imply a concrete type (e.g. plain articles). `force_new` is no
    /// longer honored: document identity is `(user_id, canonical_url)`, so the same URL always
    /// resolves the same document — there is no per-save duplicate to create.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn build_url_document(
        user_id: UserId,
        url: &str,
        canonical_url: String,
        title: Option<String>,
        author: Option<String>,
        excerpt: Option<String>,
        language: Option<String>,
        lead_image_url: Option<String>,
        published_at: Option<DateTime<Utc>>,
        content_hash: Option<String>,
        item_type: Option<ItemType>,
    ) -> NewUrlDocument {
        let inferred = crate::dispatch::infer_item_type_for_url(url);
        let document_type = document_type_for(if inferred == ItemType::Article {
            item_type.unwrap_or(ItemType::Article)
        } else {
            inferred
        });

        NewUrlDocument {
            id: DocumentId::new(),
            user_id,
            document_type,
            canonical_url,
            original_url: Some(url.to_string()),
            content_hash,
            title: title.unwrap_or_default(),
            author,
            excerpt,
            published_at,
            language,
            domain: extract_domain(url),
            thumbnail_url: lead_image_url.clone(),
            lead_image_url,
        }
    }

    /// Map a save outcome to the API result. An already-active entry is reported as `exists`
    /// (idempotent save); any new/restored save is `queued`.
    pub(super) fn save_result(outcome: &SaveToLibraryOutcome) -> super::SaveResult {
        super::SaveResult {
            library_entry_id: outcome.entry.id,
            document_id: outcome.document.id,
            status: if outcome.already_active {
                "exists"
            } else {
                "queued"
            },
        }
    }
}
