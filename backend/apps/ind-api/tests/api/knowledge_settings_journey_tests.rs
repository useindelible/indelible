use ind_application::repos::entity::EntityRepository;
use ind_domain::{DocumentId, EntityType};
use ind_persistence::repos::PgEntityRepository;
use ind_test_support::spawn_app;
use reqwest::StatusCode;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::common::{SaveScenario, assert_json_response, assert_status, document_id_from_response};

const RSS: &str = r#"<?xml version="1.0"?><rss version="2.0"><channel>
<title>Onboarding Feed</title><link>https://example.com/</link>
<description>Onboarding boundary</description></channel></rss>"#;

#[tokio::test]
async fn onboarding_and_settings_persist_real_account_configuration() {
    let feed = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/feed.xml"))
        .respond_with(
            ResponseTemplate::new(200)
                .append_header("content-type", "application/rss+xml")
                .set_body_string(RSS),
        )
        .expect(1)
        .mount(&feed)
        .await;

    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);
    let initial =
        assert_json_response(client.get("/api/v1/onboarding").await, StatusCode::OK).await;
    assert_eq!(initial["current_step"], 0);
    let feed_url = format!("{}/feed.xml", feed.uri());
    let completed = assert_json_response(
        client
            .post_json(
                "/api/v1/onboarding/steps/3/complete",
                &json!({"data": {"feed_urls": [feed_url]}}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(completed["current_step"], 3);
    let subscriptions = assert_json_response(
        client.get("/api/v1/feeds/subscriptions").await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(subscriptions["data"][0]["input_url"], feed_url);
    assert_eq!(
        app.worker()
            .pending_job_count_by_type("feed.poll")
            .await
            .unwrap(),
        1
    );

    let mut preferences = assert_json_response(
        client.get("/api/v1/settings/preferences").await,
        StatusCode::OK,
    )
    .await;
    preferences["theme"] = json!("dark");
    preferences["reader"]["email_open_mode"] = json!("original");
    preferences["ai"]["custom_prompt"] = json!("  Be concise  ");
    let preferences = assert_json_response(
        client
            .patch_json("/api/v1/settings/preferences", &preferences)
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(preferences["theme"], "dark");
    assert_eq!(preferences["reader"]["email_open_mode"], "original");
    assert_eq!(preferences["ai"]["custom_prompt"], "Be concise");

    let mut notifications = assert_json_response(
        client.get("/api/v1/settings/notifications").await,
        StatusCode::OK,
    )
    .await;
    notifications["daily_review_reminder_time"] = json!(" 07:30 ");
    notifications["marketing_emails"] = json!(true);
    let notifications = assert_json_response(
        client
            .patch_json("/api/v1/settings/notifications", &notifications)
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(notifications["daily_review_reminder_time"], "07:30");
    assert_eq!(notifications["marketing_emails"], true);

    let mut archival = assert_json_response(
        client.get("/api/v1/settings/archival").await,
        StatusCode::OK,
    )
    .await;
    archival["archive_formats"]["readable_html"] = json!(false);
    archival["processing"]["browser_timeout_secs"] = json!(5);
    archival["processing"]["max_concurrent_archives"] = json!(99);
    archival["proxy"]["url"] = json!("   ");
    archival["proxy"]["all_requests"] = json!(true);
    let archival = assert_json_response(
        client
            .patch_json("/api/v1/settings/archival", &archival)
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(archival["archive_formats"]["readable_html"], true);
    assert_eq!(archival["processing"]["browser_timeout_secs"], 30);
    assert_eq!(archival["processing"]["max_concurrent_archives"], 10);
    assert!(archival["proxy"]["url"].is_null());
    assert_eq!(archival["proxy"]["all_requests"], false);
}

#[tokio::test]
async fn highlight_lifecycle_persists_notes_tags_and_tenant_boundaries() {
    let scenario = SaveScenario::new().await;
    let saved = scenario
        .extension_reader_save("https://example.com/highlight-journey")
        .await;
    let document_id = document_id_from_response(&saved);
    let client = scenario.web_client();
    let created = assert_json_response(
        client
            .post_json(
                &format!("/api/v1/documents/{document_id}/highlights"),
                &json!({
                    "color": "yellow",
                    "text_content": "Integration Reader",
                    "locator": {"type": "html", "start_offset": 0, "end_offset": 18}
                }),
            )
            .await,
        StatusCode::CREATED,
    )
    .await;
    let highlight_id = created["id"].as_str().unwrap();
    let changed = assert_json_response(
        client
            .patch_json(
                &format!("/api/v1/highlights/{highlight_id}"),
                &json!({"color": "purple"}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(changed["color"], "purple");
    let note = assert_json_response(
        client
            .put_json(
                &format!("/api/v1/highlights/{highlight_id}/note"),
                &json!({"body": "Connect this to the architecture notes."}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(note["highlight_id"], highlight_id);
    let tags = assert_json_response(
        client
            .put_json(
                &format!("/api/v1/highlights/{highlight_id}/tags"),
                &json!({"tags": [" Rust ", "Architecture"]}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(tags["tags"], json!(["architecture", "rust"]));

    let document_highlights = assert_json_response(
        client
            .get(&format!("/api/v1/documents/{document_id}/highlights"))
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(document_highlights["count"], 1);
    assert_eq!(
        document_highlights["highlights"][0]["note"]["body"],
        note["body"]
    );
    let recent = assert_json_response(
        client.get("/api/v1/highlights/recent?limit=1").await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(recent["highlights"][0]["id"], highlight_id);

    let stranger = scenario.app.create_web_session().await;
    assert_status(
        scenario
            .app
            .authed_client(&stranger)
            .delete(&format!("/api/v1/highlights/{highlight_id}"))
            .await,
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_status(
        client
            .delete(&format!("/api/v1/highlights/{highlight_id}/note"))
            .await,
        StatusCode::NO_CONTENT,
    )
    .await;
    assert_status(
        client
            .delete(&format!("/api/v1/highlights/{highlight_id}"))
            .await,
        StatusCode::NO_CONTENT,
    )
    .await;
}

#[tokio::test]
async fn entity_journey_preserves_mentions_merges_reindexing_and_tenant_scope() {
    let scenario = SaveScenario::new().await;
    let saved = scenario
        .extension_reader_save("https://example.com/entity-journey")
        .await;
    let document_id: DocumentId = document_id_from_response(&saved).parse().unwrap();
    let repo = PgEntityRepository::new(scenario.app.pool().clone());
    let rust = repo
        .insert_canonical(
            scenario.web.user.id,
            "Rust",
            EntityType::Work,
            Some("A systems language"),
        )
        .await
        .unwrap();
    let systems = repo
        .insert_canonical(
            scenario.web.user.id,
            "Systems Programming",
            EntityType::Work,
            None,
        )
        .await
        .unwrap();
    repo.set_document_mentions(
        scenario.web.user.id,
        document_id,
        &[(rust.id, 3), (systems.id, 2)],
    )
    .await
    .unwrap();

    let client = scenario.web_client();
    let listed = assert_json_response(
        client.get("/api/v1/entities?type=topic").await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(listed["data"].as_array().unwrap().len(), 2);
    let detail = assert_json_response(
        client.get(&format!("/api/v1/entities/{}", rust.id)).await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(detail["total_mentions"], 3);
    assert_eq!(detail["co_occurring"][0]["id"], systems.id.to_string());
    let documents = assert_json_response(
        client
            .get(&format!("/api/v1/entities/{}/documents", rust.id))
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(documents["data"][0]["id"], document_id.to_string());

    let renamed = assert_json_response(
        client
            .patch_json(
                &format!("/api/v1/entities/{}", rust.id),
                &json!({"name": " Rust Language ", "description": "  Memory safe  "}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(renamed["name"], "Rust Language");
    assert_eq!(renamed["description"], "Memory safe");
    let merged = assert_json_response(
        client
            .post_json(
                &format!("/api/v1/entities/{}/merge", systems.id),
                &json!({"target_id": rust.id.to_string()}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(merged["id"], rust.id.to_string());
    assert_eq!(merged["total_mentions"], 5);
    assert_eq!(
        scenario
            .pending_job_count_by_type("search.reindex_document")
            .await,
        1,
        "update and merge converge on one deduplicated reindex job"
    );

    let stranger = scenario.app.create_web_session().await;
    assert_status(
        scenario
            .app
            .authed_client(&stranger)
            .get(&format!("/api/v1/entities/{}", rust.id))
            .await,
        StatusCode::NOT_FOUND,
    )
    .await;
}
