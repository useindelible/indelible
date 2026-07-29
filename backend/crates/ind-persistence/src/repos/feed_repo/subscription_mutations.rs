use ind_application::AppError;
use ind_domain::*;

use super::PgFeedRepository;
use super::subscription_rows::SubscriptionRow;
use super::types::*;

impl PgFeedRepository {
    pub(super) async fn create_subscription_impl(
        &self,
        subscription: FeedSubscription,
    ) -> Result<FeedSubscription, AppError> {
        let row = sqlx::query_as!(
            SubscriptionRow,
            "WITH inserted AS ( \
                INSERT INTO feed_subscriptions \
                    (id, user_id, source_id, input_url, title_override, auto_save, \
                     auto_save_collection_id, poll_interval_override_minutes, status, \
                     created_at, updated_at) \
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) \
                RETURNING id, user_id, source_id, input_url, title_override, auto_save, \
                          auto_save_collection_id, poll_interval_override_minutes, status, \
                          created_at, updated_at \
             ) \
             SELECT inserted.id, inserted.user_id, inserted.source_id, inserted.input_url, \
                    inserted.title_override, inserted.auto_save, inserted.auto_save_collection_id, \
                    inserted.poll_interval_override_minutes, inserted.status, inserted.created_at, \
                    inserted.updated_at, \
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
             FROM inserted \
             INNER JOIN feed_sources src ON src.id = inserted.source_id",
            subscription.id.into_uuid(),
            subscription.user_id.into_uuid(),
            subscription.source_id.into_uuid(),
            subscription.input_url,
            subscription.title_override,
            subscription.auto_save,
            subscription.auto_save_collection_id.map(|c| c.into_uuid()),
            subscription.poll_interval_override_minutes,
            feed_status_to_str(subscription.status),
            subscription.created_at,
            subscription.updated_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_subscription_error)?;

        FeedSubscription::try_from(row)
    }

    pub(super) async fn set_subscription_title_override_impl(
        &self,
        id: FeedSubscriptionId,
        user_id: UserId,
        title_override: Option<String>,
    ) -> Result<FeedSubscription, AppError> {
        let row = sqlx::query_as!(
            SubscriptionRow,
            "WITH updated AS ( \
                UPDATE feed_subscriptions \
                SET title_override = $3, updated_at = now() \
                WHERE id = $1 AND user_id = $2 \
                RETURNING id, user_id, source_id, input_url, title_override, auto_save, \
                          auto_save_collection_id, poll_interval_override_minutes, status, \
                          created_at, updated_at \
             ) \
             SELECT updated.id, updated.user_id, updated.source_id, updated.input_url, \
                    updated.title_override, updated.auto_save, updated.auto_save_collection_id, \
                    updated.poll_interval_override_minutes, updated.status, updated.created_at, \
                    updated.updated_at, \
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
             FROM updated \
             INNER JOIN feed_sources src ON src.id = updated.source_id",
            id.into_uuid(),
            user_id.into_uuid(),
            title_override,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_subscription_error)?
        .ok_or_else(|| AppError::Domain(DomainError::NotFound {
            entity: "FeedSubscription",
            id: id.to_string(),
        }))?;

        FeedSubscription::try_from(row)
    }

    pub(super) async fn set_subscription_auto_save_impl(
        &self,
        id: FeedSubscriptionId,
        user_id: UserId,
        auto_save: bool,
        collection_id: Option<Option<CollectionId>>,
    ) -> Result<FeedSubscription, AppError> {
        let row = match collection_id {
            None => {
                sqlx::query_as!(
                    SubscriptionRow,
                    "WITH updated AS ( \
                        UPDATE feed_subscriptions \
                        SET auto_save = $3, updated_at = now() \
                        WHERE id = $1 AND user_id = $2 \
                        RETURNING id, user_id, source_id, input_url, title_override, auto_save, \
                                  auto_save_collection_id, poll_interval_override_minutes, status, \
                                  created_at, updated_at \
                     ) \
                     SELECT updated.id, updated.user_id, updated.source_id, updated.input_url, \
                            updated.title_override, updated.auto_save, updated.auto_save_collection_id, \
                            updated.poll_interval_override_minutes, updated.status, updated.created_at, \
                            updated.updated_at, \
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
                     FROM updated \
                     INNER JOIN feed_sources src ON src.id = updated.source_id",
                    id.into_uuid(),
                    user_id.into_uuid(),
                    auto_save,
                )
                .fetch_optional(&self.pool)
                .await
                .map_err(map_subscription_error)?
            }
            Some(collection_id) => {
                sqlx::query_as!(
                    SubscriptionRow,
                    "WITH updated AS ( \
                        UPDATE feed_subscriptions \
                        SET auto_save = $3, auto_save_collection_id = $4, updated_at = now() \
                        WHERE id = $1 AND user_id = $2 \
                        RETURNING id, user_id, source_id, input_url, title_override, auto_save, \
                                  auto_save_collection_id, poll_interval_override_minutes, status, \
                                  created_at, updated_at \
                     ) \
                     SELECT updated.id, updated.user_id, updated.source_id, updated.input_url, \
                            updated.title_override, updated.auto_save, updated.auto_save_collection_id, \
                            updated.poll_interval_override_minutes, updated.status, updated.created_at, \
                            updated.updated_at, \
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
                     FROM updated \
                     INNER JOIN feed_sources src ON src.id = updated.source_id",
                    id.into_uuid(),
                    user_id.into_uuid(),
                    auto_save,
                    collection_id.map(|c| c.into_uuid()),
                )
                .fetch_optional(&self.pool)
                .await
                .map_err(map_subscription_error)?
            }
        }
        .ok_or_else(|| AppError::Domain(DomainError::NotFound {
            entity: "FeedSubscription",
            id: id.to_string(),
        }))?;

        FeedSubscription::try_from(row)
    }

    pub(super) async fn set_subscription_poll_interval_impl(
        &self,
        id: FeedSubscriptionId,
        user_id: UserId,
        poll_interval_override_minutes: Option<i32>,
    ) -> Result<FeedSubscription, AppError> {
        let row = sqlx::query_as!(
            SubscriptionRow,
            "WITH updated AS ( \
                UPDATE feed_subscriptions \
                SET poll_interval_override_minutes = $3, updated_at = now() \
                WHERE id = $1 AND user_id = $2 \
                RETURNING id, user_id, source_id, input_url, title_override, auto_save, \
                          auto_save_collection_id, poll_interval_override_minutes, status, \
                          created_at, updated_at \
             ) \
             SELECT updated.id, updated.user_id, updated.source_id, updated.input_url, \
                    updated.title_override, updated.auto_save, updated.auto_save_collection_id, \
                    updated.poll_interval_override_minutes, updated.status, updated.created_at, \
                    updated.updated_at, \
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
             FROM updated \
             INNER JOIN feed_sources src ON src.id = updated.source_id",
            id.into_uuid(),
            user_id.into_uuid(),
            poll_interval_override_minutes,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_subscription_error)?
        .ok_or_else(|| AppError::Domain(DomainError::NotFound {
            entity: "FeedSubscription",
            id: id.to_string(),
        }))?;

        FeedSubscription::try_from(row)
    }

    pub(super) async fn set_subscription_status_impl(
        &self,
        id: FeedSubscriptionId,
        user_id: UserId,
        status: FeedStatus,
    ) -> Result<FeedSubscription, AppError> {
        let row = sqlx::query_as!(
            SubscriptionRow,
            "WITH updated AS ( \
                UPDATE feed_subscriptions \
                SET status = $3, updated_at = now() \
                WHERE id = $1 AND user_id = $2 \
                RETURNING id, user_id, source_id, input_url, title_override, auto_save, \
                          auto_save_collection_id, poll_interval_override_minutes, status, \
                          created_at, updated_at \
             ) \
             SELECT updated.id, updated.user_id, updated.source_id, updated.input_url, \
                    updated.title_override, updated.auto_save, updated.auto_save_collection_id, \
                    updated.poll_interval_override_minutes, updated.status, updated.created_at, \
                    updated.updated_at, \
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
             FROM updated \
             INNER JOIN feed_sources src ON src.id = updated.source_id",
            id.into_uuid(),
            user_id.into_uuid(),
            feed_status_to_str(status),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_subscription_error)?
        .ok_or_else(|| AppError::Domain(DomainError::NotFound {
            entity: "FeedSubscription",
            id: id.to_string(),
        }))?;

        FeedSubscription::try_from(row)
    }
}
