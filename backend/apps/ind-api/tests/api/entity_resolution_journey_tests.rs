use ind_domain::{DocumentId, ExtractEntitiesDocumentJob, job_types};
use serde_json::json;
use wiremock::MockServer;

use super::common::{
    SaveScenario, build_worker_context, configure_mila, dispatch_ai_job, document_id_from_response,
    mount_mila_completion,
};

fn deepseek(entity_type: &str, description: &str) -> serde_json::Value {
    json!({"entities": [{
        "name": "DeepSeek",
        "entity_type": entity_type,
        "description": description,
        "mention_count": 1,
        "aliases": []
    }]})
}

#[tokio::test]
async fn same_name_entity_extracted_under_another_type_merges_into_the_existing_one() {
    let provider = MockServer::start().await;
    let scenario = SaveScenario::new().await;
    configure_mila(&scenario, &provider.uri()).await;
    let context = build_worker_context(&scenario.app);
    let user_id = scenario.web.user.id.into_uuid();

    mount_mila_completion(
        &provider,
        "entities",
        deepseek(
            "organization",
            "Chinese AI company that released DeepSeek-R1",
        ),
    )
    .await;
    let first = scenario
        .extension_reader_save("https://example.com/deepseek-company")
        .await;
    let first_id: DocumentId = document_id_from_response(&first).parse().unwrap();
    dispatch_ai_job(
        &context,
        job_types::DOCUMENT_AI_ENTITIES,
        serde_json::to_value(ExtractEntitiesDocumentJob {
            document_id: first_id,
        })
        .unwrap(),
    )
    .await
    .unwrap();

    provider.reset().await;
    // Same referent (the company), drifted type label — the unambiguous case the fix targets.
    mount_mila_completion(
        &provider,
        "entities",
        deepseek("work", "Chinese AI lab behind the R1 and V3 models"),
    )
    .await;
    mount_mila_completion(
        &provider,
        "entity_resolution",
        json!({"results": [{"entity": 1, "match": 1, "confidence": 0.95}]}),
    )
    .await;
    let second = scenario
        .extension_reader_save("https://example.com/deepseek-models")
        .await;
    let second_id: DocumentId = document_id_from_response(&second).parse().unwrap();
    dispatch_ai_job(
        &context,
        job_types::DOCUMENT_AI_ENTITIES,
        serde_json::to_value(ExtractEntitiesDocumentJob {
            document_id: second_id,
        })
        .unwrap(),
    )
    .await
    .unwrap();

    let entities: Vec<(uuid::Uuid, String)> = sqlx::query_as(
        "SELECT id, entity_type FROM entities WHERE user_id = $1 AND name = 'DeepSeek'",
    )
    .bind(user_id)
    .fetch_all(scenario.app.pool())
    .await
    .unwrap();
    assert_eq!(
        entities.len(),
        1,
        "one DeepSeek entity expected, got {entities:?}"
    );
    let (entity_id, entity_type) = &entities[0];
    assert_eq!(entity_type, "organization");

    let mention_docs: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM entity_mentions WHERE entity_id = $1 AND document_id = ANY($2)",
    )
    .bind(entity_id)
    .bind(vec![first_id.into_uuid(), second_id.into_uuid()])
    .fetch_one(scenario.app.pool())
    .await
    .unwrap();
    assert_eq!(mention_docs, 2);

    let alias_target: Option<uuid::Uuid> = sqlx::query_scalar(
        "SELECT entity_id FROM entity_aliases \
         WHERE user_id = $1 AND entity_type = 'work' AND name = 'DeepSeek'",
    )
    .bind(user_id)
    .fetch_optional(scenario.app.pool())
    .await
    .unwrap();
    assert_eq!(alias_target.as_ref(), Some(entity_id));
}
