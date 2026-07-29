use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ind_test_support::{TestAppOptions, spawn_app, spawn_app_with_options};
use reqwest::StatusCode;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::common::assert_json_response;

const FIREFOX_REDIRECT_URI: &str =
    "https://38bd18db5de5caccb6ab6c1271fec03ec1662d5c.extensions.allizom.org/indelible";

#[tokio::test]
async fn extension_start_validates_callback_and_redirects_once_to_spa_consent() {
    let app = spawn_app().await;
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let challenge = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQ";

    let mut start_url =
        url::Url::parse(&format!("{}/api/v1/auth/extension/start", app.address)).unwrap();
    start_url
        .query_pairs_mut()
        .append_pair("code_challenge", challenge)
        .append_pair("state", "state with symbols & spaces")
        .append_pair("redirect_uri", FIREFOX_REDIRECT_URI);
    let response = client.get(start_url).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
    let location = response.headers().get(reqwest::header::LOCATION).unwrap();
    let redirect = url::Url::parse(location.to_str().unwrap()).unwrap();
    assert_eq!(redirect.origin().ascii_serialization(), app.address);
    assert_eq!(redirect.path(), "/extension/auth");
    assert_eq!(
        redirect.query_pairs().collect::<Vec<_>>(),
        [
            ("code_challenge".into(), challenge.into()),
            ("state".into(), "state with symbols & spaces".into()),
            ("redirect_uri".into(), FIREFOX_REDIRECT_URI.into()),
        ]
    );

    let mut attacker_url =
        url::Url::parse(&format!("{}/api/v1/auth/extension/start", app.address)).unwrap();
    attacker_url
        .query_pairs_mut()
        .append_pair("code_challenge", challenge)
        .append_pair("state", "state")
        .append_pair("redirect_uri", "https://attacker.example/indelible");
    let rejected = client.get(attacker_url).send().await.unwrap();
    assert_eq!(rejected.status(), StatusCode::BAD_REQUEST);

    assert_eq!(
        client
            .get(format!("{}/api/v1/auth/extension/start", app.address))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn extension_start_supports_a_split_frontend_origin() {
    let app = spawn_app_with_options(TestAppOptions {
        frontend_url: Some("https://app.example.com/base-path?stale=true#fragment".to_string()),
        ..TestAppOptions::default()
    })
    .await;
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let mut start_url =
        url::Url::parse(&format!("{}/api/v1/auth/extension/start", app.address)).unwrap();
    start_url
        .query_pairs_mut()
        .append_pair(
            "code_challenge",
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQ",
        )
        .append_pair("state", "split-origin")
        .append_pair("redirect_uri", FIREFOX_REDIRECT_URI);
    let response = client.get(start_url).send().await.unwrap();

    assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
    let redirect = url::Url::parse(
        response
            .headers()
            .get(reqwest::header::LOCATION)
            .unwrap()
            .to_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        redirect.origin().ascii_serialization(),
        "https://app.example.com"
    );
    assert_eq!(redirect.path(), "/extension/auth");
    assert_eq!(redirect.fragment(), None);
    assert_eq!(redirect.query_pairs().count(), 3);
}

#[tokio::test]
async fn extension_pkce_crosses_authorize_exchange_refresh_status_and_revoke_boundaries() {
    let app = spawn_app().await;
    let web = app.create_web_session().await;
    let web_client = app.authed_client(&web);
    let verifier = "surgical-extension-verifier-with-enough-entropy-1234567890";
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let redirect_uri = FIREFOX_REDIRECT_URI.to_string();

    assert_eq!(
        web_client
            .post_json(
                "/api/v1/auth/extension/authorize",
                &json!({
                    "code_challenge": challenge,
                    "code_challenge_method": "S256",
                    "redirect_uri": "https://attacker.example/callback",
                    "state": "rejected"
                }),
            )
            .await
            .status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let authorized = assert_json_response(
        web_client
            .post_json(
                "/api/v1/auth/extension/authorize",
                &json!({
                    "code_challenge": challenge,
                    "code_challenge_method": "S256",
                    "redirect_uri": redirect_uri,
                    "state": "round-trip-state"
                }),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(authorized["state"], "round-trip-state");
    let code = authorized["code"].as_str().unwrap();

    let exchanged = assert_json_response(
        app.post_json_anon(
            "/api/v1/auth/extension/token",
            &json!({
                "code": code,
                "code_verifier": verifier,
                "redirect_uri": redirect_uri
            }),
        )
        .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(exchanged["token_type"], "Bearer");
    let access = exchanged["access_token"].as_str().unwrap();
    let refresh = exchanged["refresh_token"].as_str().unwrap();
    let status: Value = app
        .client()
        .get(format!("{}/api/v1/extension/status", app.address))
        .bearer_auth(access)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(status["authenticated"], true);
    assert_eq!(status["user"]["id"], web.user.id.to_string());
    assert_eq!(
        app.post_json_anon(
            "/api/v1/auth/extension/token",
            &json!({
                "code": code,
                "code_verifier": verifier,
                "redirect_uri": redirect_uri
            }),
        )
        .await
        .status(),
        StatusCode::BAD_REQUEST
    );

    let rotated = assert_json_response(
        app.post_json_anon(
            "/api/v1/auth/extension/refresh",
            &json!({"refresh_token": refresh}),
        )
        .await,
        StatusCode::OK,
    )
    .await;
    let rotated_refresh = rotated["refresh_token"].as_str().unwrap();
    assert_ne!(rotated_refresh, refresh);
    assert_eq!(
        app.post_json_anon(
            "/api/v1/auth/extension/revoke",
            &json!({"refresh_token": rotated_refresh}),
        )
        .await
        .status(),
        StatusCode::NO_CONTENT
    );
    let active_refresh_tokens: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM refresh_tokens WHERE user_id = $1 AND revoked_at IS NULL",
    )
    .bind(web.user.id.into_uuid())
    .fetch_one(app.pool())
    .await
    .unwrap();
    assert_eq!(active_refresh_tokens, 0);
}
