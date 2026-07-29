use std::collections::HashMap;

use super::*;

#[test]
fn ingest_address_routing_and_validation_table() {
    let feed = "feed.useindelible.com";
    let library = "library.useindelible.com";
    for (address, expected) in [
        (
            "x7k2m9p0@feed.useindelible.com",
            Some(("x7k2m9p0", EmailDestination::Feed)),
        ),
        (
            "SamA@library.useindelible.com",
            Some(("sama", EmailDestination::Library)),
        ),
        (
            "abc12345-feed@shared.resend.app",
            Some(("abc12345", EmailDestination::Feed)),
        ),
        (
            "abc12345-lib@feed.useindelible.com",
            Some(("abc12345-lib", EmailDestination::Feed)),
        ),
        (
            "abc12345-feed@library.useindelible.com",
            Some(("abc12345-feed", EmailDestination::Library)),
        ),
        ("user@unknown.com", None),
        ("abc12345-archive@shared.resend.app", None),
        ("ab@feed.useindelible.com", None),
        ("admin@feed.useindelible.com", None),
        ("@feed.useindelible.com", None),
    ] {
        let actual = parse_ingest_address(address, feed, library);
        assert_eq!(
            actual
                .as_ref()
                .map(|value| (value.token.as_str(), value.destination)),
            expected,
            "{address}"
        );
    }
}

#[test]
fn shared_domain_suffix_routes_to_explicit_destination() {
    let shared = "shared.resend.app";
    for (address, expected) in [
        (
            "abc12345-feed@shared.resend.app",
            Some(("abc12345", EmailDestination::Feed)),
        ),
        (
            "abc12345-lib@shared.resend.app",
            Some(("abc12345", EmailDestination::Library)),
        ),
        ("ab-lib@shared.resend.app", None),
    ] {
        let actual = parse_ingest_address(address, shared, shared);
        assert_eq!(
            actual
                .as_ref()
                .map(|value| (value.token.as_str(), value.destination)),
            expected,
            "{address}"
        );
    }
}

#[test]
fn formatted_addresses_round_trip_through_shared_and_distinct_domains() {
    for (feed_domain, library_domain, destination, expected) in [
        (
            "feed.example",
            "library.example",
            EmailDestination::Feed,
            "abc12345@feed.example",
        ),
        (
            "feed.example",
            "library.example",
            EmailDestination::Library,
            "abc12345@library.example",
        ),
        (
            "shared.example",
            "shared.example",
            EmailDestination::Feed,
            "abc12345-feed@shared.example",
        ),
        (
            "shared.example",
            "shared.example",
            EmailDestination::Library,
            "abc12345-lib@shared.example",
        ),
    ] {
        let address = format_ingest_address(
            "abc12345",
            destination,
            Some(feed_domain),
            Some(library_domain),
        )
        .unwrap();
        assert_eq!(address, expected);
        let parsed = parse_ingest_address(&address, feed_domain, library_domain).unwrap();
        assert_eq!(parsed.token, "abc12345");
        assert_eq!(parsed.destination, destination);
    }
}

#[test]
fn content_mode_and_primary_url_tables() {
    let long = "a".repeat(501);
    for (text, html, expected) in [
        (Some(long.as_str()), None, ContentMode::ModeA),
        (Some("Check this article"), None, ContentMode::ModeB),
        (None, None, ContentMode::ModeB),
    ] {
        assert_eq!(detect_content_mode(text, html), expected);
    }
    for (html, text, expected) in [
        (
            Some(
                r#"<a href="https://example.com/unsubscribe">x</a><a href="https://example.com/article">read</a>"#,
            ),
            None,
            Some("https://example.com/article"),
        ),
        (
            None,
            Some("read https://example.com/post now"),
            Some("https://example.com/post"),
        ),
        (Some("<p>none</p>"), Some("none"), None),
    ] {
        assert_eq!(extract_primary_url(html, text).as_deref(), expected);
    }
}

#[test]
fn sender_canonicalization_table() {
    for (raw, expected) in [
        ("Alice@Example.COM", "alice@example.com"),
        ("alice+news@gmail.com", "alice@gmail.com"),
        ("a+b+c@example.com", "a@example.com"),
        (" INVALID ", "invalid"),
    ] {
        assert_eq!(canonicalize_address(raw), expected, "{raw}");
    }
}

#[test]
fn forwarded_header_precedence_and_spoof_boundary() {
    let cases = [
        (
            vec![
                ("X-Original-From", "Ben Thompson <ben+news@Stratechery.com>"),
                ("Reply-To", "other@example.com"),
            ],
            "forwarder@gmail.com",
            "ben@stratechery.com",
            true,
        ),
        (
            vec![("Reply-To", "support@brand.com")],
            "noreply@brand.com",
            "noreply@brand.com",
            false,
        ),
        (
            vec![
                ("Reply-To", "noreply@site.com"),
                ("X-Forwarded-For", "user@gmail.com"),
            ],
            "forwarder@gmail.com",
            "noreply@site.com",
            true,
        ),
    ];
    for (pairs, envelope, expected, forwarded) in cases {
        let headers = pairs
            .into_iter()
            .map(|(key, value)| (key.into(), value.into()))
            .collect::<HashMap<_, _>>();
        let parsed = parse_forwarded_headers(&headers, envelope);
        assert_eq!(
            (parsed.original_address.as_str(), parsed.is_forwarded),
            (expected, forwarded)
        );
    }
}
