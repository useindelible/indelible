use ind_application::repos::email_sender::EmailSenderRepository;
use ind_domain::CanonicalAddress;
use ind_persistence::repos::PgEmailSenderRepository;
use ind_test_support::spawn_app;
use reqwest::StatusCode;
use serde_json::json;

use super::common::{assert_json_response as response, assert_status};

#[tokio::test]
async fn obsidian_journey_provisions_permissioned_access_and_previews_settings() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);
    let empty = response(client.get("/api/v1/integrations").await, StatusCode::OK).await;
    assert!(empty["connections"].as_array().unwrap().is_empty());
    response(
        client
            .post_json("/api/v1/integrations/obsidian/setup", &json!({}))
            .await,
        StatusCode::OK,
    )
    .await;
    let token = response(
        client
            .post_json(
                "/api/v1/tokens",
                &json!({"name": "Obsidian", "permissions": ["obsidian:sync"]}),
            )
            .await,
        StatusCode::CREATED,
    )
    .await;
    let raw_token = token["raw_token"].as_str().unwrap();
    assert!(raw_token.starts_with("ind_"));
    assert_eq!(token["permissions"], json!(["obsidian:sync"]));
    let integrations = response(client.get("/api/v1/integrations").await, StatusCode::OK).await;
    let connection = &integrations["connections"][0];
    assert_eq!(connection["provider"], "obsidian");
    assert_eq!(connection["status"], "pending");
    let connection_id = connection["id"].as_str().unwrap();
    let settings_path = format!("/api/v1/integrations/{connection_id}/obsidian/settings");
    let mut settings = response(client.get(&settings_path).await, StatusCode::OK).await;
    settings["group_files_in_category_folders"] = json!(false);
    settings["export_all_reader_documents"] = json!(true);
    settings["properties_template"] = json!("source: indelible");
    settings["page_title_template"] = json!("## {{title}}");
    settings["file_name_template"] = json!("{{title}} custom");
    let patched = response(
        client.patch_json(&settings_path, &settings).await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(patched["export_all_reader_documents"], true);
    let preview = response(
        client
            .post_json(
                &format!("/api/v1/integrations/{connection_id}/obsidian/preview"),
                &json!({}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert!(
        preview["file_path"]
            .as_str()
            .unwrap()
            .ends_with(" custom.md")
    );
    assert!(
        preview["full_content"]
            .as_str()
            .unwrap()
            .contains("source: indelible")
    );

    let saved = response(
        client
            .post_json(
                "/api/v1/library",
                &json!({"url": "https://example.com/obsidian-export", "title": "Export me"}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    let subject_id = saved["library_entry_id"].as_str().unwrap();
    let export_request = |method: reqwest::Method, path: &str| {
        app.client()
            .request(method, format!("{}{}", app.address, path))
            .bearer_auth(raw_token)
    };
    assert_status(
        client.get("/api/v1/export/obsidian/runs/bad").await,
        StatusCode::NOT_FOUND,
    )
    .await;
    let run = response(
        export_request(reqwest::Method::POST, "/api/v1/export/obsidian/runs")
            .json(&json!({"force_subject_ids": [subject_id]}))
            .send()
            .await
            .unwrap(),
        StatusCode::ACCEPTED,
    )
    .await;
    let run_id = run["run_id"].as_str().unwrap();
    assert_eq!(run["task_status"], "pending");
    let status = response(
        export_request(
            reqwest::Method::GET,
            &format!("/api/v1/export/obsidian/runs/{run_id}"),
        )
        .send()
        .await
        .unwrap(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(status["run_id"], run_id);
    response(
        export_request(
            reqwest::Method::POST,
            &format!("/api/v1/export/obsidian/runs/{run_id}/ack"),
        )
        .json(&json!({}))
        .send()
        .await
        .unwrap(),
        StatusCode::OK,
    )
    .await;
    let refreshed = response(
        export_request(reqwest::Method::POST, "/api/v1/export/obsidian/refresh")
            .json(&json!({"subject_ids": [subject_id], "reason": "manual-test"}))
            .send()
            .await
            .unwrap(),
        StatusCode::OK,
    )
    .await;
    assert_eq!(refreshed["queued"], 1);
    let durable: (i64, i64) = sqlx::query_as(
        "SELECT \
           (SELECT count(*) FROM job_outbox WHERE job_type = 'integration.obsidian.sync_connection'), \
           (SELECT count(*) FROM obsidian_export_refresh_queue)",
    )
    .fetch_one(app.pool())
    .await
    .unwrap();
    assert_eq!(durable, (1, 1));
}

#[tokio::test]
async fn email_sender_journey_is_user_scoped_and_persists_preferences() {
    let app = spawn_app().await;
    let owner = app.create_web_session().await;
    let other = app.create_web_session().await;
    let sender = PgEmailSenderRepository::new(app.pool().clone())
        .upsert_for_user(
            owner.user.id,
            &CanonicalAddress::new("newsletter@example.com"),
            Some("<weekly.example>"),
            Some("Weekly"),
        )
        .await
        .unwrap();
    let client = app.authed_client(&owner);
    let list = response(client.get("/api/v1/email-senders").await, StatusCode::OK).await;
    assert_eq!(list["data"][0]["id"], sender.id.to_string());
    let path = format!("/api/v1/email-senders/{}", sender.id);
    let changed = response(
        client
            .patch_json(
                &path,
                &json!({"blocked": true, "render_default": "original"}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(changed["blocked"], true);
    assert_eq!(changed["render_default"], "original");
    assert_status(
        app.authed_client(&other)
            .patch_json(&path, &json!({"blocked": false}))
            .await,
        StatusCode::NOT_FOUND,
    )
    .await;
}
