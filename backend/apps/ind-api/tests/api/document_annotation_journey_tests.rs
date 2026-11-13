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
        let color = match document_type {
            DocumentType::Book | DocumentType::Pdf => "bookmark",
            _ => "yellow",
        };
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
                        "color": color,
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

#[tokio::test]
async fn a_note_write_on_a_stale_base_is_refused_with_the_current_note() {
    let app = spawn_app().await;
    let owner = app.create_web_session().await;
    let client = app.authed_client(&owner);
    let document = annotatable_article(&app, &owner).await;
    let path = format!("/api/v1/documents/{}/note", document.id);

    let first = assert_json_response(
        client.put_json(&path, &json!({"body": "first"})).await,
        StatusCode::OK,
    )
    .await;
    let theirs = assert_json_response(
        client.put_json(&path, &json!({"body": "theirs"})).await,
        StatusCode::OK,
    )
    .await;

    let stale = json!({"body": "mine", "base_updated_at": first["updated_at"]});
    let refused = client.put_json(&path, &stale).await;
    let content_type = refused.headers()[reqwest::header::CONTENT_TYPE]
        .to_str()
        .unwrap()
        .to_owned();
    let conflict = assert_json_response(refused, StatusCode::CONFLICT).await;
    assert!(
        content_type.starts_with("application/problem+json"),
        "{content_type}"
    );
    assert_eq!(
        conflict["type"],
        "https://indelible.app/problems/note-conflict"
    );
    assert_eq!(conflict["title"], "Conflict");
    assert_eq!(conflict["code"], "note_conflict");
    assert_eq!(conflict["status"], 409);
    assert_eq!(conflict["note"]["body"], "theirs");
    assert_eq!(conflict["note"]["updated_at"], theirs["updated_at"]);
    let stored = assert_json_response(client.get(&path).await, StatusCode::OK).await;
    assert_eq!(stored["body"], "theirs");

    let forced = json!({"body": "mine", "base_updated_at": first["updated_at"], "force": true});
    assert_json_response(client.put_json(&path, &forced).await, StatusCode::OK).await;
    let stored = assert_json_response(client.get(&path).await, StatusCode::OK).await;
    assert_eq!(stored["body"], "mine");
}

#[tokio::test]
async fn a_note_write_on_the_seen_base_is_applied() {
    let app = spawn_app().await;
    let owner = app.create_web_session().await;
    let client = app.authed_client(&owner);
    let document = annotatable_article(&app, &owner).await;
    let path = format!("/api/v1/documents/{}/note", document.id);

    let first = assert_json_response(
        client.put_json(&path, &json!({"body": "first"})).await,
        StatusCode::OK,
    )
    .await;
    let seen = json!({"body": "second", "base_updated_at": first["updated_at"]});
    let second = assert_json_response(client.put_json(&path, &seen).await, StatusCode::OK).await;
    assert_eq!(second["body"], "second");

    let epoch = json!({"body": "third", "base_updated_at": "1970-01-01T00:00:00Z"});
    assert_status(client.put_json(&path, &epoch).await, StatusCode::CONFLICT).await;
}

async fn annotatable_article(app: &TestApp, owner: &ind_test_support::TestAuthSession) -> Document {
    let document = DocumentFactory::new(owner.user.id)
        .with_document_type(DocumentType::Article)
        .insert(app.pool())
        .await;
    insert_asset(
        app,
        &document,
        ArchiveAssetKind::ReadableHtml,
        ArchiveAssetStatus::Completed,
    )
    .await;
    document
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
