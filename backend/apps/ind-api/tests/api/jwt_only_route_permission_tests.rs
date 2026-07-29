use super::resource_route_permission_support::{RouteCase, RoutePermissionFixture};

const JWT_ONLY_ROUTES: &[RouteCase] = &[
    RouteCase::get("/api/v1/me"),
    RouteCase::patch("/api/v1/me"),
    RouteCase::delete("/api/v1/me"),
    RouteCase::post("/api/v1/me/password"),
    RouteCase::post("/api/v1/me/email"),
    RouteCase::post("/api/v1/me/avatar"),
    RouteCase::get("/api/v1/onboarding"),
    RouteCase::post("/api/v1/onboarding/steps/1/complete"),
    RouteCase::post("/api/v1/onboarding/skip"),
    RouteCase::get("/api/v1/home"),
    RouteCase::get("/api/v1/settings/home"),
    RouteCase::patch("/api/v1/settings/home"),
    RouteCase::get("/api/v1/settings/preferences"),
    RouteCase::patch("/api/v1/settings/preferences"),
    RouteCase::get("/api/v1/settings/notifications"),
    RouteCase::patch("/api/v1/settings/notifications"),
    RouteCase::get("/api/v1/settings/archival"),
    RouteCase::patch("/api/v1/settings/archival"),
    RouteCase::get("/api/v1/events/stream"),
];

#[tokio::test]
async fn account_onboarding_home_settings_and_events_routes_remain_jwt_only() {
    let fixture = RoutePermissionFixture::new().await;
    fixture.assert_jwt_only_matrix(JWT_ONLY_ROUTES).await;
}
