use super::html::{
    BuildReaderHtmlInput, build_reader_html, format_duration_human, format_view_count,
};
use super::player::{
    CaptionTrack, PlayerResponse, ThumbnailEntry, fetch_player_response, pick_caption_track_url,
    pick_largest_thumbnail,
};
use super::transcript::{TranscriptSegment, parse_json3, parse_xml};
use ind_application::error::AppError;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[test]
fn caption_track_selection_prefers_english_then_falls_back() {
    let tracks = vec![
        CaptionTrack {
            base_url: Some("fr_url".into()),
            vss_id: Some(".fr".into()),
        },
        CaptionTrack {
            base_url: Some("en_url".into()),
            vss_id: Some(".en".into()),
        },
    ];
    assert_eq!(pick_caption_track_url(&tracks).as_deref(), Some("en_url"));
    let tracks = vec![CaptionTrack {
        base_url: Some("zz_url".into()),
        vss_id: Some(".zz".into()),
    }];
    assert_eq!(pick_caption_track_url(&tracks).as_deref(), Some("zz_url"));
}

#[test]
fn pick_largest_thumbnail_returns_widest() {
    let entries = vec![
        ThumbnailEntry {
            url: Some("small".into()),
            width: Some(120),
        },
        ThumbnailEntry {
            url: Some("large".into()),
            width: Some(1920),
        },
    ];
    assert_eq!(pick_largest_thumbnail(entries).as_deref(), Some("large"));
}

#[test]
fn player_response_classifies_only_explicit_terminal_playability_statuses() {
    for (status, terminal) in [
        ("ERROR", true),
        ("UNPLAYABLE", true),
        ("LOGIN_REQUIRED", true),
        ("OK", false),
        ("LIVE_STREAM_OFFLINE", false),
    ] {
        let response: PlayerResponse = serde_json::from_value(serde_json::json!({
            "playabilityStatus": {
                "status": status,
                "reason": "Provider diagnostic"
            }
        }))
        .unwrap();

        assert_eq!(response.is_terminally_unavailable(), terminal, "{status}");
        assert_eq!(
            response
                .playability_status
                .as_ref()
                .and_then(|value| value.reason.as_deref()),
            Some("Provider diagnostic"),
            "{status}"
        );
    }

    let response: PlayerResponse = serde_json::from_value(serde_json::json!({})).unwrap();
    assert!(!response.is_terminally_unavailable());
}

#[tokio::test]
async fn player_provider_failures_remain_retryable_external_service_errors() {
    let http = ind_egress::build_guarded_client(ind_egress::GuardedClientOptions::new(
        ind_egress::UrlRules::ingest(),
        ind_egress::EgressPolicy::permissive(),
    ))
    .unwrap();

    let status_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/youtubei/v1/player"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&status_server)
        .await;
    assert_youtube_external(
        fetch_player_response(&http, &status_server.uri(), "status-failure")
            .await
            .err()
            .unwrap(),
    );

    let json_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/youtubei/v1/player"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not-json"))
        .mount(&json_server)
        .await;
    assert_youtube_external(
        fetch_player_response(&http, &json_server.uri(), "json-failure")
            .await
            .err()
            .unwrap(),
    );

    let network_server = MockServer::start().await;
    let unavailable_base = network_server.uri();
    drop(network_server);
    assert_youtube_external(
        fetch_player_response(&http, &unavailable_base, "network-failure")
            .await
            .err()
            .unwrap(),
    );
}

fn assert_youtube_external(error: AppError) {
    assert!(matches!(
        error,
        AppError::ExternalService { ref service, .. } if service == "youtube"
    ));
}

#[test]
fn format_view_count_handles_thousands() {
    assert_eq!(format_view_count("1234567"), "1.2M views");
    assert_eq!(format_view_count("999"), "999 views");
    assert_eq!(format_view_count("2000"), "2K views");
}

#[test]
fn format_duration_human_handles_hours() {
    assert_eq!(format_duration_human(75), "1:15");
    assert_eq!(format_duration_human(3675), "1:01:15");
}

#[test]
fn parse_json3_extracts_segments() {
    let body = r#"{"events":[{"tStartMs":0,"dDurationMs":1000,"segs":[{"utf8":"Hello"}]},{"tStartMs":1000,"dDurationMs":1000,"segs":[{"utf8":"world."}]}]}"#;
    let segs = parse_json3(body).unwrap();
    assert_eq!(segs.len(), 2);
    assert_eq!(segs[0].text, "Hello");
    assert_eq!(segs[1].text, "world.");
}

#[test]
fn parse_xml_extracts_segments() {
    let body = r#"<transcript><text start="0" dur="1.5">Hello there</text><text start="2" dur="2">world.</text></transcript>"#;
    let segs = parse_xml(body);
    assert_eq!(segs.len(), 2);
    assert_eq!(segs[0].text, "Hello there");
    assert_eq!(segs[0].start_ms, 0);
    assert_eq!(segs[1].start_ms, 2000);
}

#[test]
fn build_reader_html_includes_iframe_and_stats() {
    let html = build_reader_html(BuildReaderHtmlInput {
        video_id: "abc",
        description: "hello",
        channel_name: "Sample Ch",
        view_count: Some("1000"),
        duration_seconds: Some(120),
        segments: &[],
    });
    assert!(html.contains("https://www.youtube.com/embed/abc"));
    assert!(html.contains("1K views"));
    assert!(html.contains("2:00"));
    assert!(html.contains("Sample Ch"));
    assert!(!html.contains("yt-transcript"));
    let segs = vec![TranscriptSegment {
        start_ms: 0,
        end_ms: Some(1000),
        text: "Hello there.".into(),
        new_speaker: false,
    }];
    let html = build_reader_html(BuildReaderHtmlInput {
        video_id: "abc",
        description: "",
        channel_name: "Ch",
        view_count: None,
        duration_seconds: None,
        segments: &segs,
    });
    assert!(html.contains("yt-transcript"));
    assert!(html.contains("Hello there."));
    assert!(html.contains("data-t=\"0:00\""));
}
