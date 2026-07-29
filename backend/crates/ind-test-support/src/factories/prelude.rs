pub(super) use chrono::Utc;
pub(super) use fake::{
    Fake,
    faker::{lorem::en::Sentence, name::en::Name},
};
pub(super) use ind_application::repos::{
    collection::CollectionRepository, document::DocumentRepository, feed::FeedRepository,
    feed_delivery::FeedDeliveryRepository, library::LibraryRepository, user::UserRepository,
};
pub(super) use ind_domain::{
    Collection, CollectionId, ContentSource, Document, DocumentId, DocumentType, FeedDelivery,
    FeedDeliveryId, FeedSource, FeedSourceEntryId, FeedSourceId, FeedStatus, FeedSubscription,
    FeedSubscriptionId, FeedType, FeedVisibility, LibraryEntry, LibraryEntryId, NewUrlDocument,
    Theme, TriageState, User, UserId, UserStatus,
};
pub(super) use ind_persistence::repos::{
    PgCollectionRepository, PgDocumentRepository, PgFeedDeliveryRepository, PgFeedRepository,
    PgLibraryRepository, PgUserRepository,
};

pub(super) fn short_unique_suffix() -> String {
    // The last 8 hex chars of a v7 UUID are random bits; the first 8 are the
    // high timestamp bits, identical for ~65s, which made names collide.
    uuid::Uuid::now_v7().simple().to_string()[24..32].to_string()
}
