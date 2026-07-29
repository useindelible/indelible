use std::sync::Arc;

use futures::future::BoxFuture;

use crate::error::AppError;
use crate::ports::OutboundUrlGuard;
use crate::ports::content::ExtensionSaveOperations;
use crate::repos::document_asset::DocumentAssetRepository;
use crate::repos::document_lifecycle::DocumentLifecycle;
use crate::repos::user_preferences::UserPreferencesRepository;
use crate::storage::ObjectStorage;
use ind_domain::{DocumentId, ItemType, LibraryEntryId, UserId};

mod archive;
mod assets;
mod builders;
mod quick;
mod reader;
mod utils;

/// Extension save service. Every save shape routes through the atomic
/// `DocumentLifecycle::save_to_library` primitive: URL identity dedupes on
/// `(user_id, canonical_url)` and inserts/restores a `library_entries` row. Browser-provided
/// readable/monolith HTML is attached via the staged `document.attach_provided_content` flow (see
/// `handlers::provided_content`). Server-rendered pdf/screenshot for extension saves are delegated
/// to the document preparation pipeline when the user's archival preferences enable them.
pub struct ExtensionSaveService {
    lifecycle: Arc<dyn DocumentLifecycle>,
    document_asset_repo: Arc<dyn DocumentAssetRepository>,
    object_storage: Arc<dyn ObjectStorage>,
    user_preferences_repo: Arc<dyn UserPreferencesRepository>,
    url_guard: Arc<dyn OutboundUrlGuard>,
}

// Document identity dedupes on (user_id, canonical_url), so duplicate override would need a
// first-class "save as new variant" product contract. Reading metadata is computed from the
// readable asset at read time.
pub struct QuickSaveInput {
    pub url: String,
    pub title: Option<String>,
}

pub struct ReaderSaveInput {
    pub url: String,
    pub canonical_url: Option<String>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub excerpt: Option<String>,
    pub reader_html: String,
    pub language: Option<String>,
    pub lead_image_url: Option<String>,
    pub item_type: Option<ItemType>,
}

pub struct FullArchiveInput {
    pub url: String,
    pub canonical_url: Option<String>,
    pub title: Option<String>,
    pub reader_html: Option<String>,
    pub html_base64: String,
    pub lead_image_url: Option<String>,
    pub excerpt: Option<String>,
    pub author: Option<String>,
    pub language: Option<String>,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
    pub item_type: Option<ItemType>,
}

/// Result of an extension save.
pub struct SaveResult {
    pub library_entry_id: LibraryEntryId,
    pub document_id: DocumentId,
    pub status: &'static str,
}

impl ExtensionSaveService {
    pub fn new(
        lifecycle: Arc<dyn DocumentLifecycle>,
        document_asset_repo: Arc<dyn DocumentAssetRepository>,
        object_storage: Arc<dyn ObjectStorage>,
        user_preferences_repo: Arc<dyn UserPreferencesRepository>,
        url_guard: Arc<dyn OutboundUrlGuard>,
    ) -> Self {
        Self {
            lifecycle,
            document_asset_repo,
            object_storage,
            user_preferences_repo,
            url_guard,
        }
    }
}

impl ExtensionSaveOperations for ExtensionSaveService {
    fn quick_save(
        &self,
        user_id: UserId,
        input: QuickSaveInput,
    ) -> BoxFuture<'_, Result<SaveResult, AppError>> {
        Box::pin(self.quick_save(user_id, input))
    }

    fn reader_save(
        &self,
        user_id: UserId,
        input: ReaderSaveInput,
    ) -> BoxFuture<'_, Result<SaveResult, AppError>> {
        Box::pin(self.reader_save(user_id, input))
    }

    fn full_archive(
        &self,
        user_id: UserId,
        input: FullArchiveInput,
    ) -> BoxFuture<'_, Result<SaveResult, AppError>> {
        Box::pin(self.full_archive(user_id, input))
    }
}
