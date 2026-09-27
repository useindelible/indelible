#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use ind_application::repos::epub_format::{EpubFormatIssue, EpubFormatRepository};
use ind_domain::{ArchiveAssetId, DocumentId, DocumentType};
use ind_persistence::repos::PgEpubFormatRepository;
use ind_test_support::{DocumentFactory, TestDb, UserFactory};
use sqlx::PgPool;
use tokio::sync::oneshot;

struct AssetRow<'a> {
    kind: &'a str,
    status: &'a str,
    version: i32,
    issue: Option<&'a str>,
}

const PENDING_EPUB: AssetRow = AssetRow {
    kind: "epub",
    status: "completed",
    version: 1,
    issue: None,
};

async fn insert_asset(pool: &PgPool, document_id: DocumentId, row: AssetRow<'_>) {
    sqlx::query!(
        "INSERT INTO archive_assets (id, document_id, asset_kind, s3_key, s3_bucket, \
             content_type, size_bytes, status, format_version, format_issue, created_at) \
         VALUES (gen_random_uuid(), $1, $2, $3, 'books', 'application/json', 1, $4, $5, $6, \
             now())",
        document_id.into_uuid(),
        row.kind,
        format!("{document_id}/{}", row.kind),
        row.status,
        row.version,
        row.issue,
    )
    .execute(pool)
    .await
    .unwrap();
}

async fn book(pool: &PgPool, toc: AssetRow<'_>, with_original: bool) -> DocumentId {
    let user = UserFactory::default().insert(pool).await;
    let document = DocumentFactory::new(user.id)
        .with_document_type(DocumentType::Book)
        .insert(pool)
        .await;
    insert_asset(pool, document.id, toc).await;
    if with_original {
        let original = AssetRow {
            kind: "original_upload",
            ..PENDING_EPUB
        };
        insert_asset(pool, document.id, original).await;
    }
    document.id
}

#[tokio::test]
async fn pending_failed_and_non_epub_rows_are_not_scanned() {
    let db = TestDb::new().await;
    let pool = db.pool();
    let with_original = book(pool, PENDING_EPUB, true).await;
    let without_original = book(pool, PENDING_EPUB, false).await;
    let skipped = [
        AssetRow {
            status: "pending",
            ..PENDING_EPUB
        },
        AssetRow {
            status: "failed",
            ..PENDING_EPUB
        },
        AssetRow {
            kind: "pdf",
            ..PENDING_EPUB
        },
        AssetRow {
            version: 2,
            ..PENDING_EPUB
        },
        AssetRow {
            issue: Some("missing_chapter"),
            ..PENDING_EPUB
        },
        AssetRow {
            issue: Some("needs_reupload"),
            ..PENDING_EPUB
        },
    ];
    let mut skipped_documents = Vec::new();
    for row in skipped {
        skipped_documents.push(book(pool, row, true).await);
    }
    let repo = PgEpubFormatRepository::new(pool.clone());

    let scanned = repo.epub_assets_pending_after(None, 10).await.unwrap();

    let mut documents: Vec<_> = scanned.iter().map(|asset| asset.document_id).collect();
    documents.sort_by_key(|id| id.into_uuid());
    let mut expected = vec![with_original, without_original];
    expected.sort_by_key(|id| id.into_uuid());
    assert_eq!(documents, expected);
    let original = |id| {
        scanned
            .iter()
            .find(|asset| asset.document_id == id)
            .unwrap()
    };
    let located = original(with_original).original.as_ref().unwrap();
    assert_eq!(located.s3_key, format!("{with_original}/original_upload"));
    assert!(original(without_original).original.is_none());

    let after_first = repo
        .epub_assets_pending_after(Some(scanned[0].asset_id), 10)
        .await
        .unwrap();
    assert_eq!(after_first, scanned[1..]);
    for document_id in skipped_documents {
        assert!(
            repo.pending_epub_asset(document_id)
                .await
                .unwrap()
                .is_none()
        );
    }
}

async fn wait_until_a_holder_blocks(pool: &PgPool) {
    let blocked = async {
        loop {
            let waiting = sqlx::query_scalar!(
                r#"SELECT count(*) AS "waiting!" FROM pg_locks
                   WHERE locktype = 'advisory' AND NOT granted
                   AND database = (SELECT oid FROM pg_database WHERE datname = current_database())"#,
            )
            .fetch_one(pool)
            .await
            .unwrap();
            if waiting > 0 {
                break;
            }
            tokio::task::yield_now().await;
        }
    };
    tokio::time::timeout(Duration::from_secs(10), blocked)
        .await
        .expect("the second holder never waited for the document lock");
}

#[tokio::test]
async fn marking_an_asset_that_no_longer_exists_is_an_error() {
    let db = TestDb::new().await;
    let repo = PgEpubFormatRepository::new(db.pool().clone());
    let gone = ArchiveAssetId::new();

    assert!(repo.mark_format(gone, 2, 1).await.is_err());
    assert!(
        repo.mark_format_issue(gone, EpubFormatIssue::MissingChapter)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn document_lock_serializes_two_holders() {
    let db = TestDb::new().await;
    let repo = Arc::new(PgEpubFormatRepository::new(db.pool().clone()));
    let document_id = DocumentId::new();
    let events = Arc::new(Mutex::new(Vec::new()));
    let (entered, first_entered) = oneshot::channel();
    let (release, released) = oneshot::channel::<()>();

    let first = tokio::spawn({
        let (repo, events) = (repo.clone(), events.clone());
        async move {
            let work = async move {
                events.lock().unwrap().push("first entered");
                entered.send(()).unwrap();
                released.await.unwrap();
                events.lock().unwrap().push("first left");
                Ok(())
            };
            repo.with_document_lock(document_id, Box::pin(work)).await
        }
    });
    first_entered.await.unwrap();
    let second = tokio::spawn({
        let (repo, events) = (repo.clone(), events.clone());
        async move {
            let work = async move {
                events.lock().unwrap().push("second entered");
                Ok(())
            };
            repo.with_document_lock(document_id, Box::pin(work)).await
        }
    });
    wait_until_a_holder_blocks(db.pool()).await;
    release.send(()).unwrap();
    first.await.unwrap().unwrap();
    second.await.unwrap().unwrap();

    assert_eq!(
        *events.lock().unwrap(),
        ["first entered", "first left", "second entered"]
    );
}

#[tokio::test]
async fn global_backfill_lock_admits_one_runner() {
    let db = TestDb::new().await;
    let repo = PgEpubFormatRepository::new(db.pool().clone());
    let second_ran = AtomicBool::new(false);

    let outer = async {
        let second = async {
            second_ran.store(true, Ordering::SeqCst);
            Ok(())
        };
        let admitted = repo.try_global_backfill_lock(Box::pin(second)).await?;
        assert!(!admitted);
        Ok(())
    };
    assert!(
        repo.try_global_backfill_lock(Box::pin(outer))
            .await
            .unwrap()
    );

    assert!(!second_ran.load(Ordering::SeqCst));
    let released = repo.try_global_backfill_lock(Box::pin(async { Ok(()) }));
    assert!(released.await.unwrap());
}
