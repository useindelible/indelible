use std::path::Path;

use tower_http::services::{ServeDir, ServeFile};

/// Serves the built web app when its directory is present, so one container
/// answers both the UI and the API on a single origin. That is the contract the
/// clients already assume: the extension calls `${serverUrl}/api/v1/...` and a
/// web route on the same host, and the SPA's production API base is the empty
/// string.
///
/// Returns `None` when there is no build to serve (source checkouts, tests),
/// leaving the router API-only.
pub fn spa_service(web_root: &Path) -> Option<ServeDir<ServeFile>> {
    let index = web_root.join("index.html");
    if !index.is_file() {
        return None;
    }

    // Unknown paths fall back to index.html so client-side routes survive a
    // hard refresh; real files are still served from disk.
    Some(ServeDir::new(web_root).fallback(ServeFile::new(index)))
}

#[cfg(test)]
mod tests {
    use axum::Router;
    use axum::body::Body;
    use axum::routing::get;
    use http::{Request, StatusCode};
    use tower::ServiceExt;

    use super::*;

    struct TempWebRoot(std::path::PathBuf);

    impl TempWebRoot {
        fn new() -> Self {
            let dir = std::env::temp_dir().join(format!("ind-web-ui-{}", uuid::Uuid::now_v7()));
            std::fs::create_dir_all(dir.join("assets")).unwrap();
            std::fs::write(dir.join("index.html"), "<html>indelible</html>").unwrap();
            std::fs::write(dir.join("assets/app.js").as_path(), "console.log(1)").unwrap();
            Self(dir)
        }
    }

    impl Drop for TempWebRoot {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    async fn body_of(response: axum::response::Response) -> String {
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    fn app(web_root: &Path) -> Router {
        Router::new()
            .route("/api/health", get(|| async { "api" }))
            .fallback_service(spa_service(web_root).expect("web root should be served"))
    }

    async fn get_path(web_root: &Path, path: &str) -> axum::response::Response {
        app(web_root)
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    #[test]
    fn absent_web_root_is_not_served() {
        let missing = std::env::temp_dir().join("ind-web-ui-does-not-exist");
        assert!(spa_service(&missing).is_none());
    }

    #[tokio::test]
    async fn built_assets_are_served() {
        let root = TempWebRoot::new();

        let response = get_path(&root.0, "/assets/app.js").await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(body_of(response).await, "console.log(1)");
    }

    #[tokio::test]
    async fn client_side_routes_fall_back_to_index() {
        let root = TempWebRoot::new();

        let response = get_path(&root.0, "/library/some-document").await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(body_of(response).await, "<html>indelible</html>");
    }

    #[tokio::test]
    async fn api_routes_win_over_the_spa_fallback() {
        let root = TempWebRoot::new();

        let response = get_path(&root.0, "/api/health").await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(body_of(response).await, "api");
    }
}
