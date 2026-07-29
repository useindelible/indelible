use ind_test_support::{TestApp, TestAppOptions, spawn_app_with_options};
use reqwest::{Client, StatusCode, redirect::Policy};
use serde_json::json;
use url::Url;
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::common::assert_json_response;

async fn start_flow(client: &Client, app: &TestApp, path: &str) -> (Url, String) {
    let response = client
        .get(format!("{}{}", app.address, path))
        .send()
        .await
        .unwrap();
    if response.status() != StatusCode::TEMPORARY_REDIRECT {
        panic!("OIDC start failed: {}", response.text().await.unwrap());
    }
    let location = Url::parse(
        response.headers()[reqwest::header::LOCATION]
            .to_str()
            .unwrap(),
    )
    .unwrap();
    let state = location
        .query_pairs()
        .find_map(|(key, value)| (key == "state").then(|| value.into_owned()))
        .unwrap();
    (location, state)
}

#[tokio::test]
async fn oidc_web_and_native_flows_persist_state_validate_callbacks_and_project_errors() {
    let provider = MockServer::start().await;
    let issuer = provider.uri();
    Mock::given(method("GET"))
        .and(path_regex(r"^/.well-known/openid-configuration/?$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "issuer": issuer,
            "authorization_endpoint": format!("{}/authorize", provider.uri()),
            "token_endpoint": format!("{}/token", provider.uri()),
            "jwks_uri": format!("{}/jwks", provider.uri()),
            "response_types_supported": ["code"],
            "subject_types_supported": ["public"],
            "id_token_signing_alg_values_supported": ["RS256"],
            "scopes_supported": ["openid", "email", "profile"]
        })))
        .expect(3)
        .mount(&provider)
        .await;
    Mock::given(method("GET"))
        .and(path("/jwks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"keys": []})))
        .expect(3)
        .mount(&provider)
        .await;

    let app = spawn_app_with_options(TestAppOptions {
        oidc_issuer_url: Some(issuer.clone()),
        ..TestAppOptions::default()
    })
    .await;
    let no_redirect = Client::builder().redirect(Policy::none()).build().unwrap();
    let providers =
        assert_json_response(app.get("/api/v1/auth/providers").await, StatusCode::OK).await;
    assert_eq!(providers["providers"][0]["name"], "Test SSO");

    let response = app.get("/api/v1/auth/oauth/github/start").await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let response = app
        .get("/api/v1/auth/oauth/oidc/native/start?platform=desktop&code_challenge=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA&code_challenge_method=S256&app_state=native_state_1234")
        .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let response = app
        .get("/api/v1/auth/oauth/oidc/native/start?platform=ios&code_challenge=short&code_challenge_method=plain&app_state=bad")
        .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let (authorization, provider_error_state) =
        start_flow(&no_redirect, &app, "/api/v1/auth/oauth/oidc/start").await;
    assert_eq!(authorization.origin().ascii_serialization(), issuer);
    assert!(authorization.query_pairs().any(|(key, _)| key == "nonce"));
    assert!(
        authorization
            .query_pairs()
            .any(|(key, value)| key == "code_challenge_method" && value == "S256")
    );
    let response = no_redirect
        .get(format!(
            "{}/api/v1/auth/oauth/oidc/callback?state={provider_error_state}&error=access_denied&error_description=user_cancelled",
            app.address
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let (_, issuer_state) = start_flow(&no_redirect, &app, "/api/v1/auth/oauth/oidc/start").await;
    let response = no_redirect
        .get(format!(
            "{}/api/v1/auth/oauth/oidc/callback?state={issuer_state}&code=unused&iss=https%3A%2F%2Fwrong.example",
            app.address
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let (_, native_state) = start_flow(
        &no_redirect,
        &app,
        "/api/v1/auth/oauth/oidc/native/start?platform=ios&code_challenge=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA&code_challenge_method=S256&app_state=native_state_1234567890",
    )
    .await;
    let response = no_redirect
        .get(format!(
            "{}/api/v1/auth/oauth/oidc/callback?state={native_state}&error=access_denied&error_description=user_cancelled",
            app.address
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
    let native_redirect = Url::parse(
        response.headers()[reqwest::header::LOCATION]
            .to_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(native_redirect.scheme(), "com.useindelible.app");
    let params = native_redirect.query_pairs().collect::<Vec<_>>();
    assert!(
        params
            .iter()
            .any(|(key, value)| key == "error_code" && value == "access_denied")
    );
    assert!(
        params
            .iter()
            .any(|(key, value)| key == "state" && value == "native_state_1234567890")
    );

    let invalid_grant = no_redirect
        .post(format!("{}/api/v1/auth/oauth/native/token", app.address))
        .header("x-client-type", "ios")
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body("grant_type=refresh_token&code=invalid&code_verifier=invalid&redirect_uri=com.useindelible.app%3A%2Foauth%2Fcallback")
        .send()
        .await
        .unwrap();
    assert_eq!(invalid_grant.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        invalid_grant.json::<serde_json::Value>().await.unwrap()["error"],
        "invalid_grant"
    );
}
