use ind_application::repos::integration_connection::IntegrationConnectionRepository;
use ind_domain::IntegrationProvider;
use ind_persistence::repos::PgIntegrationConnectionRepository;
use ind_test_support::spawn_app;
use reqwest::StatusCode;
use serde_json::json;

use super::common::{assert_json_response as response, assert_status};

#[tokio::test]
async fn notion_controls_persist_selection_enqueue_sync_and_enforce_tenant_scope() {
    let app = spawn_app().await;
    let owner = app.create_web_session().await;
    let other = app.create_web_session().await;
    let connection = PgIntegrationConnectionRepository::new(app.pool().clone())
        .upsert_by_user_provider(
            owner.user.id,
            IntegrationProvider::Notion,
            json!({"workspace_id": "workspace-1", "workspace_name": "Research"}),
            "active",
        )
        .await
        .unwrap();
    let client = app.authed_client(&owner);
    let first = response(
        client
            .post_json(
                "/api/v1/library",
                &json!({"url": "https://example.com/notion-alpha", "title": "Notion Alpha"}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    let second = response(
        client
            .post_json(
                "/api/v1/library",
                &json!({"url": "https://example.com/notion-beta", "title": "Notion Beta"}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    let first_id = first["library_entry_id"].as_str().unwrap();
    let second_id = second["library_entry_id"].as_str().unwrap();
    let base = format!("/api/v1/integrations/{}/notion", connection.id);

    let defaults = response(
        client.get(&format!("{base}/settings")).await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(
        defaults,
        json!({
            "export_automatically": true,
            "include_highlight_locations": true,
            "compact_layout": true,
            "selection_enabled": false
        })
    );
    let settings = response(
        client
            .patch_json(
                &format!("{base}/settings"),
                &json!({"selection_enabled": true, "compact_layout": false}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(settings["selection_enabled"], true);
    assert_eq!(settings["compact_layout"], false);
    assert_eq!(settings["export_automatically"], true);

    let export_path = format!("{base}/export-entries");
    let page = response(
        client
            .get(&format!("{export_path}?q=Alpha&limit=1&offset=0"))
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(page["total_count"], 2);
    assert_eq!(page["filtered_count"], 1);
    assert_eq!(page["items"][0]["library_entry_id"], first_id);
    assert_eq!(page["items"][0]["selected"], true);

    assert_status(
        client
            .patch_json(
                &export_path,
                &json!({"selections": [
                    {"library_entry_id": first_id, "selected": true},
                    {"library_entry_id": first_id, "selected": false}
                ]}),
            )
            .await,
        StatusCode::UNPROCESSABLE_ENTITY,
    )
    .await;
    assert_status(
        client
            .patch_json(
                &export_path,
                &json!({"selections": [
                    {"library_entry_id": first_id, "selected": false},
                    {"library_entry_id": second_id, "selected": true}
                ]}),
            )
            .await,
        StatusCode::NO_CONTENT,
    )
    .await;
    let selected = response(client.get(&export_path).await, StatusCode::OK).await;
    let first = selected["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["library_entry_id"] == first_id)
        .unwrap();
    assert_eq!(first["selected"], false);

    let refreshed = response(
        client
            .post_json(&format!("{export_path}/{second_id}/refresh"), &json!({}))
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(refreshed["library_entry_id"], second_id);
    let sync = response(
        client
            .post_json(
                &format!("/api/v1/integrations/{}/sync", connection.id),
                &json!({}),
            )
            .await,
        StatusCode::ACCEPTED,
    )
    .await;
    assert!(sync["job_id"].is_string());

    assert_status(
        app.authed_client(&other)
            .get(&format!("{base}/settings"))
            .await,
        StatusCode::NOT_FOUND,
    )
    .await;
    let durable: (serde_json::Value, i64) = sqlx::query_as(
        "SELECT config, (SELECT count(*) FROM job_outbox WHERE job_type = \
         'integration.notion.sync_connection' AND payload->>'connection_id' = $2) \
         FROM integration_connections WHERE id = $1",
    )
    .bind(connection.id.into_uuid())
    .bind(connection.id.to_string())
    .fetch_one(app.pool())
    .await
    .unwrap();
    assert_eq!(durable.0["selection_enabled"], true);
    assert_eq!(durable.0["compact_layout"], false);
    assert_eq!(durable.1, 1);
}
