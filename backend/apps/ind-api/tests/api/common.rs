use ind_test_support::{AuthedClient, TestApp, TestAuthSession, spawn_app, test_mila_defaults};
use reqwest::StatusCode;
use serde_json::{Value, json};

pub const SIMPLE_READER_HTML: &str = "<article><h1>Integration Reader Article</h1><p>This fixture travels through real HTTP, DB, and S3 paths.</p></article>";

pub const FULL_ARCHIVE_HTML_BASE64: &str =
    "PGh0bWw+PGJvZHk+PGFydGljbGU+QXJjaGl2ZWQ8L2FydGljbGU+PC9ib2R5PjwvaHRtbD4=";

pub struct SaveScenario {
    pub app: TestApp,
    pub web: TestAuthSession,
    pub extension: TestAuthSession,
}

impl SaveScenario {
    pub async fn new() -> Self {
        let app = spawn_app().await;
        let web = app.create_web_session().await;
        let extension = app.create_extension_session(&web.user).await;

        Self {
            app,
            web,
            extension,
        }
    }

    pub fn web_client(&self) -> AuthedClient<'_> {
        self.app.authed_client(&self.web)
    }

    pub fn extension_client(&self) -> AuthedClient<'_> {
        self.app.authed_client(&self.extension)
    }

    pub async fn extension_quick_save(&self, url: &str) -> Value {
        let resp = self
            .extension_client()
            .post_json(
                "/api/v1/extension/quick-save",
                &json!({
                    "url": url,
                    "title": "Quick Save Article"
                }),
            )
            .await;

        assert_json_response(resp, StatusCode::ACCEPTED).await
    }

    pub async fn extension_reader_save(&self, url: &str) -> Value {
        let resp = self
            .extension_client()
            .post_json(
                "/api/v1/extension/reader-save",
                &json!({
                    "url": url,
                    "title": "Reader Save Article",
                    "author": "Integration Tests",
                    "excerpt": "Reader save stores readable HTML immediately.",
                    "reader_html": SIMPLE_READER_HTML,
                    "word_count": 120,
                    "reading_time_minutes": 1,
                    "language": "en"
                }),
            )
            .await;

        assert_json_response(resp, StatusCode::ACCEPTED).await
    }

    pub async fn extension_full_archive(&self, url: &str) -> Value {
        let resp = self
            .extension_client()
            .post_json(
                "/api/v1/extension/full-archive",
                &json!({
                    "url": url,
                    "title": "Full Archive Article",
                    "reader_html": SIMPLE_READER_HTML,
                    "html_base64": FULL_ARCHIVE_HTML_BASE64,
                    "excerpt": "Full archive stores reader and monolith assets immediately.",
                    "author": "Integration Tests",
                    "language": "en",
                    "word_count": 140,
                    "reading_time_minutes": 1
                }),
            )
            .await;

        assert_json_response(resp, StatusCode::ACCEPTED).await
    }

    pub async fn assert_document_asset_downloadable(&self, document_id: &str, asset_kind: &str) {
        let resp = self
            .web_client()
            .get(&format!(
                "/api/v1/documents/{document_id}/assets/{asset_kind}"
            ))
            .await;
        let body = assert_json_response(resp, StatusCode::OK).await;
        assert_eq!(body["asset_kind"], asset_kind);
        let download_url = body["download_url"]
            .as_str()
            .expect("asset response includes download_url");
        assert!(
            download_url.starts_with(&format!("{}/api/v1/assets/", self.app.address)),
            "download_url must point at the API origin, got {download_url}"
        );

        let download = self
            .app
            .client()
            .get(download_url)
            .bearer_auth(&self.web.token)
            .send()
            .await
            .expect("download asset bytes");
        let status = download.status();
        let bytes = download.bytes().await.expect("read downloaded bytes");
        assert!(
            status.is_success(),
            "asset download returned {status}; url: {download_url}"
        );
        assert!(
            !bytes.is_empty(),
            "asset download for {asset_kind} returned empty bytes"
        );
    }

    pub async fn pending_job_count_by_type(&self, job_type: &str) -> i64 {
        self.app
            .worker()
            .pending_job_count_by_type(job_type)
            .await
            .unwrap_or_else(|err| panic!("failed counting {job_type} jobs: {err}"))
    }

    /// Count `job_outbox` rows of a type regardless of dispatch state (sibling
    /// `pending_job_count_by_type` only counts undispatched rows).
    pub async fn total_job_count_by_type(&self, job_type: &str) -> i64 {
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM job_outbox WHERE job_type = $1")
            .bind(job_type)
            .fetch_one(self.app.pool())
            .await
            .unwrap_or_else(|err| panic!("failed counting {job_type} jobs: {err}"))
    }

    /// Drain every pending `job_outbox` row of `job_type` through the real worker dispatch,
    /// executing the handler (not just marking dispatched). Returns the number processed.
    pub async fn run_pending_jobs_of_type(&self, job_type: &str) -> usize {
        dispatch_pending_jobs(&self.app, job_type).await
    }

    pub async fn get_document(&self, document_id: &str) -> Value {
        let resp = self
            .web_client()
            .get(&format!("/api/v1/documents/{document_id}"))
            .await;
        assert_json_response(resp, StatusCode::OK).await
    }
}

pub async fn dispatch_pending_jobs(app: &TestApp, job_type: &str) -> usize {
    use ind_domain::{GenericJobEnvelope, JobOutboxId};

    let ctx = build_worker_context(app);
    let rows = sqlx::query_as::<_, (uuid::Uuid, serde_json::Value, Option<String>)>(
        "SELECT id, payload, dedupe_key FROM job_outbox \
             WHERE dispatched_at IS NULL AND job_type = $1 ORDER BY created_at, id",
    )
    .bind(job_type)
    .fetch_all(app.pool())
    .await
    .expect("load pending jobs");

    let processed = rows.len();
    for (id, payload, dedupe_key) in rows {
        let envelope = GenericJobEnvelope {
            outbox_id: JobOutboxId::from_uuid(id),
            job_type: job_type.to_string(),
            payload,
            dedupe_key,
        };
        ind_worker::jobs::render::dispatch_generic_job(&ctx, envelope)
            .await
            .unwrap_or_else(|err| panic!("worker dispatch for {job_type} failed: {err}"));
        sqlx::query("UPDATE job_outbox SET dispatched_at = now() WHERE id = $1")
            .bind(id)
            .execute(app.pool())
            .await
            .expect("mark dispatched");
    }
    processed
}

/// Document id (`doc_<uuid>`) parsed from a save response's `reader_url`.
pub fn document_id_from_response(body: &Value) -> String {
    body["reader_url"]
        .as_str()
        .expect("response contains reader_url")
        .rsplit('/')
        .next()
        .expect("reader_url has a document id segment")
        .to_string()
}

/// Completed/available asset kinds reported by the document reader view.
pub fn document_available_assets(doc: &Value) -> Vec<String> {
    doc["available_assets"]
        .as_array()
        .expect("available_assets array")
        .iter()
        .map(|a| a.as_str().expect("asset kind string").to_string())
        .collect()
}

pub async fn assert_json_response(resp: reqwest::Response, expected: StatusCode) -> Value {
    let status = resp.status();
    let body = resp.text().await.expect("read response body");
    assert_eq!(
        status, expected,
        "unexpected response status {status}; body: {body}"
    );
    serde_json::from_str(&body).unwrap_or_else(|err| panic!("response was not JSON: {err}; {body}"))
}

pub async fn assert_status(resp: reqwest::Response, expected: StatusCode) {
    assert_eq!(resp.status(), expected);
}

pub fn build_worker_context(app: &TestApp) -> ind_worker::context::WorkerContext {
    let renderer = app.renderer();
    let storage = renderer.storage();
    ind_worker::context::WorkerServicesBuilder::new(
        app.pool().clone(),
        renderer,
        Some(storage),
        "test-bucket".to_string(),
        test_mila_defaults(),
        ind_egress::EgressPolicy::permissive(),
        None,
    )
    .expect("worker services build")
    .with_worker_id("test-worker")
    .without_email_services()
    .build()
}
