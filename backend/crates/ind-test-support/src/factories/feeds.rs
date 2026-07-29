use super::prelude::*;

pub struct FeedSourceFactory;

impl FeedSourceFactory {
    pub async fn insert(self, pool: &sqlx::PgPool) -> FeedSource {
        let timestamp = Utc::now();
        let poll_url = format!(
            "https://{}.{}/feed.xml",
            uuid::Uuid::now_v7().simple(),
            fake::faker::internet::en::DomainSuffix().fake::<String>()
        );
        PgFeedRepository::new(pool.clone())
            .create_source(FeedSource {
                id: FeedSourceId::new(),
                canonical_key: format!("public:url:{poll_url}"),
                source_url: poll_url.clone(),
                poll_url,
                title: "Test Feed".into(),
                description: None,
                site_url: None,
                image_url: None,
                domain: None,
                feed_type: FeedType::Rss,
                visibility: FeedVisibility::Public,
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
                created_at: timestamp,
                updated_at: timestamp,
            })
            .await
            .expect("FeedSourceFactory::insert failed")
    }
}

pub struct FeedSubscriptionFactory {
    user_id: UserId,
    source: Option<FeedSource>,
}

impl FeedSubscriptionFactory {
    pub fn new(user_id: UserId) -> Self {
        Self {
            user_id,
            source: None,
        }
    }

    pub fn with_source(mut self, source: FeedSource) -> Self {
        self.source = Some(source);
        self
    }

    pub async fn insert(self, pool: &sqlx::PgPool) -> FeedSubscription {
        let source = match self.source {
            Some(source) => source,
            None => FeedSourceFactory.insert(pool).await,
        };
        let timestamp = Utc::now();
        PgFeedRepository::new(pool.clone())
            .create_subscription(FeedSubscription {
                id: FeedSubscriptionId::new(),
                user_id: self.user_id,
                source_id: source.id,
                input_url: source.poll_url.clone(),
                title_override: None,
                auto_save: false,
                auto_save_collection_id: None,
                poll_interval_override_minutes: None,
                status: FeedStatus::Active,
                created_at: timestamp,
                updated_at: timestamp,
                source,
            })
            .await
            .expect("FeedSubscriptionFactory::insert failed")
    }
}
