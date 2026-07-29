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
