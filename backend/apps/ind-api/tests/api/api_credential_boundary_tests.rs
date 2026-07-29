use ind_domain::ClientType;
use ind_test_support::{
    TestApiCredential, TestApp, TestAuthSession, TestPersonalAccessToken, UserFactory, spawn_app,
};
use reqwest::StatusCode;
use serde_json::{Value, json};

use super::common::assert_json_response as response;

struct CredentialBoundaryFixture {
    app: TestApp,
    web: TestAuthSession,
}

impl CredentialBoundaryFixture {
    async fn new() -> Self {
        let app = spawn_app().await;
        let web = app.create_web_session().await;
        Self { app, web }
    }

    async fn mint_token(&self, name: &str, permissions: &[&str]) -> TestPersonalAccessToken {
        let created = response(
            self.app
                .authed_client(&self.web)
                .post_json(
                    "/api/v1/tokens",
                    &json!({"name": name, "permissions": permissions}),
                )
                .await,
            StatusCode::CREATED,
        )
        .await;
        TestPersonalAccessToken::new(
            created["raw_token"]
                .as_str()
                .expect("created token exposes raw_token"),
        )
    }

    async fn get_as<C: TestApiCredential>(&self, credential: &C, path: &str) -> StatusCode {
        self.app.authed_client(credential).get(path).await.status()
    }

    async fn post_as<C: TestApiCredential>(
        &self,
        credential: &C,
        path: &str,
        body: &Value,
    ) -> StatusCode {
        self.app
            .authed_client(credential)
            .post_json(path, body)
            .await
            .status()
    }

    async fn delete_as<C: TestApiCredential>(&self, credential: &C, path: &str) -> StatusCode {
        self.app
            .authed_client(credential)
            .delete(path)
            .await
            .status()
    }
}

#[tokio::test]
async fn regular_jwt_clients_keep_account_access() {
    let fixture = CredentialBoundaryFixture::new().await;

    for client_type in [
        ClientType::Web,
        ClientType::Ios,
        ClientType::Android,
        ClientType::Desktop,
        ClientType::Cli,
    ] {
        let session = fixture
            .app
            .create_client_session(&fixture.web.user, client_type);
        assert_eq!(
            fixture.get_as(&session, "/api/v1/tags").await,
            StatusCode::OK,
            "{client_type:?} JWT must retain account access"
        );
    }
}

#[tokio::test]
async fn session_management_refuses_personal_tokens_and_keeps_user_jwts() {
    let fixture = CredentialBoundaryFixture::new().await;
    let token = fixture
        .mint_token("automation", &["library:read", "library:write"])
        .await;

    assert_eq!(
        fixture.get_as(&token, "/api/v1/auth/refresh-tokens").await,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        fixture
            .delete_as(&token, "/api/v1/auth/refresh-tokens")
            .await,
        StatusCode::FORBIDDEN
    );

    for client_type in [ClientType::Web, ClientType::Ios, ClientType::Desktop] {
        let session = fixture
            .app
            .create_client_session(&fixture.web.user, client_type);
        assert_eq!(
            fixture
                .get_as(&session, "/api/v1/auth/refresh-tokens")
                .await,
            StatusCode::OK
        );
    }
}

#[tokio::test]
async fn unverified_user_jwts_resend_verification_but_personal_tokens_cannot() {
    let fixture = CredentialBoundaryFixture::new().await;
    let unverified = UserFactory::new()
        .with_email_verified(false)
        .insert(fixture.app.pool())
        .await;
    let session = fixture
        .app
        .create_client_session(&unverified, ClientType::Web);
    assert_eq!(
        fixture
            .post_as(&session, "/api/v1/auth/email/resend", &json!({}))
            .await,
        StatusCode::OK
    );

    let token = fixture.mint_token("automation", &["library:read"]).await;
    assert_eq!(
        fixture
            .post_as(&token, "/api/v1/auth/email/resend", &json!({}))
            .await,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn extension_status_accepts_only_an_extension_jwt() {
    let fixture = CredentialBoundaryFixture::new().await;
    let extension = fixture
        .app
        .create_client_session(&fixture.web.user, ClientType::Extension);
    assert_eq!(
        fixture.get_as(&extension, "/api/v1/extension/status").await,
        StatusCode::OK
    );

    let token = fixture.mint_token("automation", &["library:read"]).await;
    assert_eq!(
        fixture.get_as(&token, "/api/v1/extension/status").await,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn only_a_verified_web_jwt_manages_personal_tokens() {
    let fixture = CredentialBoundaryFixture::new().await;
    let existing = response(
        fixture
            .app
            .authed_client(&fixture.web)
            .post_json(
                "/api/v1/tokens",
                &json!({"name": "existing", "permissions": ["library:read"]}),
            )
            .await,
        StatusCode::CREATED,
    )
    .await;
    let existing_id = existing["id"].as_str().expect("token id");
    let token = fixture
        .mint_token("automation", &["library:read", "library:write"])
        .await;

    let unverified = UserFactory::new()
        .with_email_verified(false)
        .insert(fixture.app.pool())
        .await;
    let unverified_web = fixture
        .app
        .create_client_session(&unverified, ClientType::Web);
    assert_eq!(
        fixture
            .post_as(
                &unverified_web,
                "/api/v1/tokens",
                &json!({"name": "unverified", "permissions": ["library:read"]}),
            )
            .await,
        StatusCode::FORBIDDEN,
        "an unverified Web JWT must not mint a personal token"
    );

    assert_eq!(
        fixture.get_as(&token, "/api/v1/tokens").await,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        fixture
            .post_as(
                &token,
                "/api/v1/tokens",
                &json!({"name": "nested", "permissions": ["library:read"]}),
            )
            .await,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        fixture
            .delete_as(&token, &format!("/api/v1/tokens/{existing_id}"))
            .await,
        StatusCode::FORBIDDEN
    );

    for client_type in [ClientType::Ios, ClientType::Android, ClientType::Desktop] {
        let session = fixture
            .app
            .create_client_session(&fixture.web.user, client_type);
        assert_eq!(
            fixture.get_as(&session, "/api/v1/tokens").await,
            StatusCode::FORBIDDEN
        );
    }
}

#[tokio::test]
async fn asset_downloads_accept_supported_jwts_and_library_read_tokens() {
    let fixture = CredentialBoundaryFixture::new().await;
    let path = "/api/v1/assets/documents/doc_01912d1e000071b0a000000000000000/readable_html";

    for client_type in [
        ClientType::Extension,
        ClientType::Web,
        ClientType::Ios,
        ClientType::Android,
        ClientType::Desktop,
        ClientType::Cli,
    ] {
        let session = fixture
            .app
            .create_client_session(&fixture.web.user, client_type);
        assert_eq!(
            fixture.get_as(&session, path).await,
            StatusCode::NOT_FOUND,
            "{client_type:?} JWT must reach the asset handler's missing-document response"
        );
    }

    let token = fixture
        .mint_token("library reader", &["library:read"])
        .await;
    assert_eq!(fixture.get_as(&token, path).await, StatusCode::NOT_FOUND);
}
