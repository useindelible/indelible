use super::resource_route_permission_support::{RouteCase, RoutePermissionFixture};

const READ_ROUTES: &[RouteCase] = &[
    RouteCase::get("/api/v1/webhooks"),
    RouteCase::get("/api/v1/webhooks/bad/deliveries"),
];

const WRITE_ROUTES: &[RouteCase] = &[
    RouteCase::post("/api/v1/webhooks"),
    RouteCase::patch("/api/v1/webhooks/bad"),
    RouteCase::delete("/api/v1/webhooks/bad"),
    RouteCase::post("/api/v1/webhooks/bad/rotate-secret"),
    RouteCase::post("/api/v1/webhooks/bad/test"),
];

#[tokio::test]
async fn webhook_routes_enforce_named_read_and_write_permissions() {
    let fixture = RoutePermissionFixture::new().await;
    fixture
        .assert_pat_matrix("webhooks:read", "library:read", READ_ROUTES)
        .await;
    fixture
        .assert_pat_matrix("webhooks:write", "webhooks:read", WRITE_ROUTES)
        .await;
}
