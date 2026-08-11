use ind_application::repos::document_asset::DocumentAssetRepository;
use ind_domain::{ArchiveAssetKind, ArchiveAssetStatus, Document, DocumentType, NewDocumentAsset};
use ind_persistence::repos::PgDocumentAssetRepository;
use ind_test_support::{AuthedClient, DocumentFactory, TestApp, spawn_app};
use reqwest::StatusCode;
use serde_json::json;

use super::common::{assert_json_response, assert_status};

#[tokio::test]
async fn annotations_require_the_completed_canonical_asset_for_each_document_format() {
    let app = spawn_app().await;
    let owner = app.create_web_session().await;
    let stranger = app.create_web_session().await;
    let owner_client = app.authed_client(&owner);
    let stranger_client = app.authed_client(&stranger);

    for (document_type, asset_kind, locator) in [
        (
            DocumentType::Article,
            ArchiveAssetKind::ReadableHtml,
            json!({"type": "html", "start_offset": 0, "end_offset": 8}),
        ),
        (
            DocumentType::Book,
            ArchiveAssetKind::Epub,
            json!({"type": "epub", "chapter": "chapter-1", "start_offset": 0, "end_offset": 8}),
        ),
        (
            DocumentType::Pdf,
            ArchiveAssetKind::Pdf,
            json!({
                "type": "pdf",
                "page": 1,
                "x": 0.1,
                "y": 0.1,
                "width": 0.2,
                "height": 0.1,
                "text_snapshot": "selected"
            }),
        ),
    ] {
        let document = DocumentFactory::new(owner.user.id)
            .with_document_type(document_type)
            .insert(app.pool())
            .await;
        insert_asset(&app, &document, asset_kind, ArchiveAssetStatus::Completed).await;

        let highlight = assert_json_response(
            owner_client
                .post_json(
                    &format!("/api/v1/documents/{}/highlights", document.id),
                    &json!({
                        "color": "yellow",
                        "text_content": "selected",
                        "locator": locator
                    }),
                )
                .await,
            StatusCode::CREATED,
        )
        .await;
        assert_eq!(highlight["text_content"], "selected");

        let note = assert_json_response(
            owner_client
                .put_json(
                    &format!("/api/v1/documents/{}/note", document.id),
                    &json!({"body": "format-specific note"}),
                )
                .await,
            StatusCode::OK,
        )
        .await;
        assert_eq!(note["body"], "format-specific note");
    }

    for document_type in [DocumentType::Article, DocumentType::Book, DocumentType::Pdf] {
        let missing = DocumentFactory::new(owner.user.id)
            .with_document_type(document_type)
            .insert(app.pool())
            .await;
        assert_annotation_writes_rejected(&owner_client, &missing).await;

        let failed = DocumentFactory::new(owner.user.id)
            .with_document_type(document_type)
            .insert(app.pool())
            .await;
        insert_asset(
            &app,
            &failed,
            canonical_asset(document_type),
            ArchiveAssetStatus::Failed,
        )
        .await;
        assert_annotation_writes_rejected(&owner_client, &failed).await;
    }

    let pdf = DocumentFactory::new(owner.user.id)
        .with_document_type(DocumentType::Pdf)
        .insert(app.pool())
        .await;
    insert_asset(
        &app,
        &pdf,
        ArchiveAssetKind::Pdf,
        ArchiveAssetStatus::Completed,
    )
    .await;
    assert_status(
        owner_client
            .post_json(
                &format!("/api/v1/documents/{}/highlights", pdf.id),
                &json!({
                    "color": "yellow",
                    "text_content": "wrong locator",
                    "locator": {
                        "type": "epub",
                        "chapter": "chapter-1",
                        "start_offset": 0,
                        "end_offset": 5
                    }
                }),
            )
            .await,
        StatusCode::UNPROCESSABLE_ENTITY,
    )
    .await;
    assert_status(
        stranger_client
            .put_json(
                &format!("/api/v1/documents/{}/note", pdf.id),
                &json!({"body": "not mine"}),
            )
            .await,
        StatusCode::NOT_FOUND,
    )
    .await;
}

fn canonical_asset(document_type: DocumentType) -> ArchiveAssetKind {
    match document_type {
        DocumentType::Book => ArchiveAssetKind::Epub,
        DocumentType::Pdf => ArchiveAssetKind::Pdf,
        _ => ArchiveAssetKind::ReadableHtml,
    }
}

async fn insert_asset(
    app: &TestApp,
    document: &Document,
    asset_kind: ArchiveAssetKind,
    status: ArchiveAssetStatus,
) {
    let failed_reason = (status == ArchiveAssetStatus::Failed).then(|| "test failure".into());
    PgDocumentAssetRepository::new(app.pool().clone())
        .upsert_document_asset(NewDocumentAsset {
            document_id: document.id,
            asset_kind,
            s3_key: format!("tests/annotations/{}/{}", document.id, asset_kind),
            s3_bucket: "test-bucket".into(),
            content_type: match asset_kind {
                ArchiveAssetKind::Epub => "application/epub+zip",
                ArchiveAssetKind::Pdf => "application/pdf",
                _ => "text/html",
            }
            .into(),
            size_bytes: 64,
            status,
            failed_reason,
        })
        .await
        .unwrap();
}

async fn assert_annotation_writes_rejected(client: &AuthedClient<'_>, document: &Document) {
    let locator = match document.document_type {
        DocumentType::Book => {
            json!({"type": "epub", "chapter": "chapter-1", "start_offset": 0, "end_offset": 8})
        }
        DocumentType::Pdf => json!({
            "type": "pdf",
            "page": 1,
            "x": 0.1,
            "y": 0.1,
            "width": 0.2,
            "height": 0.1,
            "text_snapshot": "unanchored"
        }),
        _ => json!({"type": "html", "start_offset": 0, "end_offset": 8}),
    };
    for response in [
        client
            .post_json(
                &format!("/api/v1/documents/{}/highlights", document.id),
                &json!({
                    "color": "yellow",
                    "text_content": "unanchored",
                    "locator": locator
                }),
            )
            .await,
        client
            .put_json(
                &format!("/api/v1/documents/{}/note", document.id),
                &json!({"body": "unanchored"}),
            )
            .await,
    ] {
        let body = assert_json_response(response, StatusCode::UNPROCESSABLE_ENTITY).await;
        assert_eq!(
            body["errors"][0]["message"],
            "document has no completed reader content yet; prepare it before highlighting or noting"
        );
    }
}
