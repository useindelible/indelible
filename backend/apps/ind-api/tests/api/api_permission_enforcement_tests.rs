use ind_domain::ClientType;
use ind_test_support::{
    TestApiCredential, TestApp, TestAuthSession, TestPersonalAccessToken, spawn_app,
};
use reqwest::StatusCode;
use serde_json::{Value, json};

use super::common::assert_json_response;

struct PermissionFixture {
    app: TestApp,
    web: TestAuthSession,
}

impl PermissionFixture {
    async fn new() -> Self {
        let app = spawn_app().await;
        let web = app.create_web_session().await;
        Self { app, web }
    }

    async fn mint_token(&self, name: &str, permissions: &[&str]) -> TestPersonalAccessToken {
        let created = assert_json_response(
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
}

#[tokio::test]
async fn document_asset_route_requires_library_read_permission() {
    let fixture = PermissionFixture::new().await;

    for (name, permissions, expected) in [
        (
            "library reader",
            &["library:read"][..],
            StatusCode::NOT_FOUND,
        ),
        (
            "library writer",
            &["library:write"][..],
            StatusCode::NOT_FOUND,
        ),
        ("AI caller", &["ai:use"][..], StatusCode::FORBIDDEN),
        ("vault", &["obsidian:sync"][..], StatusCode::FORBIDDEN),
    ] {
        let token = fixture.mint_token(name, permissions).await;
        assert_eq!(
            fixture
                .get_as(&token, "/api/v1/documents/bad/assets/readable_html")
                .await,
            expected,
            "{permissions:?} must follow the document asset policy"
        );
    }
}

#[tokio::test]
async fn library_query_post_uses_library_read_permission() {
    let fixture = PermissionFixture::new().await;
    let token = fixture.mint_token("reader", &["library:read"]).await;
    let query = json!({"limit": 10});

    assert_eq!(
        fixture
            .post_as(&token, "/api/v1/library/query", &query)
            .await,
        StatusCode::OK,
        "a library:read PAT must pass the library query policy"
    );

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
            fixture
                .post_as(&session, "/api/v1/library/query", &query)
                .await,
            StatusCode::OK,
            "{client_type:?} JWT must pass the library query policy"
        );
    }
}

#[tokio::test]
async fn unannotated_routes_deny_a_valid_pat() {
    let fixture = PermissionFixture::new().await;
    let token = fixture.mint_token("reader", &["library:read"]).await;

    assert_eq!(
        fixture.get_as(&token, "/api/v1/me").await,
        StatusCode::FORBIDDEN
    );
}
