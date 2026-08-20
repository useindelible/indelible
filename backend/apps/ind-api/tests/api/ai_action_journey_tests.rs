use ind_domain::{
    DocumentId, ExtractEntitiesDocumentJob, SuggestTagsDocumentJob, SummarizeDocumentJob, job_types,
};
use serde_json::{Value, json};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::common::{
    SaveScenario, build_worker_context, configure_mila, dispatch_ai_job, document_id_from_response,
    mount_mila_completion,
};

#[tokio::test]
async fn ai_actions_cross_http_worker_storage_and_persistence_boundaries() {
    let provider = MockServer::start().await;
    mount_mila_completion(
        &provider,
        "summary",
        json!({"summary": "A concise systems article."}),
    )
    .await;
    mount_mila_completion(
        &provider,
        "tags",
        json!({"tags": [" Rust ", "systems", "rust"]}),
    )
    .await;
    mount_mila_completion(
        &provider,
        "entities",
        json!({"entities": [{
            "name": "Rust Foundation",
            "entity_type": "organization",
            "description": "Stewards Rust",
            "mention_count": 2,
            "aliases": ["RF"]
        }]}),
    )
    .await;

    let scenario = SaveScenario::new().await;
    let saved = scenario
        .extension_reader_save("https://example.com/ai-action-boundary")
        .await;
    let document_id: DocumentId = document_id_from_response(&saved).parse().unwrap();
    configure_mila(&scenario, &provider.uri()).await;

    let context = build_worker_context(&scenario.app);
    assert_eq!(
        dispatch_ai_job(
            &context,
            job_types::DOCUMENT_AI_SUMMARIZE,
            serde_json::to_value(SummarizeDocumentJob { document_id }).unwrap(),
        )
        .await
        .unwrap(),
        Some(())
    );
    dispatch_ai_job(
        &context,
        job_types::DOCUMENT_AI_TAGS,
        serde_json::to_value(SuggestTagsDocumentJob { document_id }).unwrap(),
    )
    .await
    .unwrap();
    dispatch_ai_job(
        &context,
        job_types::DOCUMENT_AI_ENTITIES,
        serde_json::to_value(ExtractEntitiesDocumentJob { document_id }).unwrap(),
    )
    .await
    .unwrap();

    let outputs: Vec<(String, Value)> = sqlx::query_as(
        "SELECT output_type, content FROM ai_outputs WHERE document_id = $1 ORDER BY output_type",
    )
    .bind(document_id.into_uuid())
    .fetch_all(scenario.app.pool())
    .await
    .unwrap();
    assert_eq!(outputs.len(), 3);
    assert_eq!(
        outputs[0],
        (
            "entities".into(),
            json!([{
                "name": "Rust Foundation",
                "entity_type": "organization",
                "description": "Stewards Rust",
                "mention_count": 2,
                "aliases": ["RF"]
            }])
        )
    );
    assert_eq!(
        outputs[1],
        ("summary".into(), json!("A concise systems article."))
    );
    assert_eq!(outputs[2], ("tags".into(), json!(["rust", "systems"])));

    let completed_runs: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM ai_runs WHERE document_id = $1 AND status = 'completed' \
         AND input_tokens = 21 AND output_tokens = 8",
    )
    .bind(document_id.into_uuid())
    .fetch_one(scenario.app.pool())
    .await
    .unwrap();
    assert_eq!(completed_runs, 3);
    let completed_events: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM domain_events WHERE aggregate_id = $1 AND event_type = 'ai.output.completed'",
    )
    .bind(document_id.into_uuid())
    .fetch_one(scenario.app.pool())
    .await
    .unwrap();
    assert_eq!(completed_events, 3);
    let entity: (String, i32) = sqlx::query_as(
        "SELECT e.name, m.mention_count FROM entities e \
         JOIN entity_mentions m ON m.entity_id = e.id \
         WHERE m.document_id = $1",
    )
    .bind(document_id.into_uuid())
    .fetch_one(scenario.app.pool())
    .await
    .unwrap();
    assert_eq!(entity, ("Rust Foundation".into(), 2));

    provider.reset().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(503).set_body_json(json!({
            "error": {"message": "provider unavailable"}
        })))
        .expect(1)
        .mount(&provider)
        .await;
    let failure = dispatch_ai_job(
        &context,
        job_types::DOCUMENT_AI_SUMMARIZE,
        serde_json::to_value(SummarizeDocumentJob { document_id }).unwrap(),
    )
    .await;
    assert!(failure.is_err());
    let failed: (i64, i64) = sqlx::query_as(
        "SELECT \
           count(*) FILTER (WHERE status = 'failed'), \
           count(*) FILTER (WHERE status = 'failed' AND error_message LIKE '%provider unavailable%') \
         FROM ai_runs WHERE document_id = $1",
    )
    .bind(document_id.into_uuid())
    .fetch_one(scenario.app.pool())
    .await
    .unwrap();
    assert_eq!(failed, (1, 1));
    let failed_events: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM domain_events WHERE aggregate_id = $1 AND event_type = 'ai.output.failed'",
    )
    .bind(document_id.into_uuid())
    .fetch_one(scenario.app.pool())
    .await
    .unwrap();
    assert_eq!(failed_events, 1);
}
