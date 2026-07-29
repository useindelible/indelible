use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ind_test_support::spawn_app;
use reqwest::StatusCode;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::common::assert_json_response;

#[tokio::test]
async fn extension_pkce_crosses_authorize_exchange_refresh_status_and_revoke_boundaries() {
    let app = spawn_app().await;
    let web = app.create_web_session().await;
    let web_client = app.authed_client(&web);
    let verifier = "surgical-extension-verifier-with-enough-entropy-1234567890";
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let redirect_uri = format!("{}/extension/auth/callback", app.address);

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
