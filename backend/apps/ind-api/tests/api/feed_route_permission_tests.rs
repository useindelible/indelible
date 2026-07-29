use super::resource_route_permission_support::{RouteCase, RoutePermissionFixture};

const READ_ROUTES: &[RouteCase] = &[
    RouteCase::get("/api/v1/feeds/search"),
    RouteCase::get("/api/v1/feeds/subscriptions"),
    RouteCase::get("/api/v1/feeds/deliveries"),
    RouteCase::get("/api/v1/feeds/deliveries/stats"),
    RouteCase::get("/api/v1/feeds/deliveries/bad"),
    RouteCase::get("/api/v1/email-aliases"),
    RouteCase::get("/api/v1/email-senders"),
];

const WRITE_ROUTES: &[RouteCase] = &[
    RouteCase::post("/api/v1/feeds/subscriptions"),
    RouteCase::patch("/api/v1/feeds/subscriptions/bad"),
    RouteCase::delete("/api/v1/feeds/subscriptions/bad"),
    RouteCase::post("/api/v1/feeds/subscriptions/opml"),
    RouteCase::post("/api/v1/feeds/subscriptions/bad/retry"),
    RouteCase::post("/api/v1/feeds/deliveries/mark-all-seen"),
    RouteCase::post("/api/v1/feeds/deliveries/read-ahead"),
    RouteCase::post("/api/v1/feeds/deliveries/bad/seen"),
    RouteCase::post("/api/v1/feeds/deliveries/bad/dismiss"),
    RouteCase::post("/api/v1/feeds/deliveries/bad/prepare"),
    RouteCase::post("/api/v1/email-aliases"),
    RouteCase::delete("/api/v1/email-aliases/bad"),
    RouteCase::patch("/api/v1/email-senders/bad"),
    RouteCase::post("/api/v1/email-senders/bad/unsubscribe"),
];

#[tokio::test]
async fn feed_routes_enforce_named_read_and_write_permissions() {
    let fixture = RoutePermissionFixture::new().await;
    fixture
        .assert_pat_matrix("feeds:read", "library:read", READ_ROUTES)
        .await;
    fixture
        .assert_pat_matrix("feeds:write", "feeds:read", WRITE_ROUTES)
        .await;
}
