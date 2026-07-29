use axum::body::{Body, to_bytes};
use axum::routing::get;
use axum::{Router, response::Response};
use chrono::Duration;
use http::{Request, StatusCode, header};
use ind_auth::{ApiTokenService, CreateApiTokenRequest};
use ind_domain::{ApiPermission, ClientType, User, UserId};
use ind_http_api::{
    AccessPolicy, AppState, PermissionAccess, RequireAiReadAndLibraryRead,
    RequireAiUseAndLibraryRead, RequireExtensionAccessJwt, RequireLibraryRead,
    RequireMobileAccessJwt, RequireUserAccessJwt, RequireVerifiedUserAccessJwt,
    RequireVerifiedWebAccessJwt,
};
use ind_persistence::repos::PgApiTokenRepository;
use ind_test_support::{TestApp, TestAuthSession, UserFactory, spawn_app};
use serde_json::Value;
use tower::ServiceExt;

mod jwt;
mod permission;

struct ExtensionAllowedLibraryReadPolicy;

impl AccessPolicy for ExtensionAllowedLibraryReadPolicy {
    const REQUIRED: &'static [ApiPermission] = &[ApiPermission::LibraryRead];
    const ALLOW_EXTENSION_JWT: bool = true;
}

type ExtensionAllowedLibraryRead = PermissionAccess<ExtensionAllowedLibraryReadPolicy>;

async fn library_read(_: RequireLibraryRead) -> StatusCode {
    StatusCode::OK
}

async fn ai_read_library_read(_: RequireAiReadAndLibraryRead) -> StatusCode {
    StatusCode::OK
}

async fn ai_use_library_read(_: RequireAiUseAndLibraryRead) -> StatusCode {
    StatusCode::OK
}

async fn extension_allowed_library_read(_: ExtensionAllowedLibraryRead) -> StatusCode {
    StatusCode::OK
}

async fn user_jwt(_: RequireUserAccessJwt) -> StatusCode {
    StatusCode::OK
}

async fn verified_user_jwt(_: RequireVerifiedUserAccessJwt) -> StatusCode {
    StatusCode::OK
}

async fn verified_web_jwt(_: RequireVerifiedWebAccessJwt) -> StatusCode {
    StatusCode::OK
}

async fn extension_jwt(_: RequireExtensionAccessJwt) -> StatusCode {
    StatusCode::OK
}

async fn mobile_jwt(_: RequireMobileAccessJwt) -> StatusCode {
    StatusCode::OK
}

fn extractor_router(state: AppState) -> Router {
    Router::new()
        .route("/permission/library-read", get(library_read))
        .route(
            "/permission/ai-read-library-read",
            get(ai_read_library_read),
        )
        .route("/permission/ai-use-library-read", get(ai_use_library_read))
        .route(
            "/permission/extension-allowed-library-read",
            get(extension_allowed_library_read),
        )
        .route("/jwt/user", get(user_jwt))
        .route("/jwt/verified-user", get(verified_user_jwt))
        .route("/jwt/verified-web", get(verified_web_jwt))
        .route("/jwt/extension", get(extension_jwt))
        .route("/jwt/mobile", get(mobile_jwt))
        .with_state(state)
}

struct ExtractorFixture {
    app: TestApp,
    router: Router,
    verified: TestAuthSession,
    unverified: User,
}

impl ExtractorFixture {
    async fn new() -> Self {
        let app = spawn_app().await;
        let verified = app.create_web_session().await;
        let unverified = UserFactory::new()
            .with_email_verified(false)
            .insert(app.pool())
            .await;
        let router = extractor_router(app.state());
        Self {
            app,
            router,
            verified,
            unverified,
        }
    }

    fn jwt(&self, user: &User, client_type: ClientType) -> String {
        self.app.create_client_session(user, client_type).token
    }

    async fn pat(&self, user_id: UserId, permissions: Vec<ApiPermission>) -> String {
        ApiTokenService::new(PgApiTokenRepository::new(self.app.pool().clone()))
            .create_api_token(CreateApiTokenRequest {
                user_id,
                name: "extractor boundary".to_string(),
                permissions,
                expires_in: Some(Duration::days(1)),
            })
            .await
            .expect("create test PAT")
            .raw_token
    }

    async fn request(&self, path: &str, token: &str) -> Response {
        self.router
            .clone()
            .oneshot(
                Request::builder()
                    .uri(path)
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("build extractor request"),
            )
            .await
            .expect("extractor router response")
    }
}

async fn assert_problem(response: Response, expected_detail: &str) {
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read problem body");
    let problem: Value = serde_json::from_slice(&body).expect("parse problem body");
    assert_eq!(problem["detail"], expected_detail);
}

async fn assert_insufficient(response: Response, expected_scope: &str) {
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        response
            .headers()
            .get(header::WWW_AUTHENTICATE)
            .expect("insufficient-scope challenge"),
        expected_scope
    );
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read problem body");
    let problem: Value = serde_json::from_slice(&body).expect("parse problem body");
    assert_eq!(problem["code"], "insufficient_permissions");
}
