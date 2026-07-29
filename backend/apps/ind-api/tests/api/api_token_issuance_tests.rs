use chrono::{DateTime, Utc};
use ind_application::repos::api_token::ApiTokenRepository;
use ind_domain::ApiPermission;
use ind_persistence::repos::PgApiTokenRepository;
use ind_test_support::{TestApp, TestAuthSession, TestPersonalAccessToken, spawn_app};
use reqwest::{Response, StatusCode};
use serde_json::{Value, json};

use super::common::assert_json_response as json_response;

const MAX_EXPIRY_SECONDS: i64 = 31_536_000;

struct TokenFixture {
    app: TestApp,
    web: TestAuthSession,
}

impl TokenFixture {
    async fn new() -> Self {
        let app = spawn_app().await;
        let web = app.create_web_session().await;
        Self { app, web }
    }

    async fn create(&self, body: &Value) -> Response {
        self.app
            .authed_client(&self.web)
            .post_json("/api/v1/tokens", body)
            .await
    }
}

#[tokio::test]
async fn issuance_canonicalizes_typed_permissions_before_persisting() {
    let fixture = TokenFixture::new().await;
    let created = json_response(
        fixture
            .create(&json!({
                "name": "writer",
                "permissions": ["ai:use", "library:write", "library:write"]
            }))
            .await,
        StatusCode::CREATED,
    )
    .await;

    assert_eq!(
        created["permissions"],
        json!(["library:read", "library:write", "ai:use"])
    );
    let persisted = PgApiTokenRepository::new(fixture.app.pool().clone())
        .list_by_user(fixture.web.user.id)
        .await
        .expect("load persisted token");
    assert_eq!(
        persisted[0].permissions,
        [
            ApiPermission::LibraryRead,
            ApiPermission::LibraryWrite,
            ApiPermission::AiUse,
        ]
    );
}

#[tokio::test]
async fn issuance_rejects_empty_legacy_and_unknown_permissions() {
    let fixture = TokenFixture::new().await;

    let empty = fixture
        .create(&json!({"name": "empty", "permissions": []}))
        .await;
    assert_eq!(empty.status(), StatusCode::UNPROCESSABLE_ENTITY);

    for permission in [
        "admin",
        "extension",
        "cli",
        "read",
        "write",
        concat!("obsidian_", "plugin"),
        "future:permission",
    ] {
        let response = fixture
            .create(&json!({"name": permission, "permissions": [permission]}))
            .await;
        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "{permission} must not be accepted as a PAT permission"
        );
    }
}

#[tokio::test]
async fn omitted_expiry_defaults_to_ninety_days() {
    let fixture = TokenFixture::new().await;
    let before = Utc::now() + chrono::Duration::days(90);
    let created = json_response(
        fixture
            .create(&json!({"name": "default", "permissions": ["library:read"]}))
            .await,
        StatusCode::CREATED,
    )
    .await;
    let after = Utc::now() + chrono::Duration::days(90);
    let expires_at = created["expires_at"]
        .as_str()
        .expect("default expiry")
        .parse::<DateTime<Utc>>()
        .expect("RFC 3339 expiry");
    assert!(expires_at >= before && expires_at <= after);
}

#[tokio::test]
async fn explicit_null_expiry_creates_a_non_expiring_token() {
    let fixture = TokenFixture::new().await;
    let created = json_response(
        fixture
            .create(&json!({
                "name": "no expiry",
                "permissions": ["library:read"],
                "expires_in": null
            }))
            .await,
        StatusCode::CREATED,
    )
    .await;

    assert!(created.get("expires_at").is_none());
}

#[tokio::test]
async fn expiry_accepts_the_maximum_and_rejects_values_outside_the_positive_bound() {
    let fixture = TokenFixture::new().await;
    let before = Utc::now() + chrono::Duration::seconds(MAX_EXPIRY_SECONDS);
    let accepted = json_response(
        fixture
            .create(&json!({
                "name": "one year",
                "permissions": ["library:read"],
                "expires_in": MAX_EXPIRY_SECONDS
            }))
            .await,
        StatusCode::CREATED,
    )
    .await;
    let after = Utc::now() + chrono::Duration::seconds(MAX_EXPIRY_SECONDS);
    let expires_at = accepted["expires_at"]
        .as_str()
        .expect("maximum expiry")
        .parse::<DateTime<Utc>>()
        .expect("RFC 3339 expiry");
    assert!(expires_at >= before && expires_at <= after);

    for seconds in [-1, 0, MAX_EXPIRY_SECONDS + 1] {
        let rejected = fixture
            .create(&json!({
                "name": format!("invalid {seconds}"),
                "permissions": ["library:read"],
                "expires_in": seconds
            }))
            .await;
        assert_eq!(
            rejected.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "expires_in={seconds} must be rejected"
        );
    }
}

#[tokio::test]
async fn supplied_short_expiry_is_not_replaced_by_the_default() {
    const SHORT_EXPIRY_SECONDS: i64 = 60;

    let fixture = TokenFixture::new().await;
    let before = Utc::now() + chrono::Duration::seconds(SHORT_EXPIRY_SECONDS);
    let created = json_response(
        fixture
            .create(&json!({
                "name": "short lived",
                "permissions": ["library:read"],
                "expires_in": SHORT_EXPIRY_SECONDS
            }))
            .await,
        StatusCode::CREATED,
    )
    .await;
    let after = Utc::now() + chrono::Duration::seconds(SHORT_EXPIRY_SECONDS);
    let expires_at = created["expires_at"]
        .as_str()
        .expect("short expiry")
        .parse::<DateTime<Utc>>()
        .expect("RFC 3339 expiry");

    assert!(expires_at >= before && expires_at <= after);
}

#[tokio::test]
async fn raw_secret_is_hash_only_at_rest_and_revealed_only_by_creation() {
    let fixture = TokenFixture::new().await;
    let created = json_response(
        fixture
            .create(&json!({"name": "secret", "permissions": ["library:read"]}))
            .await,
        StatusCode::CREATED,
    )
    .await;
    let raw = created["raw_token"].as_str().expect("one-time raw token");
    assert!(created.get("token_hash").is_none());

    let persisted_hash = ind_auth::hash_token(raw);
    let persisted = PgApiTokenRepository::new(fixture.app.pool().clone())
        .find_by_token_hash(&persisted_hash)
        .await
        .expect("load token by hash")
        .expect("persisted token");
    assert_eq!(persisted.token_hash, persisted_hash);
    assert_ne!(persisted_hash, raw);

    let listed = json_response(
        fixture
            .app
            .authed_client(&fixture.web)
            .get("/api/v1/tokens")
            .await,
        StatusCode::OK,
    )
    .await;
    assert!(listed["data"][0].get("raw_token").is_none());
    assert!(listed["data"][0].get("token_hash").is_none());
}

#[tokio::test]
async fn expired_token_fails_before_reaching_a_permissioned_route() {
    let fixture = TokenFixture::new().await;
    let created = json_response(
        fixture
            .create(&json!({"name": "expired", "permissions": ["library:read"]}))
            .await,
        StatusCode::CREATED,
    )
    .await;
    let raw = created["raw_token"].as_str().expect("one-time raw token");
    let token_hash = ind_auth::hash_token(raw);
    let token = TestPersonalAccessToken::new(raw);

    let updated = sqlx::query(
        "UPDATE api_tokens SET expires_at = now() - interval '1 second' WHERE token_hash = $1",
    )
    .bind(token_hash)
    .execute(fixture.app.pool())
    .await
    .expect("expire persisted PAT");
    assert_eq!(updated.rows_affected(), 1);

    let response = fixture.app.authed_client(&token).get("/api/v1/tags").await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn revocation_invalidates_the_next_request() {
    let fixture = TokenFixture::new().await;
    let created = json_response(
        fixture
            .create(&json!({"name": "revocable", "permissions": ["library:read"]}))
            .await,
        StatusCode::CREATED,
    )
    .await;
    let raw = created["raw_token"].as_str().expect("one-time raw token");
    let token_id = created["id"].as_str().expect("created token id");
    let token = TestPersonalAccessToken::new(raw);

    let before_revoke = fixture.app.authed_client(&token).get("/api/v1/tags").await;
    assert_eq!(before_revoke.status(), StatusCode::OK);

    let revoke = fixture
        .app
        .authed_client(&fixture.web)
        .delete(&format!("/api/v1/tokens/{token_id}"))
        .await;
    assert_eq!(revoke.status(), StatusCode::NO_CONTENT);

    let after_revoke = fixture.app.authed_client(&token).get("/api/v1/tags").await;
    assert_eq!(after_revoke.status(), StatusCode::UNAUTHORIZED);
}
