use ind_test_support::{DocumentFactory, spawn_app};
use reqwest::StatusCode;
use serde_json::{Value, json};

use super::common::assert_json_response as response;

fn config(embedding_dim: i32) -> Value {
    json!({
        "chat_api_base": "https://api.openai.com/v1",
        "chat_model": "gpt-4.1-mini",
        "embedding_api_base": "https://api.openai.com/v1",
        "embedding_model": "text-embedding-3-small",
        "embedding_dim": embedding_dim,
        "model_context_window": 16000,
        "chat_context_pct": 70,
        "top_k": 5,
        "cross_item_top_k": 10,
        "cross_item_max_per_item": 3,
        "enabled": true
    })
}

fn byo_config(enabled: bool) -> Value {
    let mut body = config(768);
    body["embedding_model"] = json!("custom-embed");
    body["byo_enabled"] = json!(enabled);
    body
}

#[tokio::test]
async fn byo_toggle_schedules_the_new_effective_embedding_target() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);
    let document = DocumentFactory::new(session.user.id)
        .with_title("Provider toggle")
        .insert(app.pool())
        .await;
    ind_test_support::LibraryEntryFactory::new(session.user.id, document.id)
        .insert(app.pool())
        .await;
    sqlx::query(
        "INSERT INTO archive_assets \
         (id, document_id, asset_kind, s3_key, s3_bucket, content_type, size_bytes, created_at, status) \
         VALUES ($1, $2, 'readable_html', $3, 'test', 'text/html', 64, now(), 'completed')",
    )
    .bind(uuid::Uuid::now_v7())
    .bind(document.id.into_uuid())
    .bind(format!("documents/{}/readable.html", document.id))
    .execute(app.pool())
    .await
    .unwrap();

    response(
        client
            .post_json("/api/v1/mila/config", &byo_config(true))
            .await,
        StatusCode::OK,
    )
    .await;
    sqlx::query("DELETE FROM job_outbox WHERE payload->>'document_id' = $1")
        .bind(document.id.to_string())
        .execute(app.pool())
        .await
        .unwrap();

    response(
        client
            .post_json("/api/v1/mila/config", &byo_config(false))
            .await,
        StatusCode::OK,
    )
    .await;

    let queued: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM job_outbox \
         WHERE job_type = 'document.ai.embed' AND payload->>'document_id' = $1",
    )
    .bind(document.id.to_string())
    .fetch_one(app.pool())
    .await
    .unwrap();
    assert_eq!(queued, 1);

    sqlx::query(
        "UPDATE job_outbox SET dispatched_at = now() \
         WHERE job_type = 'document.ai.embed' AND payload->>'document_id' = $1",
    )
    .bind(document.id.to_string())
    .execute(app.pool())
    .await
    .unwrap();

    let mut chat_only_change = byo_config(false);
    chat_only_change["chat_model"] = json!("gpt-4.1");
    response(
        client
            .post_json("/api/v1/mila/config", &chat_only_change)
            .await,
        StatusCode::OK,
    )
    .await;
    let remains_dispatched: bool = sqlx::query_scalar(
        "SELECT dispatched_at IS NOT NULL FROM job_outbox \
         WHERE job_type = 'document.ai.embed' AND payload->>'document_id' = $1",
    )
    .bind(document.id.to_string())
    .fetch_one(app.pool())
    .await
    .unwrap();
    assert!(
        remains_dispatched,
        "chat-only saves must not re-arm embed work"
    );

    response(
        client
            .post_json("/api/v1/mila/config/reindex", &chat_only_change)
            .await,
        StatusCode::OK,
    )
    .await;
    let retry_rearmed: bool = sqlx::query_scalar(
        "SELECT dispatched_at IS NULL FROM job_outbox \
         WHERE job_type = 'document.ai.embed' AND payload->>'document_id' = $1",
    )
    .bind(document.id.to_string())
    .fetch_one(app.pool())
    .await
    .unwrap();
    assert!(retry_rearmed, "explicit Retry must re-arm embed work");
}

#[tokio::test]
async fn enabling_mila_preserves_the_full_document_processing_backfill() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);
    let document = DocumentFactory::new(session.user.id)
        .with_title("Enable Mila")
        .insert(app.pool())
        .await;
    ind_test_support::LibraryEntryFactory::new(session.user.id, document.id)
        .insert(app.pool())
        .await;
    sqlx::query(
        "INSERT INTO archive_assets \
         (id, document_id, asset_kind, s3_key, s3_bucket, content_type, size_bytes, created_at, status) \
         VALUES ($1, $2, 'readable_html', $3, 'test', 'text/html', 64, now(), 'completed')",
    )
    .bind(uuid::Uuid::now_v7())
    .bind(document.id.into_uuid())
    .bind(format!("documents/{}/readable.html", document.id))
    .execute(app.pool())
    .await
    .unwrap();

    let mut disabled = byo_config(false);
    disabled["enabled"] = json!(false);
    response(
        client.post_json("/api/v1/mila/config", &disabled).await,
        StatusCode::OK,
    )
    .await;
    let mut enabled = disabled;
    enabled["enabled"] = json!(true);
    response(
        client.post_json("/api/v1/mila/config", &enabled).await,
        StatusCode::OK,
    )
    .await;

    let queued_types: Vec<String> = sqlx::query_scalar(
        "SELECT job_type FROM job_outbox \
         WHERE payload->>'document_id' = $1 ORDER BY job_type",
    )
    .bind(document.id.to_string())
    .fetch_all(app.pool())
    .await
    .unwrap();
    assert_eq!(
        queued_types,
        vec![
            "document.ai.embed",
            "document.ai.entities",
            "document.ai.summarize",
            "document.ai.tags",
        ]
    );
}

#[tokio::test]
async fn mila_journey_persists_config_presets_and_sessions_through_real_services() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);

    let initial = response(client.get("/api/v1/mila/config").await, StatusCode::OK).await;
    assert_eq!(initial["embedding_dim"], 768);
    assert_eq!(
        client
            .post_json("/api/v1/mila/config", &config(1536))
            .await
            .status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let saved = response(
        client.post_json("/api/v1/mila/config", &config(768)).await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(saved["chat_model"], "gpt-4.1-mini");
    assert_eq!(saved["enabled"], true);
    assert_eq!(
        client
            .post_json("/api/v1/mila/config/reindex", &config(768))
            .await
            .status(),
        StatusCode::OK
    );
    let status = response(client.get("/api/v1/mila/status").await, StatusCode::OK).await;
    assert_eq!(status["enabled"], true);

    let presets = response(client.get("/api/v1/mila/presets").await, StatusCode::OK).await;
    assert!(!presets["groups"].as_array().unwrap().is_empty());
    let preset = response(
        client
            .post_json(
                "/api/v1/mila/presets",
                &json!({
                    "action": "summary",
                    "name": "Research Summary",
                    "system_prompt": "Write as a research analyst.",
                    "is_default": true
                }),
            )
            .await,
        StatusCode::CREATED,
    )
    .await;
    let preset_id = preset["id"].as_str().unwrap();
    let updated = response(
        client
            .patch_json(
                &format!("/api/v1/mila/presets/{preset_id}"),
                &json!({"system_prompt": "Answer like a teacher.", "is_default": false}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(updated["system_prompt"], "Answer like a teacher.");
    assert_eq!(
        client
            .delete(&format!("/api/v1/mila/presets/{preset_id}"))
            .await
            .status(),
        StatusCode::NO_CONTENT
    );

    let document = DocumentFactory::new(session.user.id)
        .with_title("Mila Journey")
        .insert(app.pool())
        .await;
    let thread = response(
        client
            .post_json(
                "/api/v1/mila/sessions",
                &json!({"session_type": "single_document", "document_id": document.id}),
            )
            .await,
        StatusCode::CREATED,
    )
    .await;
    let thread_id = thread["id"].as_str().unwrap();
    let threads = response(client.get("/api/v1/mila/sessions").await, StatusCode::OK).await;
    assert_eq!(threads["sessions"][0]["id"], thread_id);
    let conversation = response(
        client
            .get(&format!("/api/v1/mila/sessions/{thread_id}/messages"))
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(conversation["session"]["id"], thread_id);
    assert!(conversation["messages"].as_array().unwrap().is_empty());
    assert_eq!(
        client
            .delete(&format!("/api/v1/mila/sessions/{thread_id}"))
            .await
            .status(),
        StatusCode::NO_CONTENT
    );
}
