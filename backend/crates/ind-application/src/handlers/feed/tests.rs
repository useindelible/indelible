use url::Url;

use super::discovery::{discover_feed_url, is_private_feed_url};
use super::providers::{parse_twitter_url, parse_youtube_url};

#[test]
fn youtube_and_twitter_routes_normalize_supported_sources_and_reject_spoofs() {
    for (raw, canonical, provider) in [
        (
            "https://youtube.com/@Veritasium",
            "youtube:user:veritasium",
            None,
        ),
        (
            "https://youtube.com/channel/UC123",
            "youtube:channel:UC123",
            None,
        ),
        (
            "https://youtube.com/user/Veritasium",
            "youtube:user:veritasium",
            None,
        ),
        (
            "https://youtube.com/c/Creators",
            "youtube:custom:creators",
            None,
        ),
        (
            "https://rsshub.example/youtube/user/@Veritasium",
            "youtube:user:veritasium",
            Some("rsshub"),
        ),
        (
            "https://rsshub.example/youtube/user/Veritasium",
            "youtube:user:veritasium",
            Some("rsshub"),
        ),
        (
            "https://rsshub.example/youtube/channel/UC123",
            "youtube:channel:UC123",
            Some("rsshub"),
        ),
        (
            "https://rsshub.example/youtube/c/Creators",
            "youtube:custom:creators",
            Some("rsshub"),
        ),
    ] {
        let parsed = parse_youtube_url(&Url::parse(raw).unwrap()).unwrap();
        assert_eq!(parsed.canonical_key, canonical, "{raw}");
        assert_eq!(parsed.input_provider.as_deref(), provider, "{raw}");
    }
    for raw in [
        "https://not-youtube.com/@veritasium",
        "https://youtube.com.evil.test/@veritasium",
        "https://youtube.com/@",
        "https://youtube.com/feeds/videos.xml?channel_id=UC123",
        "https://youtube.com/watch",
        "https://rsshub.example/youtube/unknown/name",
    ] {
        assert!(
            parse_youtube_url(&Url::parse(raw).unwrap()).is_none(),
            "{raw}"
        );
    }

    for (raw, canonical, provider) in [
        ("https://x.com/SpaceX", "twitter:user:spacex", None),
        (
            "https://nitter.example/SpaceX/rss",
            "twitter:user:spacex",
            Some("nitter"),
        ),
        (
            "https://rsshub.example/twitter/user/SpaceX",
            "twitter:user:spacex",
            Some("rsshub"),
        ),
    ] {
        let parsed = parse_twitter_url(&Url::parse(raw).unwrap()).unwrap();
        assert_eq!(parsed.canonical_key, canonical, "{raw}");
        assert_eq!(parsed.input_provider.as_deref(), provider, "{raw}");
    }
    for raw in [
        "https://x.com/home",
        "https://x.com",
        "https://x.com.evil.test/SpaceX",
    ] {
        assert!(
            parse_twitter_url(&Url::parse(raw).unwrap()).is_none(),
            "{raw}"
        );
    }
}

#[test]
fn discovery_recognizes_private_sources_and_resolves_safe_alternate_links() {
    for raw in [
        "https://user:pass@example.com/feed",
        "https://example.com/feed?token=secret",
        "https://example.com/collections/abc/rss",
    ] {
        assert!(is_private_feed_url(&Url::parse(raw).unwrap()), "{raw}");
    }
    assert!(!is_private_feed_url(
        &Url::parse("https://example.com/public.xml").unwrap()
    ));

    let base = Url::parse("https://example.com/blog/post").unwrap();
    for (html, expected) in [
        (
            r#"<link rel="alternate" type="application/rss+xml" href="/feed.xml">"#,
            "https://example.com/feed.xml",
        ),
        (
            "<link rel='alternate' type='application/atom+xml' href='../atom.xml'>",
            "https://example.com/atom.xml",
        ),
        (
            "<link rel=alternate type=application/rss+xml href=feed.xml>",
            "https://example.com/blog/feed.xml",
        ),
    ] {
        assert_eq!(
            discover_feed_url(&base, Some(html)).unwrap().as_str(),
            expected
        );
    }
    assert!(discover_feed_url(&base, Some("<link rel='stylesheet' href='x.css'>")).is_none());
    assert!(discover_feed_url(&base, None).is_none());
}

mod resolver {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use chrono::{DateTime, Duration, Utc};

    use crate::error::AppError;
    use crate::handlers::feed::{FeedService, SubscribeInput};
    use crate::ports::{
        FeedParseError, FeedParser, FetchRequest, FetchResponse, HttpFetchError, HttpFetcher,
        OpmlParseError, OpmlParser, ParsedFeed, ParsedFeedKind,
    };
    use crate::repos::feed::FeedRepository;
    use crate::repos::outbox::JobOutboxRepository;
    use crate::repos::{Cursor, Page};
    use ind_domain::{
        ActiveSubscription, CollectionId, FeedProviderInstance, FeedSearchSurface, FeedSource,
        FeedSourceEntry, FeedSourceEntryId, FeedSourceId, FeedStatus, FeedSubscription,
        FeedSubscriptionId, FeedType, FeedVisibility, JobOutbox, PollOutcome, SourceDetailsUpdate,
        UserId,
    };

    use super::super::providers::channel_id_from_page;

    const CHANNEL_ID: &str = "UCHnyfMqiRRG1u-2MsSQLbXA";
    const NATIVE_ATOM_URL: &str =
        "https://www.youtube.com/feeds/videos.xml?channel_id=UCHnyfMqiRRG1u-2MsSQLbXA";
    const ATOM_BODY: &str = r#"<?xml version="1.0"?><feed xmlns="http://www.w3.org/2005/Atom"><title>Fixture Channel</title></feed>"#;
    const SITE_URL: &str = "https://example.com/";
    const SITE_FEED_URL: &str = "https://example.com/feed.xml";
    const SITE_HTML: &str = r#"<html><head><link rel="alternate" type="application/rss+xml" href="/feed.xml"></head></html>"#;
    const PRIVATE_SITE_FEED_URL: &str = "https://example.com/feed.xml?token=secret";
    const PRIVATE_SITE_HTML: &str = r#"<html><head><link rel="alternate" type="application/rss+xml" href="/feed.xml?token=secret"></head></html>"#;

    fn handle_page_html() -> String {
        // Decoy channelId entries mimic the recommendation shelves a real
        // channel page embeds before its own canonical link.
        format!(
            r#"<html><head>
            <script>var x = {{"channelId":"UCdecoy0000000000000001","canonicalBaseUrl":"/channel/UCdecoy0000000000000002"}}</script>
            <link rel="canonical" href="https://www.youtube.com/channel/{CHANNEL_ID}">
            <link rel="alternate" type="application/rss+xml" href="https://www.youtube.com/feeds/videos.xml?channel_id={CHANNEL_ID}">
            </head><body></body></html>"#
        )
    }

    #[test]
    fn channel_id_extraction_prefers_the_canonical_link_over_decoys() {
        assert_eq!(
            channel_id_from_page(&handle_page_html()),
            Some(CHANNEL_ID.to_string())
        );
    }

    #[test]
    fn channel_id_extraction_falls_back_to_the_rss_alternate_link() {
        let html = format!(
            r#"<link rel="alternate" type="application/rss+xml" href="https://www.youtube.com/feeds/videos.xml?channel_id={CHANNEL_ID}">"#
        );
        assert_eq!(channel_id_from_page(&html), Some(CHANNEL_ID.to_string()));
    }

    #[test]
    fn channel_id_extraction_is_independent_of_attribute_order_and_quote_style() {
        // Attribute order inside a tag is arbitrary in valid HTML, and either
        // quote style is legal; neither changes which link is canonical.
        for html in [
            format!(
                r#"<link href="https://www.youtube.com/channel/{CHANNEL_ID}" rel="canonical">"#
            ),
            format!(
                r#"<link rel='canonical' href='https://www.youtube.com/channel/{CHANNEL_ID}'>"#
            ),
            format!(
                r#"<link href='https://www.youtube.com/channel/{CHANNEL_ID}' rel='canonical'/>"#
            ),
            // Unquoted values are legal too: the scheme's own slashes must not
            // terminate the value, and a self-closing solidus must not stick to
            // the last attribute.
            format!(r#"<link href=https://www.youtube.com/channel/{CHANNEL_ID} rel=canonical>"#),
            format!(r#"<link rel=canonical href=https://www.youtube.com/channel/{CHANNEL_ID}/>"#),
            format!(
                r#"<link rel="alternate" type="application/rss+xml" href="https://www.youtube.com/feeds/videos.xml?channel_id=UCdecoy0000000000000001">
                   <link href="https://www.youtube.com/channel/{CHANNEL_ID}" rel="canonical">"#
            ),
        ] {
            assert_eq!(
                channel_id_from_page(&html),
                Some(CHANNEL_ID.to_string()),
                "failed for {html}"
            );
        }
    }

    #[test]
    fn channel_id_extraction_rejects_pages_without_a_trustworthy_link() {
        assert_eq!(channel_id_from_page("<html><body>nope</body></html>"), None);
        assert_eq!(
            channel_id_from_page(r#"{"channelId":"UCdecoy0000000000000001"}"#),
            None
        );
        // Malformed id in an otherwise-right place.
        assert_eq!(
            channel_id_from_page(
                r#"<link rel="canonical" href="https://www.youtube.com/channel/short">"#
            ),
            None
        );
    }

    fn support_gap() -> AppError {
        AppError::ExternalService {
            service: "test-double".into(),
            message: "not used by this test".into(),
        }
    }

    #[derive(Default)]
    struct RecordingFetcher {
        bodies: HashMap<String, (String, &'static str)>,
        requests: Mutex<Vec<String>>,
    }

    impl RecordingFetcher {
        fn with(mut self, url: &str, body: &str, content_type: &'static str) -> Self {
            self.bodies
                .insert(url.to_string(), (body.to_string(), content_type));
            self
        }

        fn requested(&self) -> Vec<String> {
            self.requests.lock().unwrap().clone()
        }
    }

    #[async_trait::async_trait]
    impl HttpFetcher for RecordingFetcher {
        async fn fetch(&self, request: FetchRequest) -> Result<FetchResponse, HttpFetchError> {
            self.requests.lock().unwrap().push(request.url.clone());
            match self.bodies.get(&request.url) {
                Some((body, content_type)) => Ok(FetchResponse {
                    status: 200,
                    content_type: Some((*content_type).to_string()),
                    body: bytes::Bytes::from(body.clone().into_bytes()),
                }),
                None => Err(HttpFetchError::Send(format!(
                    "no fixture for {}",
                    request.url
                ))),
            }
        }
    }

    /// Parses anything containing an Atom `<feed` marker; everything else is
    /// a parse failure, mirroring how RSSHub HTML error pages behave.
    struct MarkerParser;

    impl FeedParser for MarkerParser {
        fn parse(&self, body: &[u8]) -> Result<ParsedFeed, FeedParseError> {
            if body.windows(5).any(|w| w == b"<feed") {
                Ok(ParsedFeed {
                    kind: ParsedFeedKind::Atom,
                    title: Some("Fixture Channel".into()),
                    description: None,
                    links: vec![],
                    icon_url: None,
                    logo_url: None,
                    entries: vec![],
                })
            } else {
                Err(FeedParseError::Parse("not a feed".into()))
            }
        }
    }

    struct RejectingOpml;

    impl OpmlParser for RejectingOpml {
        fn parse_feed_urls(&self, _: &str) -> Result<Vec<String>, OpmlParseError> {
            Err(OpmlParseError::Invalid("not used by this test".into()))
        }
    }

    struct RejectingOutbox;

    #[async_trait::async_trait]
    impl JobOutboxRepository for RejectingOutbox {
        async fn enqueue(
            &self,
            _: &str,
            _: serde_json::Value,
            _: Option<String>,
            _: DateTime<Utc>,
        ) -> Result<JobOutbox, AppError> {
            Err(support_gap())
        }
    }

    struct InstancesOnlyRepo {
        instances: Vec<FeedProviderInstance>,
        successes: Mutex<Vec<uuid::Uuid>>,
        failures: Mutex<Vec<uuid::Uuid>>,
        source: Option<FeedSource>,
        subscription: Option<FeedSubscription>,
    }

    impl InstancesOnlyRepo {
        fn new(instances: Vec<FeedProviderInstance>) -> Self {
            Self {
                instances,
                successes: Mutex::new(vec![]),
                failures: Mutex::new(vec![]),
                source: None,
                subscription: None,
            }
        }

        fn with_existing_subscription(mut self, subscription: FeedSubscription) -> Self {
            self.source = Some(subscription.source.clone());
            self.subscription = Some(subscription);
            self
        }
    }

    fn rsshub_instance(base_url: &str) -> FeedProviderInstance {
        FeedProviderInstance {
            id: uuid::Uuid::from_u128(0xFEED_0001),
            provider_type: "rsshub".into(),
            base_url: base_url.into(),
            priority: 10,
            enabled: true,
            last_success_at: None,
            last_failure_at: None,
            consecutive_failures: 0,
        }
    }

    #[async_trait::async_trait]
    impl FeedRepository for InstancesOnlyRepo {
        async fn find_source_by_id(&self, _: FeedSourceId) -> Result<Option<FeedSource>, AppError> {
            Err(support_gap())
        }
        async fn find_source_by_canonical_key(
            &self,
            canonical_key: &str,
        ) -> Result<Option<FeedSource>, AppError> {
            Ok(self
                .source
                .as_ref()
                .filter(|source| source.canonical_key == canonical_key)
                .cloned())
        }
        async fn create_source(&self, _: FeedSource) -> Result<FeedSource, AppError> {
            Err(support_gap())
        }
        async fn update_source_details(
            &self,
            _: FeedSourceId,
            _: SourceDetailsUpdate,
        ) -> Result<FeedSource, AppError> {
            Err(support_gap())
        }
        async fn bump_source_popularity(
            &self,
            _: FeedSourceId,
            _: i32,
        ) -> Result<FeedSource, AppError> {
            Err(support_gap())
        }
        async fn mark_source_poll_requested(
            &self,
            _: FeedSourceId,
            _: DateTime<Utc>,
        ) -> Result<FeedSource, AppError> {
            Err(support_gap())
        }
        async fn mark_source_poll_success(
            &self,
            _: FeedSourceId,
            _: PollOutcome,
            _: Option<DateTime<Utc>>,
        ) -> Result<FeedSource, AppError> {
            Err(support_gap())
        }
        async fn mark_source_poll_failure(
            &self,
            _: FeedSourceId,
            _: DateTime<Utc>,
            _: String,
            _: i32,
        ) -> Result<FeedSource, AppError> {
            Err(support_gap())
        }
        async fn clear_source_lease(&self, _: FeedSourceId) -> Result<(), AppError> {
            Err(support_gap())
        }
        async fn claim_due_sources(
            &self,
            _: DateTime<Utc>,
            _: &str,
            _: i64,
            _: Duration,
        ) -> Result<Vec<FeedSource>, AppError> {
            Err(support_gap())
        }
        async fn search_public_sources(
            &self,
            _: &str,
            _: FeedSearchSurface,
            _: u32,
        ) -> Result<Vec<FeedSource>, AppError> {
            Err(support_gap())
        }
        async fn find_subscription_by_id(
            &self,
            _: FeedSubscriptionId,
        ) -> Result<Option<FeedSubscription>, AppError> {
            Err(support_gap())
        }
        async fn find_subscription_by_user_and_source(
            &self,
            user_id: UserId,
            source_id: FeedSourceId,
        ) -> Result<Option<FeedSubscription>, AppError> {
            Ok(self
                .subscription
                .as_ref()
                .filter(|subscription| {
                    subscription.user_id == user_id && subscription.source_id == source_id
                })
                .cloned())
        }
        async fn create_subscription(
            &self,
            _: FeedSubscription,
        ) -> Result<FeedSubscription, AppError> {
            Err(support_gap())
        }
        async fn delete_subscription(
            &self,
            _: FeedSubscriptionId,
            _: UserId,
        ) -> Result<FeedSourceId, AppError> {
            Err(support_gap())
        }
        async fn delete_source_if_orphaned(&self, _: FeedSourceId) -> Result<(), AppError> {
            Err(support_gap())
        }
        async fn list_subscriptions_by_user(
            &self,
            _: UserId,
            _: Option<Cursor>,
            _: u32,
        ) -> Result<Page<FeedSubscription>, AppError> {
            Err(support_gap())
        }
        async fn list_active_subscriptions_for_source(
            &self,
            _: FeedSourceId,
        ) -> Result<Vec<ActiveSubscription>, AppError> {
            Err(support_gap())
        }
        async fn set_subscription_title_override(
            &self,
            _: FeedSubscriptionId,
            _: UserId,
            _: Option<String>,
        ) -> Result<FeedSubscription, AppError> {
            Err(support_gap())
        }
        async fn set_subscription_auto_save(
            &self,
            _: FeedSubscriptionId,
            _: UserId,
            _: bool,
            _: Option<Option<CollectionId>>,
        ) -> Result<FeedSubscription, AppError> {
            Err(support_gap())
        }
        async fn set_subscription_poll_interval(
            &self,
            _: FeedSubscriptionId,
            _: UserId,
            _: Option<i32>,
        ) -> Result<FeedSubscription, AppError> {
            Err(support_gap())
        }
        async fn set_subscription_status(
            &self,
            _: FeedSubscriptionId,
            _: UserId,
            _: FeedStatus,
        ) -> Result<FeedSubscription, AppError> {
            Err(support_gap())
        }
        async fn find_source_entry_by_source_guid(
            &self,
            _: FeedSourceId,
            _: &str,
        ) -> Result<Option<FeedSourceEntry>, AppError> {
            Err(support_gap())
        }
        async fn find_source_entry_by_id(
            &self,
            _: FeedSourceEntryId,
        ) -> Result<Option<FeedSourceEntry>, AppError> {
            Err(support_gap())
        }
        async fn create_source_entry(
            &self,
            _: FeedSourceEntry,
        ) -> Result<FeedSourceEntry, AppError> {
            Err(support_gap())
        }
        async fn create_or_adopt_polled_source_entry(
            &self,
            _: FeedSourceEntry,
        ) -> Result<FeedSourceEntry, AppError> {
            Err(support_gap())
        }
        async fn set_source_entry_canonical_url(
            &self,
            _: FeedSourceEntryId,
            _: &str,
        ) -> Result<(), AppError> {
            Err(support_gap())
        }
        async fn set_source_entry_language_if_missing(
            &self,
            _: FeedSourceEntryId,
            _: &str,
        ) -> Result<bool, AppError> {
            Err(support_gap())
        }
        async fn source_entries_missing_canonical_url_after(
            &self,
            _: uuid::Uuid,
            _: i64,
        ) -> Result<Vec<(FeedSourceEntryId, String)>, AppError> {
            Err(support_gap())
        }
        async fn list_provider_instances(
            &self,
            _: &str,
        ) -> Result<Vec<FeedProviderInstance>, AppError> {
            Err(support_gap())
        }
        async fn list_all_enabled_provider_instances(
            &self,
        ) -> Result<Vec<FeedProviderInstance>, AppError> {
            Ok(self.instances.clone())
        }
        async fn record_provider_instance_success(&self, id: uuid::Uuid) -> Result<(), AppError> {
            self.successes.lock().unwrap().push(id);
            Ok(())
        }
        async fn record_provider_instance_failure(&self, id: uuid::Uuid) -> Result<(), AppError> {
            self.failures.lock().unwrap().push(id);
            Ok(())
        }
    }

    fn service(fetcher: Arc<RecordingFetcher>, repo: Arc<InstancesOnlyRepo>) -> FeedService {
        FeedService::new(
            repo,
            Arc::new(RejectingOutbox),
            fetcher,
            Arc::new(MarkerParser),
            Arc::new(RejectingOpml),
        )
    }

    fn user() -> UserId {
        "usr_01890000-0000-7000-8000-000000000042".parse().unwrap()
    }

    fn existing_subscription() -> FeedSubscription {
        let now = Utc::now();
        let source = FeedSource {
            id: FeedSourceId::new(),
            canonical_key: "public:url:https://example.com/feed.xml".into(),
            source_url: SITE_FEED_URL.into(),
            poll_url: SITE_FEED_URL.into(),
            title: "Fixture Channel".into(),
            description: None,
            site_url: None,
            image_url: None,
            domain: Some("example.com".into()),
            feed_type: FeedType::Atom,
            visibility: FeedVisibility::Public,
            provider: None,
            is_resolvable: false,
            popularity: 1,
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
        FeedSubscription {
            id: FeedSubscriptionId::new(),
            user_id: user(),
            source_id: source.id,
            input_url: SITE_FEED_URL.into(),
            title_override: None,
            auto_save: false,
            auto_save_collection_id: None,
            poll_interval_override_minutes: None,
            status: FeedStatus::Active,
            created_at: now,
            updated_at: now,
            source,
        }
    }

    #[tokio::test]
    async fn discovered_feed_url_becomes_the_poll_url_and_public_canonical_key() {
        let fetcher = Arc::new(
            RecordingFetcher::default()
                .with(SITE_URL, SITE_HTML, "text/html")
                .with(SITE_FEED_URL, ATOM_BODY, "application/atom+xml"),
        );
        let repo = Arc::new(InstancesOnlyRepo::new(vec![]));

        let resolved = service(fetcher, repo)
            .resolve_source(user(), SITE_URL)
            .await
            .unwrap();

        assert_eq!(resolved.poll_url, SITE_FEED_URL);
        assert_eq!(
            resolved.canonical_key,
            "public:url:https://example.com/feed.xml"
        );
    }

    #[tokio::test]
    async fn direct_and_discovered_feed_urls_share_a_canonical_key() {
        let fetcher = Arc::new(
            RecordingFetcher::default()
                .with(SITE_URL, SITE_HTML, "text/html")
                .with(SITE_FEED_URL, ATOM_BODY, "application/atom+xml"),
        );
        let repo = Arc::new(InstancesOnlyRepo::new(vec![]));
        let service = service(fetcher, repo);

        let direct = service.resolve_source(user(), SITE_FEED_URL).await.unwrap();
        let discovered = service.resolve_source(user(), SITE_URL).await.unwrap();

        assert_eq!(discovered.canonical_key, direct.canonical_key);
    }

    #[tokio::test]
    async fn discovered_private_feed_url_uses_a_user_private_canonical_key() {
        let fetcher = Arc::new(
            RecordingFetcher::default()
                .with(SITE_URL, PRIVATE_SITE_HTML, "text/html")
                .with(PRIVATE_SITE_FEED_URL, ATOM_BODY, "application/atom+xml"),
        );
        let repo = Arc::new(InstancesOnlyRepo::new(vec![]));

        let resolved = service(fetcher, repo)
            .resolve_source(user(), SITE_URL)
            .await
            .unwrap();

        assert_eq!(resolved.poll_url, PRIVATE_SITE_FEED_URL);
        assert_eq!(resolved.visibility, FeedVisibility::Private);
        assert_eq!(
            resolved.canonical_key,
            "private:usr_01890000-0000-7000-8000-000000000042:https://example.com/feed.xml?token=secret"
        );
    }

    #[tokio::test]
    async fn subscribing_to_an_existing_user_source_returns_the_existing_subscription() {
        let fetcher = Arc::new(RecordingFetcher::default().with(
            SITE_FEED_URL,
            ATOM_BODY,
            "application/atom+xml",
        ));
        let existing = existing_subscription();
        let expected_id = existing.id;
        let repo = Arc::new(InstancesOnlyRepo::new(vec![]).with_existing_subscription(existing));

        let result = service(fetcher, repo)
            .subscribe(
                user(),
                SubscribeInput {
                    url: SITE_FEED_URL.into(),
                    title_override: None,
                    poll_interval_override_minutes: None,
                },
            )
            .await
            .unwrap();

        assert_eq!(result.subscription.id, expected_id);
        assert!(!result.is_new);
    }

    #[tokio::test]
    async fn handle_urls_resolve_through_the_native_atom_feed_without_rsshub() {
        let fetcher = Arc::new(
            RecordingFetcher::default()
                .with(
                    "https://www.youtube.com/@veritasium",
                    &handle_page_html(),
                    "text/html",
                )
                .with(NATIVE_ATOM_URL, ATOM_BODY, "application/atom+xml"),
        );
        let repo = Arc::new(InstancesOnlyRepo::new(vec![rsshub_instance(
            "https://rsshub.dead.example",
        )]));

        let resolved = service(fetcher.clone(), repo.clone())
            .resolve_source(user(), "https://www.youtube.com/@veritasium")
            .await
            .unwrap();

        assert_eq!(resolved.poll_url, NATIVE_ATOM_URL);
        assert_eq!(resolved.title, "Fixture Channel");
        assert!(
            fetcher
                .requested()
                .iter()
                .all(|url| !url.contains("rsshub")),
            "RSSHub must not be consulted when the native feed resolves, got {:?}",
            fetcher.requested()
        );
    }

    #[tokio::test]
    async fn channel_id_urls_go_straight_to_the_native_atom_feed() {
        let fetcher = Arc::new(RecordingFetcher::default().with(
            NATIVE_ATOM_URL,
            ATOM_BODY,
            "application/atom+xml",
        ));
        let repo = Arc::new(InstancesOnlyRepo::new(vec![]));

        let resolved = service(fetcher.clone(), repo.clone())
            .resolve_source(
                user(),
                &format!("https://www.youtube.com/channel/{CHANNEL_ID}"),
            )
            .await
            .unwrap();

        assert_eq!(resolved.poll_url, NATIVE_ATOM_URL);
        assert_eq!(
            fetcher.requested(),
            vec![NATIVE_ATOM_URL.to_string()],
            "a channel id needs no page fetch and no RSSHub"
        );
    }

    #[tokio::test]
    async fn failed_handle_resolution_falls_back_to_rsshub() {
        // No fixture for the handle page: resolution fails, RSSHub serves.
        let rsshub = rsshub_instance("https://rsshub.live.example");
        let rsshub_feed_url = "https://rsshub.live.example/youtube/user/@veritasium";
        let fetcher = Arc::new(RecordingFetcher::default().with(
            rsshub_feed_url,
            ATOM_BODY,
            "application/atom+xml",
        ));
        let repo = Arc::new(InstancesOnlyRepo::new(vec![rsshub.clone()]));

        let resolved = service(fetcher.clone(), repo.clone())
            .resolve_source(user(), "https://www.youtube.com/@veritasium")
            .await
            .unwrap();

        assert_eq!(resolved.poll_url, rsshub_feed_url);
        assert_eq!(
            repo.successes.lock().unwrap().as_slice(),
            &[rsshub.id],
            "the serving instance must record a success"
        );
    }
}
