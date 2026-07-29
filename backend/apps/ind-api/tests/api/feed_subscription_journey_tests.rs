use ind_test_support::spawn_app;
use reqwest::StatusCode;
use reqwest::multipart::{Form, Part};
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::common::{assert_json_response as response, assert_status};

#[tokio::test]
async fn feed_subscription_discovers_updates_searches_imports_and_scopes_real_sources() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/site"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/html")
                .set_body_string(format!(
                    r#"<html><head><link rel="alternate" type="application/rss+xml" href="{}/feed.xml"></head></html>"#,
                    server.uri()
                )),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/feed.xml"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/rss+xml")
                .set_body_string(format!(
                    r#"<?xml version="1.0"?><rss version="2.0"><channel>
                    <title>Boundary Engineering</title><link>{}/site</link>
                    <description>Surgical feed coverage</description>
                    <item><title>First issue</title><link>https://example.com/first</link>
                    <guid>first</guid></item></channel></rss>"#,
                    server.uri()
                )),
        )
        .mount(&server)
        .await;

    let app = spawn_app().await;
    let owner = app.create_web_session().await;
    let stranger = app.create_web_session().await;
    let client = app.authed_client(&owner);
    let site_url = format!("{}/site", server.uri());
    assert_status(
        client
            .post_json(
                "/api/v1/feeds/subscriptions",
                &json!({"url": site_url, "title": null, "poll_interval_override_minutes": 0}),
            )
            .await,
        StatusCode::UNPROCESSABLE_ENTITY,
    )
    .await;
    let created = response(
        client
            .post_json(
                "/api/v1/feeds/subscriptions",
                &json!({
                    "url": site_url,
                    "title": "My engineering feed",
                    "poll_interval_override_minutes": 15
                }),
            )
            .await,
        StatusCode::CREATED,
    )
    .await;
    assert_eq!(created["is_new"], true);
    assert_eq!(
        created["subscription"]["source"]["name"],
        "Boundary Engineering"
    );
    assert_eq!(created["subscription"]["source"]["source_kind"], "rss");
    assert_eq!(created["subscription"]["source"]["visibility"], "public");
    let subscription_id = created["subscription"]["id"].as_str().unwrap();

    let search = response(
        client
            .get("/api/v1/feeds/search?query=Boundary&surface=rss&limit=10")
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(search["items"][0]["name"], "Boundary Engineering");
    assert_status(
        client
            .get("/api/v1/feeds/search?query=Boundary&surface=invalid")
            .await,
        StatusCode::UNPROCESSABLE_ENTITY,
    )
    .await;
    let list = response(
        client.get("/api/v1/feeds/subscriptions?limit=1").await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(list["data"][0]["id"], subscription_id);
    let path = format!("/api/v1/feeds/subscriptions/{subscription_id}");
    let updated = response(
        client
            .patch_json(
                &path,
                &json!({
                    "title": "Renamed feed",
                    "auto_save": true,
                    "poll_interval_override_minutes": 30,
                    "status": "paused"
                }),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(updated["title_override"], "Renamed feed");
    assert_eq!(updated["auto_save"], true);
    assert_eq!(updated["poll_interval_override_minutes"], 30);
    assert_eq!(updated["status"], "paused");
    assert_status(
        app.authed_client(&stranger)
            .patch_json(&path, &json!({"title": "Stolen"}))
            .await,
        StatusCode::NOT_FOUND,
    )
    .await;
    let retried = response(
        client.post_json(&format!("{path}/retry"), &json!({})).await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(retried["id"], subscription_id);

    let opml = format!(
        r#"<?xml version="1.0"?><opml version="2.0"><body>
        <outline text="Duplicate" type="rss" xmlUrl="{site_url}"/>
        </body></opml>"#
    );
    let imported = response(
        client
            .post_multipart(
                "/api/v1/feeds/subscriptions/opml",
                Form::new().part(
                    "file",
                    Part::text(opml)
                        .file_name("subscriptions.opml")
                        .mime_str("text/xml")
                        .unwrap(),
                ),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(imported["created"], 0);
    assert_eq!(imported["skipped"], 1);
    assert!(imported["errors"].as_array().unwrap().is_empty());

    assert_status(client.delete(&path).await, StatusCode::NO_CONTENT).await;
    let remaining = response(
        client.get("/api/v1/feeds/subscriptions").await,
        StatusCode::OK,
    )
    .await;
    assert!(remaining["data"].as_array().unwrap().is_empty());
    let durable: (i64, i64) = sqlx::query_as(
        "SELECT (SELECT count(*) FROM feed_subscriptions), \
         (SELECT count(*) FROM job_outbox WHERE job_type = 'feed.poll')",
    )
    .fetch_one(app.pool())
    .await
    .unwrap();
    assert_eq!(durable.0, 0);
    assert_eq!(durable.1, 1);
}
