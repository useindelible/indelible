use chrono::{DateTime, Utc};
use ind_application::AppError;
use ind_domain::*;
use uuid::Uuid;

use super::types::*;

pub(super) struct SubscriptionRow {
    pub(super) id: Uuid,
    pub(super) user_id: Uuid,
    pub(super) source_id: Uuid,
    pub(super) input_url: String,
    pub(super) title_override: Option<String>,
    pub(super) auto_save: bool,
    pub(super) auto_save_collection_id: Option<Uuid>,
    pub(super) poll_interval_override_minutes: Option<i32>,
    pub(super) status: String,
    pub(super) created_at: DateTime<Utc>,
    pub(super) updated_at: DateTime<Utc>,
    pub(super) source_canonical_key: String,
    pub(super) source_source_url: String,
    pub(super) source_poll_url: String,
    pub(super) source_title: String,
    pub(super) source_description: Option<String>,
    pub(super) source_site_url: Option<String>,
    pub(super) source_image_url: Option<String>,
    pub(super) source_domain: Option<String>,
    pub(super) source_feed_type: String,
    pub(super) source_visibility: String,
    pub(super) source_provider: Option<String>,
    pub(super) source_is_resolvable: bool,
    pub(super) source_popularity: i32,
    pub(super) source_last_entry_added_at: Option<DateTime<Utc>>,
    pub(super) source_last_polled_at: Option<DateTime<Utc>>,
    pub(super) source_next_poll_at: Option<DateTime<Utc>>,
    pub(super) source_last_etag: Option<String>,
    pub(super) source_last_modified: Option<String>,
    pub(super) source_consecutive_failures: i32,
    pub(super) source_last_error: Option<String>,
    pub(super) source_lease_owner: Option<String>,
    pub(super) source_lease_expires_at: Option<DateTime<Utc>>,
    pub(super) source_created_at: DateTime<Utc>,
    pub(super) source_updated_at: DateTime<Utc>,
}

impl TryFrom<SubscriptionRow> for FeedSubscription {
    type Error = AppError;

    fn try_from(row: SubscriptionRow) -> Result<Self, Self::Error> {
        let source = FeedSource {
            id: FeedSourceId::from_uuid(row.source_id),
            canonical_key: row.source_canonical_key,
            source_url: row.source_source_url,
            poll_url: row.source_poll_url,
            title: row.source_title,
            description: row.source_description,
            site_url: row.source_site_url,
            image_url: row.source_image_url,
            domain: row.source_domain,
            feed_type: parse_feed_type(&row.source_feed_type)?,
            visibility: parse_visibility(&row.source_visibility)?,
            provider: row.source_provider,
            is_resolvable: row.source_is_resolvable,
            popularity: row.source_popularity,
            last_entry_added_at: row.source_last_entry_added_at,
            last_polled_at: row.source_last_polled_at,
            next_poll_at: row.source_next_poll_at,
            last_etag: row.source_last_etag,
            last_modified: row.source_last_modified,
            consecutive_failures: row.source_consecutive_failures,
            last_error: row.source_last_error,
            lease_owner: row.source_lease_owner,
            lease_expires_at: row.source_lease_expires_at,
            created_at: row.source_created_at,
            updated_at: row.source_updated_at,
        };

        Ok(FeedSubscription {
            id: FeedSubscriptionId::from_uuid(row.id),
            user_id: UserId::from_uuid(row.user_id),
            source_id: FeedSourceId::from_uuid(row.source_id),
            input_url: row.input_url,
            title_override: row.title_override,
            auto_save: row.auto_save,
            auto_save_collection_id: row.auto_save_collection_id.map(CollectionId::from_uuid),
            poll_interval_override_minutes: row.poll_interval_override_minutes,
            status: parse_feed_status(&row.status)?,
            created_at: row.created_at,
            updated_at: row.updated_at,
            source,
        })
    }
}
