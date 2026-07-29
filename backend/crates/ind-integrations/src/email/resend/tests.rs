use super::*;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn provider() -> ResendProvider {
    ResendProvider::new(
        "whsec_dGVzdHNlY3JldGtleWF0bGVhc3QzMmJ5dGVzIQ==",
        "key".into(),
    )
    .unwrap()
}

#[test]
fn webhook_metadata_and_full_email_share_resends_header_contract() {
    assert!(ResendProvider::new("invalid", "key".into()).is_err());
    let provider = provider();
    assert_eq!(provider.provider(), EmailIngestProvider::Resend);
    let body = serde_json::to_vec(&serde_json::json!({
        "data": {
            "email_id": "email_1",
            "to": ["token@feed.useindelible.com"],
            "from": "Digest <digest@example.com>",
            "headers": [{"name": "List-ID", "value": "<digest.example>"}]
        }
    }))
    .unwrap();
    let metadata = provider.parse_webhook_metadata(&body).unwrap();
    assert_eq!(metadata.provider_email_id, "email_1");
    assert_eq!(metadata.list_id.as_deref(), Some("<digest.example>"));
    assert!(provider.parse_webhook_metadata(b"not-json").is_err());
    assert!(
        provider
            .parse_webhook_metadata(br#"{"data":{"email_id":"","to":[]}}"#)
            .is_err()
    );

    let nested = serde_json::json!({
        "unsubscribe": {"url": "https://example.com/unsub", "mail": "leave@example.com"},
        "unsubscribe-post": {"name": "List-Unsubscribe=One-Click"}
    });
    let inbound = build_inbound(
        ResendFullEmail {
            from: "\"Digest Team\" <digest@example.com>".into(),
            to: metadata.to_addresses,
            subject: "Issue 1".into(),
            html: Some("<p>Body</p>".into()),
            text: Some("Body".into()),
            headers: serde_json::json!({
                "Message-ID": "<message@example>",
                "List-ID": "<digest.example>",
                "list": nested.to_string()
            }),
        },
        metadata.provider_email_id,
    );
    assert_eq!(inbound.from_address, "digest@example.com");
    assert_eq!(inbound.from_display_name.as_deref(), Some("Digest Team"));
    assert_eq!(inbound.message_id.as_deref(), Some("<message@example>"));
    assert_eq!(
        inbound.list_unsubscribe.as_deref(),
        Some("<https://example.com/unsub>, <mailto:leave@example.com>")
    );
    assert_eq!(
        inbound.list_unsubscribe_post.as_deref(),
        Some("List-Unsubscribe=One-Click")
    );
    assert_eq!(
        parse_from_field("bare@example.com"),
        ("bare@example.com".into(), None)
    );
    assert_eq!(parse_from_field("reversed><"), ("reversed><".into(), None));
}

#[tokio::test]
async fn response_body_cap_accepts_boundary_and_rejects_declared_oversize() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/small"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![b'x'; 16]))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/large"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![b'x'; 17]))
        .mount(&server)
        .await;
    let small = reqwest::get(format!("{}/small", server.uri()))
        .await
        .unwrap();
    assert_eq!(read_capped_body(small, 16).await.unwrap().len(), 16);
    let large = reqwest::get(format!("{}/large", server.uri()))
        .await
        .unwrap();
    assert!(matches!(
        read_capped_body(large, 16).await,
        Err(EmailIngestError::ProviderApi(_))
    ));
}
