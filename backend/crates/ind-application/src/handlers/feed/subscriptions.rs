use chrono::Utc;

use crate::error::AppError;
use crate::repos::{Cursor, Page};
use ind_domain::{
    DomainError, FeedSearchSurface, FeedSource, FeedSourceId, FeedStatus, FeedSubscription,
    FeedSubscriptionId, FeedVisibility, UserId, job_types,
};

use super::{
    FeedService, ResolvedFeedSource, SubscribeInput, SubscribeResult, UpdateSubscriptionInput,
};

impl FeedService {
    pub async fn search_public_sources(
        &self,
        query: &str,
        surface: FeedSearchSurface,
        limit: u32,
    ) -> Result<Vec<FeedSource>, AppError> {
        self.feed_repo
            .search_public_sources(query, surface, limit)
            .await
    }
    pub async fn subscribe(
        &self,
        user_id: UserId,
        input: SubscribeInput,
    ) -> Result<SubscribeResult, AppError> {
        let resolved = self.resolve_source(user_id, &input.url).await?;
        let source = self.find_or_create_source(resolved).await?;

        if let Some(existing) = self
            .feed_repo
            .find_subscription_by_user_and_source(user_id, source.id)
            .await?
        {
            return Err(AppError::Domain(DomainError::Conflict {
                entity: "FeedSubscription",
                message: format!("already subscribed to this feed ({})", existing.id),
            }));
        }

        let now = Utc::now();
        let subscription = FeedSubscription {
            id: FeedSubscriptionId::new(),
            user_id,
            source_id: source.id,
            input_url: input.url,
            title_override: input.title_override,
            auto_save: false,
            auto_save_collection_id: None,
            poll_interval_override_minutes: input.poll_interval_override_minutes,
            status: FeedStatus::Active,
            created_at: now,
            updated_at: now,
            source: source.clone(),
        };

        self.feed_repo.create_subscription(subscription).await?;

        if source.visibility == FeedVisibility::Public {
            self.feed_repo.bump_source_popularity(source.id, 1).await?;
        }

        self.request_poll(source.id).await?;

        let subscription = self
            .feed_repo
            .find_subscription_by_user_and_source(user_id, source.id)
            .await?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "FeedSubscription",
                    id: source.id.to_string(),
                })
            })?;

        Ok(SubscribeResult {
            subscription,
            is_new: true,
        })
    }
    pub async fn unsubscribe(
        &self,
        user_id: UserId,
        id: FeedSubscriptionId,
    ) -> Result<(), AppError> {
        let source_id = self.feed_repo.delete_subscription(id, user_id).await?;
        self.feed_repo.delete_source_if_orphaned(source_id).await
    }
    pub async fn update_subscription(
        &self,
        user_id: UserId,
        id: FeedSubscriptionId,
        input: UpdateSubscriptionInput,
    ) -> Result<FeedSubscription, AppError> {
        let subscription = self
            .feed_repo
            .find_subscription_by_id(id)
            .await?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "FeedSubscription",
                    id: id.to_string(),
                })
            })?;

        if subscription.user_id != user_id {
            return Err(AppError::Domain(DomainError::NotFound {
                entity: "FeedSubscription",
                id: id.to_string(),
            }));
        }

        if let Some(title_override) = input.title_override {
            self.feed_repo
                .set_subscription_title_override(id, user_id, title_override)
                .await?;
        }

        if input.auto_save.is_some() || input.auto_save_collection_id.is_some() {
            self.feed_repo
                .set_subscription_auto_save(
                    id,
                    user_id,
                    input.auto_save.unwrap_or(subscription.auto_save),
                    input.auto_save_collection_id,
                )
                .await?;
        }

        if let Some(poll_interval_override_minutes) = input.poll_interval_override_minutes {
            self.feed_repo
                .set_subscription_poll_interval(id, user_id, poll_interval_override_minutes)
                .await?;
            self.request_poll(subscription.source_id).await?;
        }

        if let Some(status) = input.status {
            self.feed_repo
                .set_subscription_status(id, user_id, status)
                .await?;
        }

        self.feed_repo
            .find_subscription_by_id(id)
            .await?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "FeedSubscription",
                    id: id.to_string(),
                })
            })
    }
    pub async fn list_subscriptions(
        &self,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<FeedSubscription>, AppError> {
        self.feed_repo
            .list_subscriptions_by_user(user_id, cursor, limit)
            .await
    }
    pub async fn retry_subscription(
        &self,
        user_id: UserId,
        id: FeedSubscriptionId,
    ) -> Result<FeedSubscription, AppError> {
        let mut subscription = self
            .feed_repo
            .find_subscription_by_id(id)
            .await?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "FeedSubscription",
                    id: id.to_string(),
                })
            })?;

        if subscription.user_id != user_id {
            return Err(AppError::Domain(DomainError::NotFound {
                entity: "FeedSubscription",
                id: id.to_string(),
            }));
        }

        if matches!(subscription.status, FeedStatus::Error) {
            subscription = self
                .feed_repo
                .set_subscription_status(id, user_id, FeedStatus::Active)
                .await?;
        }

        self.request_poll(subscription.source_id).await?;
        self.feed_repo
            .find_subscription_by_id(id)
            .await?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "FeedSubscription",
                    id: id.to_string(),
                })
            })
    }
    async fn find_or_create_source(
        &self,
        resolved: ResolvedFeedSource,
    ) -> Result<FeedSource, AppError> {
        if let Some(existing) = self
            .feed_repo
            .find_source_by_canonical_key(&resolved.canonical_key)
            .await?
        {
            return Ok(existing);
        }

        let now = Utc::now();
        let source = FeedSource {
            id: FeedSourceId::new(),
            canonical_key: resolved.canonical_key,
            source_url: resolved.source_url,
            poll_url: resolved.poll_url,
            title: resolved.title,
            description: resolved.description,
            site_url: resolved.site_url,
            image_url: resolved.image_url,
            domain: resolved.domain,
            feed_type: resolved.feed_type,
            visibility: resolved.visibility,
            provider: resolved.provider,
            is_resolvable: resolved.is_resolvable,
            popularity: 0,
            last_entry_added_at: None,
            last_polled_at: None,
            next_poll_at: Some(now),
            last_etag: None,
            last_modified: None,
            consecutive_failures: 0,
            last_error: None,
            lease_owner: None,
            lease_expires_at: None,
            created_at: now,
            updated_at: now,
        };

        self.feed_repo.create_source(source).await
    }
    async fn request_poll(&self, source_id: FeedSourceId) -> Result<(), AppError> {
        let when = Utc::now();
        self.feed_repo
            .mark_source_poll_requested(source_id, when)
            .await?;
        let payload = serde_json::to_value(ind_domain::FeedPollJob { source_id })
            .map_err(|e| AppError::Repository(Box::new(e)))?;
        self.outbox_repo
            .enqueue(
                job_types::FEED_POLL,
                payload,
                Some(format!("{}:{source_id}", job_types::FEED_POLL)),
                when,
            )
            .await?;
        Ok(())
    }
}
