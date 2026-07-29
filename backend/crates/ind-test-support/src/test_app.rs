use std::sync::Arc;

use crate::mock_renderer::StorageBackedMockRenderer;
use crate::worker_harness::TestWorkerHarness;
use crate::{TestDb, UserFactory};
use ind_domain::{ClientType, User};

pub const TEST_CIPHER_KEY_B64: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

#[derive(Debug, Clone)]
pub struct TestAppOptions {
    pub allow_signups: bool,
    pub oidc_issuer_url: Option<String>,
    pub asset_serving_mode: &'static str,
}

impl Default for TestAppOptions {
    fn default() -> Self {
        Self {
            allow_signups: true,
            oidc_issuer_url: None,
            // Self-hosters run the shipped default; tests must exercise it too.
            asset_serving_mode: "passthrough",
        }
    }
}

pub fn test_mila_defaults() -> ind_domain::MilaPlatformDefaults {
    ind_domain::MilaPlatformDefaults {
        chat_api_base: "http://localhost:0".into(),
        chat_model: "test-chat".into(),
        embedding_api_base: "http://localhost:0".into(),
        embedding_model: "test-embedding".into(),
        embedding_dim: 768,
        model_context_window: 16_000,
        chat_context_pct: 70,
        chunk_size: 512,
        chunk_overlap: 64,
        top_k: 5,
        cross_item_top_k: 10,
        cross_item_max_per_item: 3,
        enabled: false,
        supports_structured_output: true,
        supports_reasoning_effort: true,
    }
}

fn test_config(base_url: &str, options: &TestAppOptions) -> ind_api::config::ServerConfig {
    let mut config = serde_json::json!({
        "server": {"host": "127.0.0.1", "port": 0, "environment": "development", "base_url": base_url},
        "database_url": "postgres://unused",
        "auth": {
            "csrf_secret": "test-csrf-secret",
            "jwt_secret": "test-jwt-secret-that-is-at-least-32-bytes-long-for-hs256",
            "credential_key": TEST_CIPHER_KEY_B64,
            "allow_signups": options.allow_signups
        },
        "cors": {"origins": [base_url], "frontend_url": base_url},
        "egress": {"allow_private_targets": true},
        "storage": {
            "s3_enabled": false,
            "max_upload_bytes": ind_ingest::MAX_UPLOAD_BYTES,
            "max_import_upload_bytes": 200 * 1024 * 1024,
            "asset_serving_mode": options.asset_serving_mode
        },
        "mila": test_mila_defaults(),
        "tts": {"enabled": true, "use_mock_adapter": true, "deployment": "self_hosted"},
        "rate_limit": {
            "login": {"requests": 1000, "window_secs": 60},
            "registration": {"requests": 1000, "window_secs": 60},
            "password_reset": {"requests": 1000, "window_secs": 60},
            "user_api": {"requests": 1000, "window_secs": 60}
        },
        "log_level": "error"
    });
    if let Some(issuer_url) = options.oidc_issuer_url.as_ref() {
        config["oauth"] = serde_json::json!({
            "oidc_enabled": true,
            "oidc_issuer_url": issuer_url,
            "oidc_client_id": "test-oidc-client",
            "oidc_client_secret": "test-oidc-secret",
            "oidc_provider_name": "Test SSO",
            "oidc_auto_create_users": true
        });
    }
    serde_json::from_value(config).expect("test server config must deserialize")
}

#[derive(Debug, Clone)]
pub struct TestAuthSession {
    pub user: User,
    pub token: String,
}

pub struct AuthedClient<'a> {
    app: &'a TestApp,
    token: &'a str,
}

pub struct TestApp {
    db: TestDb,
    pub address: String,
    pub jwt_secret: Vec<u8>,
    client: reqwest::Client,
    renderer: Arc<StorageBackedMockRenderer>,
    worker: TestWorkerHarness,
    _server_handle: tokio::task::JoinHandle<()>,
    realtime_listener_handle: tokio::task::JoinHandle<()>,
}

pub async fn spawn_app() -> TestApp {
    TestApp::new().await
}

pub async fn spawn_app_with_options(options: TestAppOptions) -> TestApp {
    TestApp::with_options(options).await
}

impl TestApp {
    pub async fn new() -> Self {
        Self::with_options(TestAppOptions::default()).await
    }

    pub async fn with_options(options: TestAppOptions) -> Self {
        let db = TestDb::new().await;
        let jwt_secret = b"test-jwt-secret-that-is-at-least-32-bytes-long-for-hs256".to_vec();
        let storage = db.storage().await;
        let renderer = Arc::new(StorageBackedMockRenderer::new(storage.clone()));
        let worker = TestWorkerHarness::new(db.pool().clone());

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to bind test server");
        let port = listener.local_addr().unwrap().port();
        let address = format!("http://127.0.0.1:{port}");

        let config = test_config(&address, &options);
        let services = ind_api::services::build_with_overrides(
            &config,
            db.pool().clone(),
            ind_api::services::ServiceOverrides {
                storage: Some(storage),
            },
        )
        .await
        .expect("production service graph must build for tests");
        let app = ind_api::router::build(services.state, &config, services.rate_limit_config);
        let realtime_listener_handle = services.realtime_listener_handle;

        let server_handle = tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("test server failed");
        });

        Self {
            db,
            address,
            jwt_secret,
            client: reqwest::Client::new(),
            renderer,
            worker,
            _server_handle: server_handle,
            realtime_listener_handle,
        }
    }

    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }

    pub fn pool(&self) -> &sqlx::PgPool {
        self.db.pool()
    }

    pub async fn storage(&self) -> Arc<dyn ind_application::storage::ObjectStorage> {
        self.db.storage().await
    }

    pub fn s3_endpoint(&self) -> &str {
        self.db.s3_endpoint()
    }

    pub fn renderer(&self) -> Arc<StorageBackedMockRenderer> {
        self.renderer.clone()
    }

    pub fn worker(&self) -> &TestWorkerHarness {
        &self.worker
    }

    pub fn authed_client<'a>(&'a self, session: &'a TestAuthSession) -> AuthedClient<'a> {
        AuthedClient {
            app: self,
            token: &session.token,
        }
    }

    pub async fn create_web_session(&self) -> TestAuthSession {
        let user = UserFactory::new()
            .with_email_verified(true)
            .insert(self.pool())
            .await;
        let token = self.sign_test_token(&user, ClientType::Web);
        TestAuthSession { user, token }
    }

    pub async fn create_extension_session(&self, user: &User) -> TestAuthSession {
        let token = self.sign_test_token(user, ClientType::Extension);
        TestAuthSession {
            user: user.clone(),
            token,
        }
    }

    fn sign_test_token(&self, user: &User, client_type: ClientType) -> String {
        let (token, _expires) = ind_auth::sign_access_token(
            user.id,
            client_type,
            &["read".to_string(), "write".to_string()],
            &self.jwt_secret,
        )
        .expect("failed to sign test JWT");
        token
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.address, path)
    }

    async fn send(&self, request: reqwest::RequestBuilder) -> reqwest::Response {
        request.send().await.expect("test HTTP request failed")
    }

    pub async fn get(&self, path: &str) -> reqwest::Response {
        self.send(self.client.get(self.url(path))).await
    }

    pub async fn post_json_anon<T: serde::Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> reqwest::Response {
        self.send(self.client.post(self.url(path)).json(body)).await
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        self._server_handle.abort();
        self.realtime_listener_handle.abort();
    }
}

impl AuthedClient<'_> {
    fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        self.app
            .client
            .request(method, self.app.url(path))
            .bearer_auth(self.token)
    }

    pub async fn get(&self, path: &str) -> reqwest::Response {
        self.app
            .send(self.request(reqwest::Method::GET, path))
            .await
    }

    pub async fn post_json<T: serde::Serialize>(&self, path: &str, body: &T) -> reqwest::Response {
        self.app
            .send(self.request(reqwest::Method::POST, path).json(body))
            .await
    }

    pub async fn post_multipart(
        &self,
        path: &str,
        form: reqwest::multipart::Form,
    ) -> reqwest::Response {
        self.app
            .send(self.request(reqwest::Method::POST, path).multipart(form))
            .await
    }

    pub async fn patch_json<T: serde::Serialize>(&self, path: &str, body: &T) -> reqwest::Response {
        self.app
            .send(self.request(reqwest::Method::PATCH, path).json(body))
            .await
    }

    pub async fn put_json<T: serde::Serialize>(&self, path: &str, body: &T) -> reqwest::Response {
        self.app
            .send(self.request(reqwest::Method::PUT, path).json(body))
            .await
    }

    pub async fn delete(&self, path: &str) -> reqwest::Response {
        self.app
            .send(self.request(reqwest::Method::DELETE, path))
            .await
    }

    pub async fn delete_json<T: serde::Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> reqwest::Response {
        self.app
            .send(self.request(reqwest::Method::DELETE, path).json(body))
            .await
    }
}
