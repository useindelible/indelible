use std::sync::Arc;

use chrono::Utc;
use futures::future::BoxFuture;
use tracing::info;

use crate::error::AppError;
use crate::ports::EmailIngestOperations;
use crate::repos::email_alias::EmailAliasRepository;
use crate::repos::email_ingest::EmailIngestLogRepository;
use crate::repos::feed::FeedRepository;
use crate::repos::feed_delivery::FeedDeliveryRepository;
use crate::repos::user::UserRepository;
use ind_domain::{
    FeedDelivery, FeedDeliveryId, FeedSource, FeedSourceEntry, FeedSourceEntryId, FeedSourceId,
    FeedStatus, FeedSubscription, FeedSubscriptionId, FeedType, FeedVisibility, UserId,
    parse_from_header,
};

pub struct FeedRouteInput<'a> {
    pub user_id: UserId,
    pub from_address: &'a str,
    pub from_display_name: Option<&'a str>,
    pub subject: &'a str,
    pub guid: &'a str,
    pub content_html: Option<&'a str>,
    pub excerpt: Option<&'a str>,
    pub language: Option<&'a str>,
}

pub struct EmailIngestService {
    user_repo: Arc<dyn UserRepository>,
    feed_repo: Arc<dyn FeedRepository>,
    delivery_repo: Arc<dyn FeedDeliveryRepository>,
    ingest_log_repo: Arc<dyn EmailIngestLogRepository>,
}

pub struct EmailIngestOperationsService {
    user_repo: Arc<dyn UserRepository>,
    ingest_log_repo: Arc<dyn EmailIngestLogRepository>,
    alias_repo: Arc<dyn EmailAliasRepository>,
}

impl EmailIngestOperationsService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        ingest_log_repo: Arc<dyn EmailIngestLogRepository>,
        alias_repo: Arc<dyn EmailAliasRepository>,
    ) -> Self {
        Self {
            user_repo,
            ingest_log_repo,
            alias_repo,
        }
    }
}

impl EmailIngestOperations for EmailIngestOperationsService {
    fn claim_and_enqueue(
        &self,
        input: crate::repos::email_ingest::ClaimAndEnqueueInput<'_>,
    ) -> BoxFuture<'_, Result<Option<crate::repos::email_ingest::EmailIngestLogRow>, AppError>>
    {
        let provider = input.provider.to_owned();
        let provider_email_id = input.provider_email_id.to_owned();
        let user_id = input.user_id;
        let destination = input.destination.to_owned();
        let job_type = input.job_type.to_owned();
        let job_payload = input.job_payload;
        let raw_payload = input.raw_payload.map(ToOwned::to_owned);
        let from_address = input.from_address.to_owned();
        let list_id = input.list_id.map(ToOwned::to_owned);
        Box::pin(async move {
            self.ingest_log_repo
                .claim_and_enqueue(crate::repos::email_ingest::ClaimAndEnqueueInput {
                    provider: &provider,
                    provider_email_id: &provider_email_id,
                    user_id,
                    destination: &destination,
                    job_type: &job_type,
                    job_payload,
                    raw_payload: raw_payload.as_deref(),
                    from_address: &from_address,
                    list_id: list_id.as_deref(),
                })
                .await
        })
    }

    fn resolve_ingest_recipient(
        &self,
        destination: ind_domain::EmailDestination,
        local_part: &str,
    ) -> BoxFuture<'_, Result<Option<ind_domain::User>, AppError>> {
        let local_part = local_part.to_owned();
        Box::pin(async move {
            if let Some(alias) = self
                .alias_repo
                .find_active(destination, &local_part)
                .await?
            {
                return self.user_repo.find_by_id(alias.user_id).await;
            }
            self.user_repo.find_by_email_token(&local_part).await
        })
    }
}

impl EmailIngestService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        feed_repo: Arc<dyn FeedRepository>,
        delivery_repo: Arc<dyn FeedDeliveryRepository>,
        ingest_log_repo: Arc<dyn EmailIngestLogRepository>,
    ) -> Self {
        Self {
            user_repo,
            feed_repo,
            delivery_repo,
            ingest_log_repo,
        }
    }

    pub fn user_repo(&self) -> &dyn UserRepository {
        &*self.user_repo
    }

    pub fn feed_repo(&self) -> &dyn FeedRepository {
        &*self.feed_repo
    }

    pub fn ingest_log_repo(&self) -> &dyn EmailIngestLogRepository {
        &*self.ingest_log_repo
    }

    pub async fn route_to_feed(
        &self,
        input: &FeedRouteInput<'_>,
    ) -> Result<FeedDelivery, AppError> {
        let canonical_key = build_newsletter_canonical_key(input.from_address);
        let source = self
            .find_or_create_source(&canonical_key, input.from_address, input.from_display_name)
            .await?;
        let subscription = self
            .find_or_create_subscription(input.user_id, &source, input.from_address)
            .await?;
        let guid = input.guid;
        let subject = input.subject;
        let content_html = input.content_html;
        let excerpt = input.excerpt;

        let existing = self
            .feed_repo
            .find_source_entry_by_source_guid(source.id, guid)
            .await?;

        if let Some(entry) = existing {
            return self.create_delivery(&subscription, &entry).await;
        }

        let now = Utc::now();
        let entry = FeedSourceEntry {
            id: FeedSourceEntryId::new(),
            source_id: source.id,
            guid: guid.to_string(),
            title: subject.to_string(),
            url: None,
            // Email entries have no source URL; identity is the origin (no canonical URL).
            canonical_url: None,
            author: input.from_display_name.map(String::from),
            excerpt: excerpt.map(String::from),
            content_html: content_html.map(String::from),
            language: input.language.map(String::from),
            lead_image_url: None,
            published_at: Some(now),
            discovered_at: now,
        };

        let entry = self.feed_repo.create_source_entry(entry).await?;

        self.create_delivery(&subscription, &entry).await
    }

    async fn find_or_create_source(
        &self,
        canonical_key: &str,
        from_address: &str,
        from_display_name: Option<&str>,
    ) -> Result<FeedSource, AppError> {
        if let Some(source) = self
            .feed_repo
            .find_source_by_canonical_key(canonical_key)
            .await?
        {
            return Ok(source);
        }

        let title = from_display_name.unwrap_or(from_address).to_string();
        let now = Utc::now();

        let source = FeedSource {
            id: FeedSourceId::new(),
            canonical_key: canonical_key.to_string(),
            source_url: format!("mailto:{from_address}"),
            poll_url: String::new(),
            title,
            description: None,
            site_url: None,
            image_url: None,
            domain: None,
            feed_type: FeedType::Newsletter,
            visibility: FeedVisibility::Private,
            provider: None,
            is_resolvable: false,
            popularity: 0,
            last_entry_added_at: None,
            last_polled_at: None,
            next_poll_at: None,
            last_etag: None,
            last_modified: None,
            consecutive_failures: 0,
            last_error: None,
            lease_owner: None,
            lease_expires_at: None,
            created_at: now,
            updated_at: now,
        };

        info!(
            canonical_key = canonical_key,
            from_address = from_address,
            "creating newsletter feed source"
        );

        self.feed_repo.create_source(source).await
    }

    async fn find_or_create_subscription(
        &self,
        user_id: UserId,
        source: &FeedSource,
        from_address: &str,
    ) -> Result<FeedSubscription, AppError> {
        if let Some(sub) = self
            .feed_repo
            .find_subscription_by_user_and_source(user_id, source.id)
            .await?
        {
            return Ok(sub);
        }

        let now = Utc::now();
        let subscription = FeedSubscription {
            id: FeedSubscriptionId::new(),
            user_id,
            source_id: source.id,
            input_url: format!("mailto:{from_address}"),
            title_override: None,
            auto_save: false,
            auto_save_collection_id: None,
            poll_interval_override_minutes: None,
            status: FeedStatus::Active,
            created_at: now,
            updated_at: now,
            source: source.clone(),
        };

        info!(
            user_id = %user_id,
            source_id = %source.id,
            "creating newsletter subscription"
        );

        self.feed_repo.create_subscription(subscription).await
    }

    async fn create_delivery(
        &self,
        subscription: &FeedSubscription,
        entry: &FeedSourceEntry,
    ) -> Result<FeedDelivery, AppError> {
        let now = Utc::now();
        let delivery = FeedDelivery {
            id: FeedDeliveryId::new(),
            subscription_id: subscription.id,
            source_id: subscription.source_id,
            source_entry_id: entry.id,
            user_id: subscription.user_id,
            document_id: None,
            delivered_at: now,
            seen_at: None,
            dismissed_at: None,
            hidden_at: None,
            created_at: now,
            updated_at: now,
        };

        self.delivery_repo
            .upsert_delivery(delivery)
            .await
            .map(|upsert| upsert.delivery)
    }
}

fn build_newsletter_canonical_key(from_address: &str) -> String {
    let (canonical, _) = parse_from_header(from_address);
    format!("newsletter:email:{}", canonical.as_str())
}

pub fn generate_excerpt(text: Option<&str>, max_len: usize) -> Option<String> {
    let text = text?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.len() <= max_len {
        return Some(trimmed.to_string());
    }
    let truncated = &trimmed[..trimmed.floor_char_boundary(max_len)];
    Some(format!("{truncated}..."))
}
