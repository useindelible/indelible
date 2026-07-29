#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use chrono::Utc;
use ind_application::repos::document::DocumentRepository;
use ind_application::repos::feed::FeedRepository;
use ind_application::repos::feed_delivery::FeedDeliveryRepository;
use ind_domain::{
    CanonicalizationConfig, DocumentId, DocumentType, FeedDeliveryState, FeedPollJob, FeedSource,
    FeedSourceId, FeedType, FeedVisibility, NewUrlDocument, canonicalize_url,
};
use ind_persistence::repos::{PgDocumentRepository, PgFeedDeliveryRepository, PgFeedRepository};
use ind_test_support::{FeedSubscriptionFactory, TestDb, TestWorkerHarness, UserFactory};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

mod common;
use common::build_worker_ctx;

const ENTRY_URL: &str = "https://example.com/posts/surgical-poll";
const LAST_MODIFIED: &str = "Wed, 15 Jul 2026 08:00:00 GMT";
const RSS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Indelible Engineering</title>
    <link>https://example.com/</link>
    <description>Real worker boundary</description>
    <language>en-US</language>
    <item>
      <guid>surgical-poll-1</guid>
      <title>Surgical Poll</title>
      <link>https://example.com/posts/surgical-poll?utm_source=rss</link>
      <description><![CDATA[<p>Durable feed content.</p><img src="https://example.com/lead.jpg">]]></description>
      <enclosure url="https://example.com/episode.mp3" type="audio/mpeg" length="42" />
    </item>
  </channel>
</rss>"#;

#[tokio::test]
async fn poll_fetches_persists_delivers_autosaves_and_repolls_idempotently() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/feed.xml"))
        .respond_with(
            ResponseTemplate::new(200)
                .append_header("content-type", "application/rss+xml")
                .append_header("etag", "\"surgical-v1\"")
                .append_header("last-modified", LAST_MODIFIED)
                .set_body_string(RSS),
        )
        .expect(2)
        .mount(&server)
        .await;

    let db = TestDb::new().await;
    let ctx = build_worker_ctx(&db).await;
    let feed_jobs = ctx.feed_jobs();
    let feeds = PgFeedRepository::new(db.pool().clone());
    let deliveries = PgFeedDeliveryRepository::new(db.pool().clone());
    let linked_user = UserFactory::new().insert(db.pool()).await;
    let autosave_user = UserFactory::new().insert(db.pool()).await;
    let source = source(format!("{}/feed.xml", server.uri()));
    feeds.create_source(source.clone()).await.unwrap();
    let linked_subscription = FeedSubscriptionFactory::new(linked_user.id)
        .with_source(source.clone())
        .insert(db.pool())
        .await;
    let autosave_subscription = FeedSubscriptionFactory::new(autosave_user.id)
        .with_source(source.clone())
        .insert(db.pool())
        .await;
    feeds
        .set_subscription_auto_save(autosave_subscription.id, autosave_user.id, true, None)
        .await
        .unwrap();

    let canonical = canonicalize_url(ENTRY_URL, &CanonicalizationConfig::default())
        .unwrap()
        .into_string();
    let existing = PgDocumentRepository::new(db.pool().clone())
        .upsert_url_backed(NewUrlDocument {
            id: DocumentId::new(),
            user_id: linked_user.id,
            document_type: DocumentType::Article,
            canonical_url: canonical,
            original_url: Some(ENTRY_URL.into()),
            content_hash: None,
            title: "Already saved".into(),
            author: None,
            excerpt: None,
            published_at: None,
            language: Some("ENG_us".into()),
            domain: None,
            lead_image_url: None,
            thumbnail_url: None,
        })
        .await
        .unwrap();
    assert_eq!(existing.language.as_deref(), Some("en-us"));

    for _ in 0..2 {
        ind_worker::jobs::feed::handle_feed_poll(
            &feed_jobs,
            FeedPollJob {
                source_id: source.id,
            },
        )
        .await
        .unwrap();
    }

    let updated = feeds.find_source_by_id(source.id).await.unwrap().unwrap();
    assert_eq!(updated.title, "Indelible Engineering");
    assert_eq!(updated.site_url.as_deref(), Some("https://example.com/"));
    assert_eq!(updated.feed_type, FeedType::Rss);
    assert_eq!(updated.last_etag.as_deref(), Some("\"surgical-v1\""));
    assert_eq!(updated.last_modified.as_deref(), Some(LAST_MODIFIED));
    assert_eq!(updated.consecutive_failures, 0);
    assert!(updated.last_polled_at.is_some());

    let entry = feeds
        .find_source_entry_by_source_guid(source.id, "surgical-poll-1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(entry.title, "Surgical Poll");
    assert_eq!(entry.language.as_deref(), Some("en-us"));
    assert_eq!(entry.excerpt.as_deref(), Some("Durable feed content."));
    assert_eq!(
        entry.lead_image_url.as_deref(),
        Some("https://example.com/lead.jpg")
    );

    let linked = deliveries
        .list_deliveries(
            linked_user.id,
            FeedDeliveryState::Unseen,
            Some(linked_subscription.id),
            None,
            10,
        )
        .await
        .unwrap();
    assert_eq!(linked.items.len(), 1);
    assert_eq!(linked.items[0].delivery.document_id, Some(existing.id));
    let autosaved = deliveries
        .list_deliveries(
            autosave_user.id,
            FeedDeliveryState::Unseen,
            Some(autosave_subscription.id),
            None,
            10,
        )
        .await
        .unwrap();
    assert_eq!(autosaved.items.len(), 1);
    assert_eq!(autosaved.items[0].delivery.document_id, None);
    assert_eq!(
        TestWorkerHarness::new(db.pool().clone())
            .pending_job_count_by_type("feed.autosave")
            .await
            .unwrap(),
        1,
        "re-poll must not enqueue the autosave twice"
    );
}

#[tokio::test]
async fn poll_records_conditional_success_failure_and_orphan_cleanup() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/not-modified"))
        .respond_with(ResponseTemplate::new(304).append_header("etag", "\"cached\""))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/failure"))
        .respond_with(ResponseTemplate::new(503))
        .expect(1)
        .mount(&server)
        .await;

    let db = TestDb::new().await;
    let ctx = build_worker_ctx(&db).await;
    let feed_jobs = ctx.feed_jobs();
    let feeds = PgFeedRepository::new(db.pool().clone());
    let user = UserFactory::new().insert(db.pool()).await;

    let mut cached = source(format!("{}/not-modified", server.uri()));
    cached.last_etag = Some("\"cached\"".into());
    cached.last_modified = Some(LAST_MODIFIED.into());
    cached.consecutive_failures = 4;
    cached.last_error = Some("old failure".into());
    feeds.create_source(cached.clone()).await.unwrap();
    FeedSubscriptionFactory::new(user.id)
        .with_source(cached.clone())
        .insert(db.pool())
        .await;
    ind_worker::jobs::feed::handle_feed_poll(
        &feed_jobs,
        FeedPollJob {
            source_id: cached.id,
        },
    )
    .await
    .unwrap();
    let cached = feeds.find_source_by_id(cached.id).await.unwrap().unwrap();
    assert_eq!(cached.consecutive_failures, 0);
    assert_eq!(cached.last_error, None);
    let requests = server.received_requests().await.unwrap();
    let conditional = requests
        .iter()
        .find(|request| request.url.path() == "/not-modified")
        .unwrap();
    assert_eq!(conditional.headers["if-none-match"], "\"cached\"");
    assert_eq!(conditional.headers["if-modified-since"], LAST_MODIFIED);

    let failing = source(format!("{}/failure", server.uri()));
    feeds.create_source(failing.clone()).await.unwrap();
    FeedSubscriptionFactory::new(user.id)
        .with_source(failing.clone())
        .insert(db.pool())
        .await;
    ind_worker::jobs::feed::handle_feed_poll(
        &feed_jobs,
        FeedPollJob {
            source_id: failing.id,
        },
    )
    .await
    .unwrap();
    let failing = feeds.find_source_by_id(failing.id).await.unwrap().unwrap();
    assert_eq!(failing.consecutive_failures, 1);
    assert!(failing.last_error.as_deref().unwrap().contains("HTTP 503"));

    let orphan = source("https://example.com/orphan.xml".into());
    feeds.create_source(orphan.clone()).await.unwrap();
    ind_worker::jobs::feed::handle_feed_poll(
        &feed_jobs,
        FeedPollJob {
            source_id: orphan.id,
        },
    )
    .await
    .unwrap();
    assert!(feeds.find_source_by_id(orphan.id).await.unwrap().is_none());
}

fn source(poll_url: String) -> FeedSource {
    let now = Utc::now();
    FeedSource {
        id: FeedSourceId::new(),
        canonical_key: format!("rss:{}", uuid::Uuid::now_v7()),
        source_url: poll_url.clone(),
        poll_url,
        title: "Before poll".into(),
        description: None,
        site_url: None,
        image_url: None,
        domain: None,
        feed_type: FeedType::Rss,
        visibility: FeedVisibility::Public,
        provider: None,
        is_resolvable: true,
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
    }
}
