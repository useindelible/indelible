use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::FieldError;
use crate::extract::Validate;

#[derive(Debug, Deserialize, ToSchema)]
pub struct SubscribeBody {
    pub url: String,
    pub title: Option<String>,
    pub poll_interval_override_minutes: Option<i32>,
}

impl Validate for SubscribeBody {
    fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut errors = Vec::new();
        if self.url.trim().is_empty() {
            errors.push(FieldError {
                field: "url".into(),
                message: "must not be empty".into(),
            });
        }
        if let Some(minutes) = self.poll_interval_override_minutes
            && minutes <= 0
        {
            errors.push(FieldError {
                field: "poll_interval_override_minutes".into(),
                message: "must be greater than 0".into(),
            });
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateSubscriptionBody {
    pub title: Option<String>,
    pub auto_save: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable_string")]
    #[schema(value_type = Option<String>, nullable)]
    pub auto_save_collection_id: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable_i32")]
    #[schema(value_type = Option<i32>, nullable)]
    pub poll_interval_override_minutes: Option<Option<i32>>,
    pub status: Option<String>,
}

impl Validate for UpdateSubscriptionBody {
    fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut errors = Vec::new();
        if let Some(ref status) = self.status
            && status.parse::<ind_domain::FeedStatus>().is_err()
        {
            errors.push(FieldError {
                field: "status".into(),
                message: format!(
                    "must be one of: {}",
                    ind_domain::FeedStatus::NAMES.join(", ")
                ),
            });
        }
        if let Some(Some(minutes)) = self.poll_interval_override_minutes
            && minutes <= 0
        {
            errors.push(FieldError {
                field: "poll_interval_override_minutes".into(),
                message: "must be greater than 0".into(),
            });
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct ListSubscriptionsParams {
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct SearchFeedSourcesParams {
    pub query: String,
    pub surface: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FeedSourceResponse {
    pub id: String,
    pub object: &'static str,
    pub url: String,
    pub poll_url: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    pub source_kind: String,
    pub visibility: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    pub is_resolvable: bool,
    pub popularity: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub last_entry_added_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub last_polled_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub next_poll_at: Option<DateTime<Utc>>,
    pub consecutive_failures: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
}

impl FeedSourceResponse {
    pub fn from_domain(source: ind_domain::FeedSource) -> Self {
        Self {
            id: source.id.to_string(),
            object: "feed_source",
            url: source.source_url,
            poll_url: source.poll_url,
            name: source.title,
            description: source.description,
            site_url: source.site_url,
            image_url: source.image_url,
            domain: source.domain,
            source_kind: format_feed_type(source.feed_type),
            visibility: format_feed_visibility(source.visibility),
            provider: source.provider,
            is_resolvable: source.is_resolvable,
            popularity: source.popularity,
            last_entry_added_at: source.last_entry_added_at,
            last_polled_at: source.last_polled_at,
            next_poll_at: source.next_poll_at,
            consecutive_failures: source.consecutive_failures,
            last_error: source.last_error,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FeedSubscriptionResponse {
    pub id: String,
    pub object: &'static str,
    pub input_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_override: Option<String>,
    pub auto_save: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_save_collection_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poll_interval_override_minutes: Option<i32>,
    pub status: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
    pub source: FeedSourceResponse,
}

impl FeedSubscriptionResponse {
    pub fn from_domain(s: ind_domain::FeedSubscription) -> Self {
        Self {
            id: s.id.to_string(),
            object: "feed_subscription",
            input_url: s.input_url,
            title_override: s.title_override,
            auto_save: s.auto_save,
            auto_save_collection_id: s.auto_save_collection_id.map(|c| c.to_string()),
            poll_interval_override_minutes: s.poll_interval_override_minutes,
            status: format_feed_status(s.status),
            created_at: s.created_at,
            updated_at: s.updated_at,
            source: FeedSourceResponse::from_domain(s.source),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SubscribeResponse {
    pub subscription: FeedSubscriptionResponse,
    pub is_new: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FeedSearchResponse {
    pub items: Vec<FeedSourceResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OpmlImportResponse {
    pub created: u32,
    pub skipped: u32,
    pub errors: Vec<String>,
}

const VALID_FEED_SEARCH_SURFACES: &[&str] = &["all", "rss", "youtube", "twitter"];

pub(crate) fn parse_feed_status(s: &str) -> Option<ind_domain::FeedStatus> {
    s.parse().ok()
}

pub(crate) fn parse_feed_search_surface(s: &str) -> Option<ind_domain::FeedSearchSurface> {
    match s {
        "all" => Some(ind_domain::FeedSearchSurface::All),
        "rss" => Some(ind_domain::FeedSearchSurface::Rss),
        "youtube" => Some(ind_domain::FeedSearchSurface::Youtube),
        "twitter" => Some(ind_domain::FeedSearchSurface::Twitter),
        _ => None,
    }
}

pub(crate) fn valid_feed_search_surfaces() -> &'static [&'static str] {
    VALID_FEED_SEARCH_SURFACES
}

pub(crate) fn format_feed_status(s: ind_domain::FeedStatus) -> String {
    s.as_str().to_string()
}

pub(crate) fn format_feed_visibility(v: ind_domain::FeedVisibility) -> String {
    match v {
        ind_domain::FeedVisibility::Public => "public",
        ind_domain::FeedVisibility::Private => "private",
    }
    .to_string()
}

pub(crate) fn format_feed_type(ft: ind_domain::FeedType) -> String {
    match ft {
        ind_domain::FeedType::Rss => "rss",
        ind_domain::FeedType::Atom => "atom",
        ind_domain::FeedType::Podcast => "podcast",
        ind_domain::FeedType::Youtube => "youtube",
        ind_domain::FeedType::Twitter => "twitter",
        ind_domain::FeedType::Newsletter => "newsletter",
    }
    .to_string()
}

pub(crate) fn parse_feed_subscription_id(
    s: &str,
) -> Result<ind_domain::FeedSubscriptionId, crate::error::ApiError> {
    s.parse().map_err(|_| crate::error::ApiError::NotFound {
        entity: "FeedSubscription",
        id: s.to_string(),
    })
}

pub(crate) fn parse_collection_id(
    s: &str,
) -> Result<ind_domain::CollectionId, crate::error::ApiError> {
    s.parse().map_err(|_| crate::error::ApiError::BadRequest {
        message: format!("invalid collection ID: {s}"),
    })
}

fn deserialize_optional_nullable_string<'de, D>(
    deserializer: D,
) -> Result<Option<Option<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<Option<String>>::deserialize(deserializer)
}

fn deserialize_optional_nullable_i32<'de, D>(
    deserializer: D,
) -> Result<Option<Option<i32>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<Option<i32>>::deserialize(deserializer)
}
