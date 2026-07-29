use ind_application::AppError;
use ind_application::repos::{Cursor, Page};
use ind_domain::*;
use uuid::Uuid;

use crate::cursor::{clamp_limit, decode_cursor_ts, encode_cursor_ts};

use super::PgFeedRepository;
use super::subscription_rows::SubscriptionRow;
use super::types::*;

impl PgFeedRepository {
    pub(super) async fn find_subscription_by_id_impl(
        &self,
        id: FeedSubscriptionId,
    ) -> Result<Option<FeedSubscription>, AppError> {
        let row = sqlx::query_as!(
            SubscriptionRow,
            "SELECT fs.id, fs.user_id, fs.source_id, fs.input_url, fs.title_override, \
                    fs.auto_save, fs.auto_save_collection_id, fs.poll_interval_override_minutes, \
                    fs.status, fs.created_at, fs.updated_at, \
                    src.canonical_key AS source_canonical_key, src.source_url AS source_source_url, \
                    src.poll_url AS source_poll_url, src.title AS source_title, \
                    src.description AS source_description, src.site_url AS source_site_url, \
                    src.image_url AS source_image_url, src.domain AS source_domain, \
                    src.feed_type AS source_feed_type, src.visibility AS source_visibility, \
                    src.provider AS source_provider, src.is_resolvable AS source_is_resolvable, \
                    src.popularity AS source_popularity, \
                    src.last_entry_added_at AS source_last_entry_added_at, \
                    src.last_polled_at AS source_last_polled_at, \
                    src.next_poll_at AS source_next_poll_at, \
                    src.last_etag AS source_last_etag, \
                    src.last_modified AS source_last_modified, \
                    src.consecutive_failures AS source_consecutive_failures, \
                    src.last_error AS source_last_error, \
                    src.lease_owner AS source_lease_owner, \
                    src.lease_expires_at AS source_lease_expires_at, \
                    src.created_at AS source_created_at, \
                    src.updated_at AS source_updated_at \
             FROM feed_subscriptions fs \
             INNER JOIN feed_sources src ON src.id = fs.source_id \
             WHERE fs.id = $1",
            id.into_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_subscription_error)?;

        row.map(FeedSubscription::try_from).transpose()
    }

    pub(super) async fn find_subscription_by_user_and_source_impl(
        &self,
        user_id: UserId,
        source_id: FeedSourceId,
    ) -> Result<Option<FeedSubscription>, AppError> {
        let row = sqlx::query_as!(
            SubscriptionRow,
            "SELECT fs.id, fs.user_id, fs.source_id, fs.input_url, fs.title_override, \
                    fs.auto_save, fs.auto_save_collection_id, fs.poll_interval_override_minutes, \
                    fs.status, fs.created_at, fs.updated_at, \
                    src.canonical_key AS source_canonical_key, src.source_url AS source_source_url, \
                    src.poll_url AS source_poll_url, src.title AS source_title, \
                    src.description AS source_description, src.site_url AS source_site_url, \
                    src.image_url AS source_image_url, src.domain AS source_domain, \
                    src.feed_type AS source_feed_type, src.visibility AS source_visibility, \
                    src.provider AS source_provider, src.is_resolvable AS source_is_resolvable, \
                    src.popularity AS source_popularity, \
                    src.last_entry_added_at AS source_last_entry_added_at, \
                    src.last_polled_at AS source_last_polled_at, \
                    src.next_poll_at AS source_next_poll_at, \
                    src.last_etag AS source_last_etag, \
                    src.last_modified AS source_last_modified, \
                    src.consecutive_failures AS source_consecutive_failures, \
                    src.last_error AS source_last_error, \
                    src.lease_owner AS source_lease_owner, \
                    src.lease_expires_at AS source_lease_expires_at, \
                    src.created_at AS source_created_at, \
                    src.updated_at AS source_updated_at \
             FROM feed_subscriptions fs \
             INNER JOIN feed_sources src ON src.id = fs.source_id \
             WHERE fs.user_id = $1 AND fs.source_id = $2",
            user_id.into_uuid(),
            source_id.into_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_subscription_error)?;

        row.map(FeedSubscription::try_from).transpose()
    }

    pub(super) async fn delete_subscription_impl(
        &self,
        id: FeedSubscriptionId,
        user_id: UserId,
    ) -> Result<FeedSourceId, AppError> {
        let row = sqlx::query!(
            "DELETE FROM feed_subscriptions \
             WHERE id = $1 AND user_id = $2 \
             RETURNING source_id",
            id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_subscription_error)?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "FeedSubscription",
                id: id.to_string(),
            })
        })?;

        Ok(FeedSourceId::from_uuid(row.source_id))
    }

    pub(super) async fn delete_source_if_orphaned_impl(
        &self,
        id: FeedSourceId,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "DELETE FROM feed_sources \
             WHERE id = $1 \
               AND NOT EXISTS (SELECT 1 FROM feed_subscriptions WHERE source_id = $1)",
            id.into_uuid(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_source_error)?;

        Ok(())
    }

    pub(super) async fn list_subscriptions_by_user_impl(
        &self,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<FeedSubscription>, AppError> {
        let limit = clamp_limit(limit);
        let fetch_limit = limit + 1;

        let rows = if let Some(ref cursor) = cursor {
            let (ts, id) = decode_cursor_ts(cursor)?;
            sqlx::query_as!(
                SubscriptionRow,
                "SELECT fs.id, fs.user_id, fs.source_id, fs.input_url, fs.title_override, \
                        fs.auto_save, fs.auto_save_collection_id, fs.poll_interval_override_minutes, \
                        fs.status, fs.created_at, fs.updated_at, \
                        src.canonical_key AS source_canonical_key, src.source_url AS source_source_url, \
                        src.poll_url AS source_poll_url, src.title AS source_title, \
                        src.description AS source_description, src.site_url AS source_site_url, \
                        src.image_url AS source_image_url, src.domain AS source_domain, \
                        src.feed_type AS source_feed_type, src.visibility AS source_visibility, \
                        src.provider AS source_provider, src.is_resolvable AS source_is_resolvable, \
                        src.popularity AS source_popularity, \
                        src.last_entry_added_at AS source_last_entry_added_at, \
                        src.last_polled_at AS source_last_polled_at, \
                        src.next_poll_at AS source_next_poll_at, src.last_etag AS source_last_etag, \
                        src.last_modified AS source_last_modified, \
                        src.consecutive_failures AS source_consecutive_failures, \
                        src.last_error AS source_last_error, src.lease_owner AS source_lease_owner, \
                        src.lease_expires_at AS source_lease_expires_at, \
                        src.created_at AS source_created_at, src.updated_at AS source_updated_at \
                 FROM feed_subscriptions fs \
                 INNER JOIN feed_sources src ON src.id = fs.source_id \
                 WHERE fs.user_id = $1 AND (fs.created_at, fs.id) < ($2, $3) \
                 ORDER BY fs.created_at DESC, fs.id DESC \
                 LIMIT $4",
                user_id.into_uuid(),
                ts,
                id,
                fetch_limit,
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_subscription_error)?
        } else {
            sqlx::query_as!(
                SubscriptionRow,
                "SELECT fs.id, fs.user_id, fs.source_id, fs.input_url, fs.title_override, \
                        fs.auto_save, fs.auto_save_collection_id, fs.poll_interval_override_minutes, \
                        fs.status, fs.created_at, fs.updated_at, \
                        src.canonical_key AS source_canonical_key, src.source_url AS source_source_url, \
                        src.poll_url AS source_poll_url, src.title AS source_title, \
                        src.description AS source_description, src.site_url AS source_site_url, \
                        src.image_url AS source_image_url, src.domain AS source_domain, \
                        src.feed_type AS source_feed_type, src.visibility AS source_visibility, \
                        src.provider AS source_provider, src.is_resolvable AS source_is_resolvable, \
                        src.popularity AS source_popularity, \
                        src.last_entry_added_at AS source_last_entry_added_at, \
                        src.last_polled_at AS source_last_polled_at, \
                        src.next_poll_at AS source_next_poll_at, src.last_etag AS source_last_etag, \
                        src.last_modified AS source_last_modified, \
                        src.consecutive_failures AS source_consecutive_failures, \
                        src.last_error AS source_last_error, src.lease_owner AS source_lease_owner, \
                        src.lease_expires_at AS source_lease_expires_at, \
                        src.created_at AS source_created_at, src.updated_at AS source_updated_at \
                 FROM feed_subscriptions fs \
                 INNER JOIN feed_sources src ON src.id = fs.source_id \
                 WHERE fs.user_id = $1 \
                 ORDER BY fs.created_at DESC, fs.id DESC \
                 LIMIT $2",
                user_id.into_uuid(),
                fetch_limit,
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_subscription_error)?
        };

        let has_more = rows.len() as i64 > limit;
        let items = rows
            .into_iter()
            .take(limit as usize)
            .map(FeedSubscription::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        let next_cursor = if has_more {
            items
                .last()
                .map(|item| encode_cursor_ts(item.created_at, item.id.into_uuid()))
        } else {
            None
        };

        Ok(Page { items, next_cursor })
    }

    pub(super) async fn list_active_subscriptions_for_source_impl(
        &self,
        source_id: FeedSourceId,
    ) -> Result<Vec<ActiveSubscription>, AppError> {
        struct Row {
            id: Uuid,
            user_id: Uuid,
            source_id: Uuid,
            auto_save: bool,
            auto_save_collection_id: Option<Uuid>,
            poll_interval_override_minutes: Option<i32>,
        }

        let rows = sqlx::query_as!(
            Row,
            "SELECT id, user_id, source_id, auto_save, auto_save_collection_id, \
                    poll_interval_override_minutes \
             FROM feed_subscriptions \
             WHERE source_id = $1 AND status = 'active' \
             ORDER BY created_at ASC",
            source_id.into_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_subscription_error)?;

        Ok(rows
            .into_iter()
            .map(|r| ActiveSubscription {
                id: FeedSubscriptionId::from(r.id),
                user_id: UserId::from(r.user_id),
                source_id: FeedSourceId::from(r.source_id),
                auto_save: r.auto_save,
                auto_save_collection_id: r.auto_save_collection_id.map(CollectionId::from),
                poll_interval_override_minutes: r.poll_interval_override_minutes,
            })
            .collect())
    }
}
