use super::resource_route_permission_support::{RouteCase, RoutePermissionFixture};

const READ_ROUTES: &[RouteCase] = &[
    RouteCase::get("/api/v1/library"),
    RouteCase::post("/api/v1/library/query"),
    RouteCase::get("/api/v1/library/count"),
    RouteCase::get("/api/v1/library/counts"),
    RouteCase::get("/api/v1/library/trash"),
    RouteCase::get("/api/v1/library/bad"),
    RouteCase::get("/api/v1/library/bad/tags"),
    RouteCase::get("/api/v1/documents/bad"),
    RouteCase::get("/api/v1/documents/bad/toc"),
    RouteCase::get("/api/v1/documents/bad/entities"),
    RouteCase::get("/api/v1/documents/bad/highlights"),
    RouteCase::get("/api/v1/documents/bad/note"),
    RouteCase::get("/api/v1/documents/bad/epub/toc"),
    RouteCase::get("/api/v1/documents/bad/epub/chapters/0"),
    RouteCase::get("/api/v1/collections"),
    RouteCase::get("/api/v1/collections/bad"),
    RouteCase::get("/api/v1/collections/bad/children"),
    RouteCase::get("/api/v1/collections/bad/entries"),
    RouteCase::get("/api/v1/highlights/recent"),
    RouteCase::get("/api/v1/highlights/bad/tags"),
    RouteCase::get("/api/v1/tags"),
    RouteCase::get("/api/v1/tags/bad"),
    RouteCase::get("/api/v1/tags/bad/entries"),
    RouteCase::get("/api/v1/tags/bad/highlights"),
    RouteCase::get("/api/v1/smart-lists"),
    RouteCase::get("/api/v1/smart-lists/bad"),
    RouteCase::get("/api/v1/smart-lists/bad/entries"),
    RouteCase::get("/api/v1/entities"),
    RouteCase::get("/api/v1/entities/bad"),
    RouteCase::get("/api/v1/entities/bad/documents"),
    RouteCase::get("/api/v1/search"),
    RouteCase::get("/api/v1/search/suggestions"),
    RouteCase::get("/api/v1/search/recent"),
    RouteCase::get("/api/v1/imports"),
    RouteCase::get("/api/v1/imports/bad"),
];

const WRITE_ROUTES: &[RouteCase] = &[
    RouteCase::post("/api/v1/library"),
    RouteCase::post("/api/v1/library/uploads"),
    RouteCase::post("/api/v1/library/from-delivery"),
    RouteCase::post("/api/v1/library/trash/empty"),
    RouteCase::delete("/api/v1/library/bad"),
    RouteCase::post("/api/v1/library/bad/restore"),
    RouteCase::post("/api/v1/library/bad/purge"),
    RouteCase::put("/api/v1/library/bad/tags"),
    RouteCase::post("/api/v1/library/bad/triage"),
    RouteCase::post("/api/v1/library/bad/favorite"),
    RouteCase::post("/api/v1/library/bad/shortlist"),
    RouteCase::post("/api/v1/documents/bad/reprocess"),
    RouteCase::post("/api/v1/documents/bad/highlights"),
    RouteCase::put("/api/v1/documents/bad/note"),
    RouteCase::patch("/api/v1/documents/bad/progress"),
    RouteCase::post("/api/v1/collections"),
    RouteCase::patch("/api/v1/collections/bad"),
    RouteCase::delete("/api/v1/collections/bad"),
    RouteCase::post("/api/v1/collections/bad/entries"),
    RouteCase::delete("/api/v1/collections/bad/entries/bad"),
    RouteCase::patch("/api/v1/highlights/bad"),
    RouteCase::delete("/api/v1/highlights/bad"),
    RouteCase::put("/api/v1/highlights/bad/note"),
    RouteCase::delete("/api/v1/highlights/bad/note"),
    RouteCase::put("/api/v1/highlights/bad/tags"),
    RouteCase::post("/api/v1/tags"),
    RouteCase::post("/api/v1/tags/merge"),
    RouteCase::patch("/api/v1/tags/bad"),
    RouteCase::delete("/api/v1/tags/bad"),
    RouteCase::post("/api/v1/smart-lists"),
    RouteCase::patch("/api/v1/smart-lists/bad"),
    RouteCase::delete("/api/v1/smart-lists/bad"),
    RouteCase::patch("/api/v1/smart-lists/bad/pin"),
    RouteCase::patch("/api/v1/entities/bad"),
    RouteCase::post("/api/v1/entities/bad/merge"),
    RouteCase::delete("/api/v1/search/recent"),
    RouteCase::delete("/api/v1/search/recent/bad"),
    RouteCase::post("/api/v1/imports/bad"),
    RouteCase::delete("/api/v1/imports/bad/rollback"),
];

#[tokio::test]
async fn library_routes_enforce_named_read_and_write_permissions() {
    let fixture = RoutePermissionFixture::new().await;
    fixture
        .assert_pat_matrix("library:read", "feeds:read", READ_ROUTES)
        .await;
    fixture
        .assert_pat_matrix("library:write", "library:read", WRITE_ROUTES)
        .await;
}
