#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::unimplemented,
        clippy::todo
    )
)]

pub mod asset_serving;
pub mod content_hash;
pub mod dispatch;
pub mod error;
pub mod event_intents;
pub mod export_summary;
pub mod handlers;
pub mod outputs;
pub mod ports;
pub mod recovery_keys;
pub mod renderer;
pub mod repos;
pub mod search_language;
pub mod services;
pub mod storage;
pub mod text;
pub mod webhooks;

pub use asset_serving::AssetServingMode;
pub use error::AppError;
pub use handlers::collection::{
    CollectionService, CollectionWithCount, CreateCollectionInput, UpdateCollectionInput,
};
pub use handlers::document_reader::DocumentReaderService;
pub use handlers::entity::{EntityOperationsService, EntityService, UpdateEntityInput};
pub use handlers::extension_save::ExtensionSaveService;
pub use handlers::feed::{
    FeedService, OpmlImportResult, ResolvedFeedSource, SubscribeInput, SubscribeResult,
    UpdateSubscriptionInput,
};
pub use handlers::feed_delivery::FeedDeliveryService;
pub use handlers::feed_preparation::{FeedPreparationConfig, FeedPreparationService};
pub use handlers::highlight::{HighlightService, HighlightWithNote};
pub use handlers::home::{HomeDashboardData, HomeService};
pub use handlers::library::LibraryService;
pub use handlers::library_upload::LibraryUploadService;
pub use handlers::mila::{ChatTarget, MilaSessionService};
pub use handlers::mila_config::MilaConfigService;
pub use handlers::settings::{PreferencesSection, SettingsService};
pub use handlers::smart_list::{CreateSmartListInput, SmartListService, UpdateSmartListInput};
pub use handlers::tag::{CreateTagInput, TagService, TagWithMeta, UpdateTagInput};
pub use ind_domain::{FilterNode, FilterOp};
pub use repos::mila_config::{ApiKeyUpdate, UpsertMilaConfigInput};
pub use repos::{Cursor, Page};
pub use search_language::{
    SearchLanguageDecision, SearchTextConfig, classify_search_language, normalize_language_tag,
};
