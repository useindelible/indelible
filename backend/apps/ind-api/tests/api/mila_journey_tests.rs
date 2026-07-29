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
