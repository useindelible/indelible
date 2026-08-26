use ind_application::repos::document_asset::DocumentAssetRepository;
use ind_domain::{ArchiveAssetKind, ArchiveAssetStatus, DocumentType, NewDocumentAsset};
use ind_persistence::repos::PgDocumentAssetRepository;
use ind_test_support::{DocumentFactory, spawn_app};
use reqwest::StatusCode;
use serde_json::json;

use super::common::{assert_json_response, assert_status};

#[tokio::test]
async fn text_quote_context_round_trips_through_create_and_list() {
    let app = spawn_app().await;
    let owner = app.create_web_session().await;
    let client = app.authed_client(&owner);
    let document = DocumentFactory::new(owner.user.id)
        .with_document_type(DocumentType::Article)
        .insert(app.pool())
        .await;
    PgDocumentAssetRepository::new(app.pool().clone())
        .upsert_document_asset(NewDocumentAsset {
            document_id: document.id,
            asset_kind: ArchiveAssetKind::ReadableHtml,
            s3_key: format!("tests/text-quote/{}", document.id),
            s3_bucket: "test-bucket".into(),
            content_type: "text/html".into(),
            size_bytes: 64,
            status: ArchiveAssetStatus::Completed,
            failed_reason: None,
        })
        .await
        .unwrap();
    let path = format!("/api/v1/documents/{}/highlights", document.id);

    let created = assert_json_response(
        client
            .post_json(
                &path,
                &json!({
                    "color": "yellow",
                    "text_content": "Beta target",
                    "locator": { "type": "html", "start_offset": 14, "end_offset": 25 },
                    "source_locator": { "type": "text_quote", "prefix": "phrase. ", "suffix": "." }
                }),
            )
            .await,
        StatusCode::CREATED,
    )
    .await;
    assert_eq!(created["source_locator"]["type"], "text_quote");

    let listed = assert_json_response(client.get(&path).await, StatusCode::OK).await;
    let stored = &listed["highlights"][0]["source_locator"];
    assert_eq!(stored["type"], "text_quote");
    assert_eq!(stored["prefix"], "phrase. ");
    assert_eq!(stored["suffix"], ".");

    assert_status(
        client
            .post_json(
                &path,
                &json!({
                    "color": "yellow",
                    "text_content": "Beta target",
                    "locator": { "type": "html", "start_offset": 14, "end_offset": 25 },
                    "source_locator": { "type": "text_quote" }
                }),
            )
            .await,
        StatusCode::UNPROCESSABLE_ENTITY,
    )
    .await;
}
