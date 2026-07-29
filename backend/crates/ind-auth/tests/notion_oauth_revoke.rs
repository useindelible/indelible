use base64::Engine;
use ind_auth::{IntegrationOAuthProviderAdapter, NotionOAuthAdapter};
use wiremock::matchers::{body_partial_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn adapter(server: &MockServer) -> NotionOAuthAdapter {
    NotionOAuthAdapter::new(
        "client-id".into(),
        "client-secret".into(),
        server.uri(),
        "http://localhost/callback".into(),
    )
}

fn basic_credentials() -> String {
    base64::engine::general_purpose::STANDARD.encode("client-id:client-secret")
}

#[tokio::test]
async fn revoke_posts_basic_auth_and_token_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/oauth/revoke"))
        .and(header(
            "authorization",
            format!("Basic {}", basic_credentials()).as_str(),
        ))
        .and(header("notion-version", ind_domain::NOTION_API_VERSION))
        .and(body_partial_json(serde_json::json!({ "token": "tok-1" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "request_id": "11111111-1111-1111-1111-111111111111"
        })))
        .expect(1)
        .mount(&server)
        .await;

    adapter(&server).revoke_token("tok-1").await.unwrap();
}

#[tokio::test]
async fn revoke_treats_invalid_grant_as_already_revoked() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/oauth/revoke"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "object": "error",
            "status": 400,
            "code": "invalid_grant",
            "message": "Invalid token."
        })))
        .expect(1)
        .mount(&server)
        .await;

    adapter(&server).revoke_token("tok-dead").await.unwrap();

    // The RFC 6749 OAuth error shape must keep working too.
    let rfc = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/oauth/revoke"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "error": "invalid_grant",
            "error_description": "Invalid token."
        })))
        .expect(1)
        .mount(&rfc)
        .await;
    adapter(&rfc).revoke_token("tok-dead").await.unwrap();
}

#[tokio::test]
async fn revoke_propagates_authentication_failures() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/oauth/revoke"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": "invalid_client"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let err = adapter(&server).revoke_token("tok-1").await.unwrap_err();
    assert!(err.to_string().contains("401"), "got: {err}");
}

#[tokio::test]
async fn revoke_propagates_server_errors_and_malformed_bad_requests() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/oauth/revoke"))
        .respond_with(ResponseTemplate::new(500).set_body_json(serde_json::json!({
            "error": "internal_server_error"
        })))
        .expect(1)
        .mount(&server)
        .await;
    let err = adapter(&server).revoke_token("tok-1").await.unwrap_err();
    assert!(err.to_string().contains("500"), "got: {err}");

    let malformed = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/oauth/revoke"))
        .respond_with(ResponseTemplate::new(400).set_body_string("not json"))
        .expect(1)
        .mount(&malformed)
        .await;
    let err = adapter(&malformed).revoke_token("tok-1").await.unwrap_err();
    assert!(err.to_string().contains("400"), "got: {err}");
}
