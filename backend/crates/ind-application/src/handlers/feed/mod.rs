use std::sync::Arc;

use crate::ports::{FeedParser, HttpFetcher, OpmlParser};
use crate::repos::feed::FeedRepository;
use crate::repos::outbox::JobOutboxRepository;
use ind_domain::{CollectionId, FeedStatus, FeedSubscription, FeedType, FeedVisibility};

mod discovery;
mod operations;
mod opml;
mod polling;
mod providers;
mod subscriptions;

#[cfg(test)]
mod tests;

pub use polling::{
    effective_poll_interval_minutes, next_poll_after_failure, next_poll_after_success,
};

const MIN_PUBLIC_POLL_INTERVAL_MINUTES: i64 = 15;
const DEFAULT_PUBLIC_POLL_INTERVAL_MINUTES: i64 = 15;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeedPollScheduleConfig {
    pub default_public_poll_interval_minutes: i64,
    pub min_public_poll_interval_minutes: i64,
}

impl FeedPollScheduleConfig {
    pub fn normalized(self) -> Self {
        let min_public_poll_interval_minutes = self.min_public_poll_interval_minutes.max(1);
        let default_public_poll_interval_minutes = self
            .default_public_poll_interval_minutes
            .max(min_public_poll_interval_minutes);

        Self {
            default_public_poll_interval_minutes,
            min_public_poll_interval_minutes,
        }
    }
}

impl Default for FeedPollScheduleConfig {
    fn default() -> Self {
        Self {
            default_public_poll_interval_minutes: DEFAULT_PUBLIC_POLL_INTERVAL_MINUTES,
            min_public_poll_interval_minutes: MIN_PUBLIC_POLL_INTERVAL_MINUTES,
        }
    }
}

pub struct FeedService {
    feed_repo: Arc<dyn FeedRepository>,
    outbox_repo: Arc<dyn JobOutboxRepository>,
    http_fetcher: Arc<dyn HttpFetcher>,
    feed_parser: Arc<dyn FeedParser>,
    opml_parser: Arc<dyn OpmlParser>,
}

pub struct SubscribeInput {
    pub url: String,
    pub title_override: Option<String>,
    pub poll_interval_override_minutes: Option<i32>,
}

#[derive(Debug)]
pub struct SubscribeResult {
    pub subscription: FeedSubscription,
    pub is_new: bool,
}

pub struct UpdateSubscriptionInput {
    pub title_override: Option<Option<String>>,
    pub auto_save: Option<bool>,
    pub auto_save_collection_id: Option<Option<CollectionId>>,
    pub poll_interval_override_minutes: Option<Option<i32>>,
    pub status: Option<FeedStatus>,
}

pub struct OpmlImportResult {
    pub created: u32,
    pub skipped: u32,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ResolvedFeedSource {
    pub canonical_key: String,
    pub source_url: String,
    pub poll_url: String,
    pub title: String,
    pub description: Option<String>,
    pub site_url: Option<String>,
    pub image_url: Option<String>,
    pub domain: Option<String>,
    pub feed_type: FeedType,
    pub visibility: FeedVisibility,
    pub provider: Option<String>,
    pub is_resolvable: bool,
}

impl FeedService {
    pub fn new(
        feed_repo: Arc<dyn FeedRepository>,
        outbox_repo: Arc<dyn JobOutboxRepository>,
        http_fetcher: Arc<dyn HttpFetcher>,
        feed_parser: Arc<dyn FeedParser>,
        opml_parser: Arc<dyn OpmlParser>,
    ) -> Self {
        Self {
            feed_repo,
            outbox_repo,
            http_fetcher,
            feed_parser,
            opml_parser,
        }
    }
}
