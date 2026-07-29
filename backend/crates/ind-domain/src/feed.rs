use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

use crate::{CollectionId, FeedSourceEntryId, FeedSourceId, FeedSubscriptionId, UserId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedType {
    Rss,
    Atom,
    Podcast,
    Youtube,
    Twitter,
    Newsletter,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedStatus {
    #[default]
    Active,
    Paused,
    Error,
}

impl FeedStatus {
    pub const NAMES: &'static [&'static str] = &["active", "paused", "error"];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Paused => "paused",
            Self::Error => "error",
        }
    }
}

impl fmt::Display for FeedStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for FeedStatus {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "active" => Ok(Self::Active),
            "paused" => Ok(Self::Paused),
            "error" => Ok(Self::Error),
            other => Err(format!("invalid feed status: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedVisibility {
    #[default]
    Public,
    Private,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedSearchSurface {
    #[default]
    All,
    Rss,
    Youtube,
    Twitter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedSource {
    pub id: FeedSourceId,
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
    pub popularity: i32,
    pub last_entry_added_at: Option<DateTime<Utc>>,
    pub last_polled_at: Option<DateTime<Utc>>,
    pub next_poll_at: Option<DateTime<Utc>>,
    pub last_etag: Option<String>,
    pub last_modified: Option<String>,
    pub consecutive_failures: i32,
    pub last_error: Option<String>,
    pub lease_owner: Option<String>,
    pub lease_expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedSubscription {
    pub id: FeedSubscriptionId,
    pub user_id: UserId,
    pub source_id: FeedSourceId,
    pub input_url: String,
    pub title_override: Option<String>,
    pub auto_save: bool,
    pub auto_save_collection_id: Option<CollectionId>,
    pub poll_interval_override_minutes: Option<i32>,
    pub status: FeedStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub source: FeedSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveSubscription {
    pub id: FeedSubscriptionId,
    pub user_id: UserId,
    pub source_id: FeedSourceId,
    pub auto_save: bool,
    pub auto_save_collection_id: Option<CollectionId>,
    pub poll_interval_override_minutes: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedProviderInstance {
    pub id: uuid::Uuid,
    pub provider_type: String,
    pub base_url: String,
    pub priority: i32,
    pub enabled: bool,
    pub last_success_at: Option<DateTime<Utc>>,
    pub last_failure_at: Option<DateTime<Utc>>,
    pub consecutive_failures: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceDetailsUpdate {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollOutcome {
    pub source_id: FeedSourceId,
    pub last_polled_at: Option<DateTime<Utc>>,
    pub next_poll_at: Option<DateTime<Utc>>,
    pub last_etag: Option<String>,
    pub last_modified: Option<String>,
    pub consecutive_failures: i32,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedSourceEntry {
    pub id: FeedSourceEntryId,
    pub source_id: FeedSourceId,
    pub guid: String,
    pub title: String,
    pub url: Option<String>,
    /// Canonicalized form of `url`, populated at poll time with the same
    /// `canonicalize_url` + `CanonicalizationConfig::default()` used by the save path.
    /// Drives the document adoption/back-link query; NULL when the URL is absent or
    /// cannot be canonicalized. See docs/document-feed-library-architecture.md.
    pub canonical_url: Option<String>,
    pub author: Option<String>,
    pub excerpt: Option<String>,
    pub content_html: Option<String>,
    pub language: Option<String>,
    /// Lead/hero image for the entry, extracted at poll time from feed media metadata
    /// (media:thumbnail / media:content) or the first substantial image in `content_html`.
    pub lead_image_url: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub discovered_at: DateTime<Utc>,
}
