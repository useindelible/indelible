#![allow(clippy::unwrap_used)]

use chrono::Utc;
use ind_application::repos::feed::FeedRepository;
use ind_domain::{
    FeedSource, FeedSourceEntry, FeedSourceEntryId, FeedSourceId, FeedType, FeedVisibility,
    PollOutcome,
};
use ind_persistence::repos::PgFeedRepository;
use ind_test_support::{FeedSourceFactory, TestDb};

/// Remote feeds can carry a NUL byte in any of their text fields (a mis-decoded UTF-16 title
/// is the usual source), and Postgres `text` rejects `0x00` outright, so an unsanitized bind
/// fails the whole statement with `invalid byte sequence for encoding "UTF8": 0x00`.
///
/// Sanitizing has to cover the read and comparison sides too, and for two distinct reasons:
/// a raw bind in a `WHERE` clause hits the very same encoding error, and a predicate that
/// sanitized *differently* from the write would silently stop matching the row it stored —
/// so the polled path would re-insert the same entry on every poll instead of adopting it.
#[tokio::test]
async fn source_entry_writes_and_lookups_strip_nul_from_text_fields() {
    let db = TestDb::new().await;
    let pool = db.pool().clone();
    let source = FeedSourceFactory.insert(&pool).await;
    let feeds = PgFeedRepository::new(pool.clone());

    // A guid that is a valid UUID once the NUL is stripped, so the polled path below can
    // still recognize it as an adoptable entry.
    let clean_guid = uuid::Uuid::now_v7().to_string();
    let raw_guid = clean_guid.replacen('-', "\u{0}-", 1);
    let title = "Break\u{0}ing News";
    let author = Some("Ada\u{0} Lovelace".to_string());
    let excerpt = Some("First\u{0} paragraph".to_string());
    let content_html = Some("<p>Body\u{0} text</p>".to_string());
    let url = Some("https://example.com/break\u{0}ing".to_string());

    let seeded = feeds
        .create_source_entry(FeedSourceEntry {
            id: FeedSourceEntryId::new(),
            source_id: source.id,
            guid: raw_guid.clone(),
            title: title.into(),
            url: url.clone(),
            canonical_url: None,
            author: author.clone(),
            excerpt: excerpt.clone(),
            content_html: content_html.clone(),
            language: None,
            lead_image_url: None,
            published_at: None,
            discovered_at: Utc::now(),
        })
        .await
        .unwrap();

    assert_eq!(seeded.guid, clean_guid);
    assert_eq!(seeded.title, "Breaking News");
    assert_eq!(seeded.author.as_deref(), Some("Ada Lovelace"));
    assert_eq!(seeded.excerpt.as_deref(), Some("First paragraph"));
    assert_eq!(seeded.content_html.as_deref(), Some("<p>Body text</p>"));
    assert_eq!(seeded.url.as_deref(), Some("https://example.com/breaking"));

    // The caller still holds the raw guid the feed gave it, and must find the row anyway.
    let found = feeds
        .find_source_entry_by_source_guid(source.id, &raw_guid)
        .await
        .unwrap()
        .expect("raw NUL-bearing guid should find the sanitized row");
    assert_eq!(found.id, seeded.id);

    // The polled path adopts the stored row rather than duplicating it, which only works if
    // its dedup predicate compares the same sanitized text the insert wrote.
    let polled = feeds
        .create_or_adopt_polled_source_entry(FeedSourceEntry {
            id: FeedSourceEntryId::new(),
            source_id: source.id,
            guid: format!("entry-content-{}", uuid::Uuid::now_v7().simple()),
            title: title.into(),
            url,
            canonical_url: Some("https://example.com/canon\u{0}ical".to_string()),
            author,
            excerpt,
            content_html,
            language: None,
            lead_image_url: Some("https://example.com/lead\u{0}.png".to_string()),
            published_at: None,
            discovered_at: Utc::now(),
        })
        .await
        .unwrap();

    assert_eq!(
        polled.id, seeded.id,
        "polled entry should adopt the stored twin"
    );
    assert_eq!(polled.title, "Breaking News");
    assert_eq!(polled.author.as_deref(), Some("Ada Lovelace"));
    assert_eq!(polled.content_html.as_deref(), Some("<p>Body text</p>"));
    assert_eq!(
        polled.canonical_url.as_deref(),
        Some("https://example.com/canonical")
    );
    assert_eq!(
        polled.lead_image_url.as_deref(),
        Some("https://example.com/lead.png")
    );

    // The canonicalization backfill writes the URL on its own, outside the entry write.
    feeds
        .set_source_entry_canonical_url(seeded.id, "https://example.com/back\u{0}fill")
        .await
        .unwrap();
    let backfilled = feeds
        .find_source_entry_by_id(seeded.id)
        .await
        .unwrap()
        .expect("entry should still exist");
    assert_eq!(
        backfilled.canonical_url.as_deref(),
        Some("https://example.com/backfill")
    );
}

/// The feed's own metadata is remote text too, and `canonical_key` doubles as the lookup key
/// for subscribe-time dedup, so it has to be sanitized on both sides like an entry guid.
#[tokio::test]
async fn source_writes_and_lookups_strip_nul_from_text_fields() {
    let db = TestDb::new().await;
    let pool = db.pool().clone();
    let feeds = PgFeedRepository::new(pool.clone());

    let poll_url = format!("https://{}.example.com/fe\u{0}ed.xml", uuid::Uuid::now_v7());
    let canonical_key = format!("public:url:{poll_url}");
    let timestamp = Utc::now();

    let created = feeds
        .create_source(FeedSource {
            id: FeedSourceId::new(),
            canonical_key: canonical_key.clone(),
            source_url: poll_url.clone(),
            poll_url: poll_url.clone(),
            title: "Daily\u{0} Dispatch".into(),
            description: Some("All the\u{0} news".to_string()),
            site_url: Some("https://example.com/si\u{0}te".to_string()),
            image_url: Some("https://example.com/lo\u{0}go.png".to_string()),
            domain: Some("exa\u{0}mple.com".to_string()),
            feed_type: FeedType::Rss,
            visibility: FeedVisibility::Public,
            provider: None,
            is_resolvable: false,
            popularity: 0,
            last_entry_added_at: None,
            last_polled_at: None,
            next_poll_at: None,
            last_etag: Some("W/\"eta\u{0}g\"".to_string()),
            last_modified: None,
            consecutive_failures: 0,
            last_error: None,
            lease_owner: None,
            lease_expires_at: None,
            created_at: timestamp,
            updated_at: timestamp,
        })
        .await
        .unwrap();

    assert_eq!(created.title, "Daily Dispatch");
    assert_eq!(created.description.as_deref(), Some("All the news"));
    assert_eq!(
        created.site_url.as_deref(),
        Some("https://example.com/site")
    );
    assert_eq!(
        created.image_url.as_deref(),
        Some("https://example.com/logo.png")
    );
    assert_eq!(created.domain.as_deref(), Some("example.com"));
    assert_eq!(created.last_etag.as_deref(), Some("W/\"etag\""));
    assert_eq!(created.canonical_key, canonical_key.replace('\u{0}', ""));

    // Resolving a feed URL hands the raw key back to the repository; it must still match.
    let found = feeds
        .find_source_by_canonical_key(&canonical_key)
        .await
        .unwrap()
        .expect("raw NUL-bearing canonical key should find the sanitized row");
    assert_eq!(found.id, created.id);

    // Every poll rewrites the HTTP validators from the response headers, long after create.
    let polled = feeds
        .mark_source_poll_success(
            created.id,
            PollOutcome {
                source_id: created.id,
                last_polled_at: Some(timestamp),
                next_poll_at: Some(timestamp),
                last_etag: Some("W/\"fre\u{0}sh\"".to_string()),
                last_modified: Some("Mon, 24 Aug 2026 00:00:00\u{0} GMT".to_string()),
                consecutive_failures: 0,
                last_error: None,
            },
            Some(timestamp),
        )
        .await
        .unwrap();

    assert_eq!(polled.last_etag.as_deref(), Some("W/\"fresh\""));
    assert_eq!(
        polled.last_modified.as_deref(),
        Some("Mon, 24 Aug 2026 00:00:00 GMT")
    );

    // A failure message quotes what the remote sent, so it carries remote bytes too.
    let failed = feeds
        .mark_source_poll_failure(
            created.id,
            timestamp,
            "parse error at \u{0} byte 12".to_string(),
            1,
        )
        .await
        .unwrap();

    assert_eq!(
        failed.last_error.as_deref(),
        Some("parse error at  byte 12")
    );
}
