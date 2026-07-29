use reqwest::StatusCode;

use super::resource_route_permission_support::{RouteCase, RoutePermissionFixture};

const READ_ROUTES: &[RouteCase] = &[
    RouteCase::get("/api/v1/integrations"),
    RouteCase::get("/api/v1/integrations/bad/notion/settings"),
    RouteCase::get("/api/v1/integrations/bad/notion/export-entries"),
    RouteCase::get("/api/v1/integrations/bad/obsidian/settings"),
    RouteCase::post("/api/v1/integrations/bad/obsidian/preview"),
];

const WRITE_ROUTES: &[RouteCase] = &[
    RouteCase::post("/api/v1/integrations/notion/authorize"),
    RouteCase::delete("/api/v1/integrations/bad"),
    RouteCase::post("/api/v1/integrations/bad/sync"),
    RouteCase::patch("/api/v1/integrations/bad/notion/settings"),
    RouteCase::patch("/api/v1/integrations/bad/notion/export-entries"),
    RouteCase::post("/api/v1/integrations/bad/notion/export-entries/bad/refresh"),
    RouteCase::patch("/api/v1/integrations/bad/obsidian/settings"),
    RouteCase::post("/api/v1/integrations/obsidian/setup"),
];

#[tokio::test]
async fn integration_routes_enforce_named_permissions_and_keep_callback_public() {
    let fixture = RoutePermissionFixture::new().await;
    fixture
        .assert_pat_matrix("integrations:read", "library:read", READ_ROUTES)
        .await;
    fixture
        .assert_pat_matrix("integrations:write", "integrations:read", WRITE_ROUTES)
        .await;

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("build no-redirect client");
    let callback = client
        .get(format!(
            "{}/api/v1/integrations/notion/callback?error=access_denied",
            fixture.app.address
        ))
        .send()
        .await
        .expect("public integration callback request");
    assert_eq!(callback.status(), StatusCode::TEMPORARY_REDIRECT);
}
