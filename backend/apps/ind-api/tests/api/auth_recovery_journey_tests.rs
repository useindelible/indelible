use chrono::{Duration, Utc};
use ind_domain::UserId;
use ind_test_support::TestApp;
use reqwest::StatusCode;
use serde_json::json;

use super::common::{assert_json_response as response, assert_status};

#[tokio::test]
async fn verification_and_password_recovery_tokens_are_scoped_expiring_and_one_time() {
    let app = TestApp::new().await;
    let registered = response(
        app.post_json_anon(
            "/api/v1/auth/register",
            &json!({
                "email": "recovery@example.com",
                "password": "InitialP@ss123!",
                "display_name": "Recovery Boundary"
            }),
        )
        .await,
        StatusCode::CREATED,
    )
    .await;
    let user_id: UserId = registered["id"].as_str().unwrap().parse().unwrap();
    let access = registered["access_token"].as_str().unwrap();
    let verify_token = "known-verification-token";
    // Registration marks the account verified because no mail transport exists.
    // The token endpoints still have to work for any flow that does produce an
    // unverified account, so arrange that state explicitly.
    sqlx::query("UPDATE users SET email_verified = false WHERE id = $1")
        .bind(user_id.into_uuid())
        .execute(app.pool())
        .await
        .unwrap();
    sqlx::query("DELETE FROM email_verification_tokens WHERE user_id = $1")
        .bind(user_id.into_uuid())
        .execute(app.pool())
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO email_verification_tokens (id, user_id, token_hash, expires_at, created_at) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(uuid::Uuid::now_v7())
    .bind(user_id.into_uuid())
    .bind(ind_auth::hash_token(verify_token))
    .bind(Utc::now() + Duration::hours(1))
    .bind(Utc::now())
    .execute(app.pool())
    .await
    .unwrap();

    let resend = || {
        app.client()
            .post(format!("{}/api/v1/auth/email/resend", app.address))
            .bearer_auth(access)
            .json(&json!({}))
            .send()
    };
    assert_eq!(
        resend().await.unwrap().status(),
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_status(
        app.post_json_anon("/api/v1/auth/email/verify", &json!({"token": "wrong"}))
            .await,
        StatusCode::BAD_REQUEST,
    )
    .await;
    let verified = response(
        app.post_json_anon("/api/v1/auth/email/verify", &json!({"token": verify_token}))
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(verified["email_verified"], true);
    assert_eq!(resend().await.unwrap().status(), StatusCode::OK);
    assert_status(
        app.post_json_anon("/api/v1/auth/email/verify", &json!({"token": verify_token}))
            .await,
        StatusCode::BAD_REQUEST,
    )
    .await;

    for email in ["missing@example.com", "recovery@example.com"] {
        let forgot = response(
            app.post_json_anon("/api/v1/auth/password/forgot", &json!({"email": email}))
                .await,
            StatusCode::OK,
        )
        .await;
        assert!(forgot["message"].as_str().unwrap().contains("reset link"));
    }
    let reset_token = "known-password-reset-token";
    sqlx::query("DELETE FROM password_reset_tokens WHERE user_id = $1")
        .bind(user_id.into_uuid())
        .execute(app.pool())
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO password_reset_tokens \
         (id, user_id, token_hash, expires_at, used_at, created_at) \
         VALUES ($1, $2, $3, $4, NULL, $5)",
    )
    .bind(uuid::Uuid::now_v7())
    .bind(user_id.into_uuid())
    .bind(ind_auth::hash_token(reset_token))
    .bind(Utc::now() + Duration::hours(1))
    .bind(Utc::now())
    .execute(app.pool())
    .await
    .unwrap();
    assert_status(
        app.post_json_anon(
            "/api/v1/auth/password/reset",
            &json!({"token": "wrong", "new_password": "ResetP@ss123!"}),
        )
        .await,
        StatusCode::BAD_REQUEST,
    )
    .await;
    let reset = response(
        app.post_json_anon(
            "/api/v1/auth/password/reset",
            &json!({"token": reset_token, "new_password": "ResetP@ss123!"}),
        )
        .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(reset["email"], "recovery@example.com");
    assert_status(
        app.post_json_anon(
            "/api/v1/auth/password/reset",
            &json!({"token": reset_token, "new_password": "OtherP@ss123!"}),
        )
        .await,
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(
        app.post_json_anon(
            "/api/v1/auth/login",
            &json!({"email": "recovery@example.com", "password": "ResetP@ss123!"}),
        )
        .await
        .status(),
        StatusCode::OK
    );
}

/// Changing an email address clears verification and revokes every session, then
/// depends on a verification link to restore access. With no outbound mail
/// transport that link can never arrive, so the endpoint must refuse rather than
/// strand the account. The refusal has to happen before any mutation.
#[tokio::test]
async fn changing_email_is_refused_while_no_mail_transport_can_deliver_the_link() {
    let app = TestApp::new().await;
    // A registered account, not the factory's: only registration sets a password
    // hash, and a refusal proves nothing unless the password is the right one.
    let password = "correct-horse-battery-staple";
    let registered = response(
        app.post_json_anon(
            "/api/v1/auth/register",
            &json!({
                "email": "mover@example.com",
                "password": password,
                "display_name": "Mover"
            }),
        )
        .await,
        StatusCode::CREATED,
    )
    .await;
    let token = registered["access_token"].as_str().expect("access token");

    let me = |bearer: &str| {
        app.client()
            .get(format!("{}/api/v1/me", app.address))
            .bearer_auth(bearer)
            .send()
    };
    let before = response(me(token).await.expect("profile"), StatusCode::OK).await;
    assert_eq!(before["email_verified"], true);

    let refused = app
        .client()
        .post(format!("{}/api/v1/me/email", app.address))
        .bearer_auth(token)
        .json(&json!({"new_email": "moved@example.com", "password": password}))
        .send()
        .await
        .expect("change email request");
    assert_eq!(
        refused.status(),
        StatusCode::SERVICE_UNAVAILABLE,
        "changing email must be refused while nothing can deliver the verification link"
    );

    // Nothing may have changed: the address, its verified state, and the session
    // must all survive, or the refusal has already caused the lockout it prevents.
    let after = response(me(token).await.expect("profile"), StatusCode::OK).await;
    assert_eq!(after["email"], before["email"], "address must be unchanged");
    assert_eq!(after["email_verified"], true, "verification must survive");
    assert_eq!(
        app.client()
            .get(format!("{}/api/v1/auth/refresh-tokens", app.address))
            .bearer_auth(token)
            .send()
            .await
            .expect("session check")
            .status(),
        StatusCode::OK,
        "the caller's session must survive a refused change"
    );
}
