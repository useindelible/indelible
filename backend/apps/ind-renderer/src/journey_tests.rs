use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use axum::body::Bytes;
use axum::http::StatusCode;
use axum::response::Html;
use axum::routing::{any, get};
use secrecy::SecretString;
use serde_json::{Value, json};

use super::browser::BrowserManager;
use super::config::{
    CaptureSettings, ChromiumSettings, EgressSettings, RendererConfig, RendererServerSettings,
    S3Settings,
};
use super::storage::S3Storage;

async fn spawn(app: Router) -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{address}"), task)
}

fn chromium_path() -> PathBuf {
    // Honor the same override the production config loader exposes; CI points
    // this at the runner's preinstalled Chrome, which none of the probes
    // below would find.
    if let Some(path) = std::env::var_os("CHROMIUM_PATH") {
        return PathBuf::from(path);
    }

    let container_path = PathBuf::from("/headless-shell/headless-shell");
    if container_path.is_file() {
        return container_path;
    }

    if let Some(cache) = std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join("Library/Caches/ms-playwright"))
        && let Ok(entries) = std::fs::read_dir(cache)
        && let Some(path) = entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("chromium_headless_shell-"))
            })
            .map(|path| {
                path.join("chrome-headless-shell-mac-arm64")
                    .join("chrome-headless-shell")
            })
            .filter(|path| path.is_file())
            .max()
    {
        return path;
    }

    PathBuf::from("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome")
}

fn config(s3_endpoint: String) -> RendererConfig {
    RendererConfig {
        server: RendererServerSettings {
            environment: "test".into(),
            log_level: "warn".into(),
            host: "127.0.0.1".into(),
            port: 0,
        },
        chromium: ChromiumSettings {
            path: chromium_path(),
            single_process: false,
            virtual_time_budget: None,
            idle_timeout_secs: 30,
        },
        capture: CaptureSettings {
            max_concurrency: 1,
            locale: "en-US".into(),
            timezone: "UTC".into(),
        },
        s3: S3Settings {
            endpoint: Some(s3_endpoint),
            region: "us-east-1".into(),
            bucket: "renderer-test".into(),
            access_key: Some(SecretString::from("test-access")),
            secret_key: Some(SecretString::from("test-secret")),
            force_path_style: true,
        },
        egress: EgressSettings {
            allow_private_targets: true,
        },
    }
}

async fn article() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
        <html><head><title>Renderer boundary</title>
        <meta name="author" content="Indelible">
        <meta property="og:image" content="https://example.com/lead.jpg"></head>
        <body><div role="dialog" style="position:fixed">Dismiss me</div>
        <article><h1>Renderer boundary</h1>
        <p>A real browser should preserve this article while producing every requested capture.</p>
        <p>The second paragraph gives the extractor enough substantive readable content.</p>
        </article></body></html>"#,
    )
}

async fn accept_s3_upload(_body: Bytes) -> StatusCode {
    StatusCode::OK
}

async fn stored_monolith() -> Html<&'static str> {
    Html(
        r#"<!doctype html><html><head><title>Stored monolith</title></head>
        <body><article><h1>Stored monolith</h1>
        <p>Archived content must remain readable while derived captures are regenerated.</p>
        </article></body></html>"#,
    )
}

#[tokio::test]
async fn render_url_crosses_http_browser_extraction_capture_and_storage_boundaries() {
    let (article_url, article_task) = spawn(Router::new().route("/article", get(article))).await;
    let s3 = Router::new()
        .route(
            "/renderer-test/source/monolith.html",
            get(stored_monolith).put(accept_s3_upload),
        )
        .fallback(any(accept_s3_upload));
    let (s3_endpoint, s3_task) = spawn(s3).await;
    let settings = config(s3_endpoint);
    let storage = Arc::new(S3Storage::from_config(&settings).await.unwrap());
    let browser = Arc::new(BrowserManager::new(
        settings.chromium.path.clone(),
        settings.chromium.single_process,
        settings.chromium.virtual_time_budget,
        settings.chromium.idle_timeout_secs,
        settings.capture.max_concurrency,
    ));
    let state = super::build_app_state(
        browser.clone(),
        settings.capture.clone(),
        storage,
        settings.egress_policy(),
    );
    let (renderer_url, renderer_task) = spawn(super::build_router(state)).await;

    let client = reqwest::Client::new();
    let idle_health: Value = client
        .get(format!("{renderer_url}/health"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        idle_health,
        json!({"status": "ok", "browser_running": false})
    );

    let response = client
        .post(format!("{renderer_url}/render/url"))
        .json(&json!({
            "item_id": "itm_renderer_boundary",
            "user_id": "usr_renderer_boundary",
            "url": format!("{article_url}/article"),
            "outputs": ["readable_html", "monolith", "screenshot", "pdf"]
        }))
        .send()
        .await
        .unwrap();
    let status = response.status();
    let rendered: Value = response.json().await.unwrap();
    assert_eq!(status, StatusCode::OK, "{rendered}");
    assert_eq!(rendered["final_url"], format!("{article_url}/article"));
    assert_eq!(rendered["artifacts"].as_array().unwrap().len(), 4);
    let kinds: Vec<_> = rendered["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|artifact| artifact["kind"].as_str().unwrap())
        .collect();
    assert!(kinds.contains(&"readable_html"));
    assert!(kinds.contains(&"monolith"));
    assert!(kinds.contains(&"screenshot"));
    assert!(kinds.contains(&"pdf"));
    assert!(rendered.get("asset_errors").is_none(), "{rendered}");
    let readable = rendered["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|artifact| artifact["kind"] == "readable_html")
        .unwrap();
    assert_eq!(readable["metadata"]["title"], "Renderer boundary");
    assert_eq!(readable["metadata"]["domain"], "127.0.0.1");
    assert!(readable["metadata"]["word_count"].as_i64().unwrap() >= 10);

    let response = client
        .post(format!("{renderer_url}/render/monolith"))
        .json(&json!({
            "item_id": "itm_monolith_boundary",
            "user_id": "usr_monolith_boundary",
            "monolith_s3_key": "source/monolith.html",
            "outputs": ["readable_html", "screenshot", "pdf"]
        }))
        .send()
        .await
        .unwrap();
    let status = response.status();
    let regenerated: Value = response.json().await.unwrap();
    assert_eq!(status, StatusCode::OK, "{regenerated}");
    assert_eq!(regenerated["artifacts"].as_array().unwrap().len(), 3);
    assert!(regenerated.get("final_url").is_none());
    let readable = regenerated["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|artifact| artifact["kind"] == "readable_html")
        .unwrap();
    assert_eq!(readable["metadata"]["title"], "Stored monolith");

    browser.shutdown().await;
    renderer_task.abort();
    article_task.abort();
    s3_task.abort();
}
