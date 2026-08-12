use futures::future::BoxFuture;
use ind_domain::{
    ArchivalSettings, ArchiveAssetKind, CollectionId, DocumentId, DocumentNote, EntityDetail,
    EntityId, EntitySummary, EntityType, FeedSearchSurface, FeedSource, FeedSubscription,
    FeedSubscriptionId, FilterNode, Highlight, HighlightId, HighlightLocator, HighlightNote,
    HighlightSourceLocator, HomeWidgetKind, LibraryEntryId, LibraryEntryWithDocument,
    NotificationPreferences, PreferencesSettings, RecentSearch, RecentSearchId, SearchPage,
    SearchRateLimitStatus, SearchSuggestion, SmartList, SmartListId, Tag, TagId, Theme,
    TriageState, UserId,
};

use crate::handlers::extension_save::{
    FullArchiveInput, QuickSaveInput, ReaderSaveInput, SaveResult,
};
use crate::repos::Page;
use crate::{
    AppError, CollectionWithCount, HighlightWithNote, HomeDashboardData, PreferencesSection,
    TagWithMeta, TaggedHighlight, UpdateSubscriptionInput,
};

mod article_toc;
mod collections;
mod document_reader;
mod entities;
mod extension;
mod feed_delivery;
mod feed_preparation;
mod feeds;
mod highlights;
mod library;
mod search;
mod settings;
mod smart_lists;
mod tags;

pub use article_toc::*;
pub use collections::*;
pub use document_reader::*;
pub use entities::*;
pub use extension::*;
pub use feed_delivery::*;
pub use feed_preparation::*;
pub use feeds::*;
pub use highlights::*;
pub use library::*;
pub use search::*;
pub use settings::*;
pub use smart_lists::*;
pub use tags::*;
