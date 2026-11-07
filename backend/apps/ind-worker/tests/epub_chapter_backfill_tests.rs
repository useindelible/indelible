#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::collections::BTreeMap;
use std::sync::Arc;

use bytes::Bytes;
use ind_application::LibraryUploadService;
use ind_application::ports::UploadFileRequest;
use ind_application::repos::epub_format::EpubFormatIssue;
use ind_application::storage::{ObjectStorage, get_object_bytes};
use ind_domain::{ArchiveAssetKind, DocumentId, ReprocessDocumentJob, UserId};
use ind_ingest::DocumentFileUploadProcessor;
use ind_persistence::repos::{
    PgDocumentAssetRepository, PgDocumentRepository, PgDocumentUploadRepository,
};
use ind_test_support::{
    TestDb, UserFactory, build_epub, document_asset, epub_format, set_epub_format,
};
use ind_worker::context::WorkerContext;
use ind_worker::jobs::backfill::run_epub_chapter_list_backfill;
use ind_worker::jobs::reprocess::handle_document_reprocess;
use serde_json::{Value, json};

struct AgedBook {
    document_id: DocumentId,
    user_id: UserId,
    toc_key: String,
    stored_toc: Value,
    chapters: BTreeMap<String, Bytes>,
}

async fn object(storage: &Arc<dyn ObjectStorage>, key: &str) -> Bytes {
    Bytes::from(get_object_bytes(storage.as_ref(), key).await.unwrap())
}

/// Uploads a book, then rewrites its TOC the way an older parser stored it: no chapter list,
/// different entry ids and titles, and metadata this parser no longer writes.
async fn aged_book(db: &TestDb, storage: &Arc<dyn ObjectStorage>) -> AgedBook {
    let pool = db.pool().clone();
    let user = UserFactory::default().insert(&pool).await;
    let uploads = LibraryUploadService::new(
        Arc::new(DocumentFileUploadProcessor),
        storage.clone(),
        Arc::new(PgDocumentUploadRepository::new(pool.clone())),
        Arc::new(PgDocumentRepository::new(pool.clone())),
        Arc::new(PgDocumentAssetRepository::new(pool.clone())),
    );
    let data = build_epub("Aged", &[("One", "First."), ("Two", "Second.")]);
    let saved = uploads
        .upload_file(
            user.id,
            UploadFileRequest {
                filename: "aged.epub".into(),
                content_type: "application/epub+zip".into(),
                data: Bytes::from(data),
                title_override: None,
                max_bytes: 1 << 20,
                asset_base_url: "http://localhost".into(),
            },
        )
        .await
        .unwrap();
    let document_id = saved.document.id;
    set_epub_format(&pool, document_id, 1, None).await;
    let toc_key = document_asset(&pool, document_id, ArchiveAssetKind::Epub)
        .await
        .s3_key;

    let mut stored_toc: Value = serde_json::from_slice(&object(storage, &toc_key).await).unwrap();
    let metadata = stored_toc["metadata"].as_object_mut().unwrap();
    metadata.remove("chapters");
    metadata.insert("title".into(), json!("Title From An Older Parser"));
    metadata.insert("chapter_indices".into(), json!([0, 1]));
    for (index, entry) in stored_toc["toc"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .enumerate()
    {
        entry["id"] = json!(format!("legacy-{index}"));
        entry["title"] = json!(format!("Legacy title {index}"));
    }
    let bytes = Bytes::from(serde_json::to_vec(&stored_toc).unwrap());
    storage
        .upload(&toc_key, "application/json", bytes)
        .await
        .unwrap();

    let prefix = toc_key.strip_suffix("epub_toc.json").unwrap();
    let mut chapters = BTreeMap::new();
    for key in storage.list_keys(prefix).await.unwrap() {
        if key.contains("epub_ch_") {
            chapters.insert(key.clone(), object(storage, &key).await);
        }
    }
    assert_eq!(chapters.len(), 2);
    AgedBook {
        document_id,
        user_id: user.id,
        toc_key,
        stored_toc,
        chapters,
    }
}

async fn setup() -> (TestDb, WorkerContext, Arc<dyn ObjectStorage>) {
    let db = TestDb::new().await;
    let ctx = common::build_worker_ctx(&db).await;
    let storage = ctx.object_storage.clone().unwrap();
    (db, ctx, storage)
}

async fn backfill(ctx: &WorkerContext, storage: &Arc<dyn ObjectStorage>) -> (u64, u64) {
    let stats = run_epub_chapter_list_backfill(ctx.epub_format_repo.clone(), storage.clone())
        .await
        .unwrap()
        .expect("no other worker holds the backfill lock");
    (stats.upgraded, stats.already_current + stats.left_pending)
}

#[tokio::test]
async fn backfill_adds_only_the_chapter_list() {
    let (db, ctx, storage) = setup().await;
    let book = aged_book(&db, &storage).await;

    assert_eq!(backfill(&ctx, &storage).await, (1, 0));

    let mut expected = book.stored_toc.clone();
    expected["metadata"]["chapters"] = json!([
        { "spine_index": 0, "href": "ch0.xhtml", "title": "One" },
        { "spine_index": 1, "href": "ch1.xhtml", "title": "Two" },
    ]);
    let patched: Value = serde_json::from_slice(&object(&storage, &book.toc_key).await).unwrap();
    assert_eq!(patched, expected);
    for (key, bytes) in &book.chapters {
        assert_eq!(
            &object(&storage, key).await,
            bytes,
            "{key} must not be rewritten"
        );
    }
    assert_eq!(epub_format(db.pool(), book.document_id).await, (2, None));
}

#[tokio::test]
async fn backfill_is_idempotent() {
    let (db, ctx, storage) = setup().await;
    let book = aged_book(&db, &storage).await;
    backfill(&ctx, &storage).await;
    let after_first = object(&storage, &book.toc_key).await;

    assert_eq!(backfill(&ctx, &storage).await, (0, 0));

    assert_eq!(object(&storage, &book.toc_key).await, after_first);
    assert_eq!(epub_format(db.pool(), book.document_id).await, (2, None));
}

async fn reprocess(ctx: &WorkerContext, book: &AgedBook) -> Result<(), ind_application::AppError> {
    let job = ReprocessDocumentJob {
        document_id: book.document_id,
        user_id: book.user_id,
    };
    handle_document_reprocess(&ctx.capture_jobs(), job).await
}

#[tokio::test]
async fn epub_reprocess_success_is_version_2() {
    let (db, ctx, storage) = setup().await;
    let book = aged_book(&db, &storage).await;
    let missing = Some(EpubFormatIssue::MissingChapter);
    set_epub_format(db.pool(), book.document_id, 1, missing).await;

    reprocess(&ctx, &book).await.unwrap();

    assert_eq!(epub_format(db.pool(), book.document_id).await, (2, None));
}

#[tokio::test]
async fn failed_epub_reprocess_keeps_the_book_needing_reprocess() {
    let (db, ctx, storage) = setup().await;
    let book = aged_book(&db, &storage).await;
    let missing = Some(EpubFormatIssue::MissingChapter);
    set_epub_format(db.pool(), book.document_id, 1, missing).await;
    let original = document_asset(
        db.pool(),
        book.document_id,
        ArchiveAssetKind::OriginalUpload,
    );
    let corrupt = Bytes::from_static(b"PK\x03\x04 not an epub");
    storage
        .upload(&original.await.s3_key, "application/epub+zip", corrupt)
        .await
        .unwrap();

    assert!(reprocess(&ctx, &book).await.is_err());

    assert_eq!(epub_format(db.pool(), book.document_id).await, (1, missing));
}
