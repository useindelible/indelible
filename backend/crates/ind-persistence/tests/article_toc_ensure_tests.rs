#![allow(clippy::unwrap_used)]

use ind_application::handlers::article_toc::{
    EnsureOutcome, StoredArticleToc, article_toc_key, ensure_article_toc, prepared_readable_key,
};
use ind_application::repos::document_asset::DocumentAssetRepository;
use ind_application::storage::get_object_string;
use ind_domain::{ArchiveAssetKind, ArchiveAssetStatus, DocumentId, NewDocumentAsset};
use ind_html::{ArticleTocStatus, prepare_reader_html};
use ind_persistence::repos::{PgDocumentAssetRepository, PgDocumentRepository};
use ind_test_support::{DocumentFactory, TestDb, UserFactory};

const LEGACY_HTML: &str =
    "<h2>History</h2><p>one two three</p><h2>Structure</h2><p>four five six seven</p>";

fn readable_asset(document_id: DocumentId, key: &str, bucket: &str) -> NewDocumentAsset {
    NewDocumentAsset {
        document_id,
        asset_kind: ArchiveAssetKind::ReadableHtml,
        s3_key: key.to_string(),
        s3_bucket: bucket.to_string(),
        content_type: "text/html".to_string(),
        size_bytes: 100,
        status: ArchiveAssetStatus::Completed,
        failed_reason: None,
    }
}

#[tokio::test]
async fn unprepared_content_swaps_pointer_and_commits_toc() {
    let db = TestDb::new().await;
    let pool = db.pool().clone();
    let storage = db.storage().await;
    let user = UserFactory::default().insert(&pool).await;
    let document = DocumentFactory::new(user.id)
        .with_title("T")
        .insert(&pool)
        .await;
    let assets = PgDocumentAssetRepository::new(pool.clone());
    let documents = PgDocumentRepository::new(pool.clone());

    let legacy_key = format!("legacy/{}/readable_html.html", document.id.into_uuid());
    storage
        .upload(
            &legacy_key,
            "text/html",
            LEGACY_HTML.as_bytes().to_vec().into(),
        )
        .await
        .unwrap();
    let readable = assets
        .upsert_document_asset(readable_asset(document.id, &legacy_key, db.bucket()))
        .await
        .unwrap();

    let outcome = ensure_article_toc(storage.as_ref(), &assets, &documents, document.id)
        .await
        .unwrap();
    let EnsureOutcome::Committed(stored) = outcome else {
        panic!("expected commit, got {outcome:?}");
    };
    assert_eq!(stored.toc.status, ArticleTocStatus::Ready);
    assert_eq!(stored.toc.entries.len(), 2);
    assert_eq!(stored.source.readable_created_at, readable.created_at);

    // Pointer swapped to the prepared immutable copy; stamp untouched.
    let readable_after = assets
        .find_by_document_and_kind(document.id, ArchiveAssetKind::ReadableHtml)
        .await
        .unwrap()
        .unwrap();
    assert_ne!(readable_after.s3_key, legacy_key);
    assert!(readable_after.s3_key.starts_with("documents/prepared/"));
    assert_eq!(readable_after.created_at, readable.created_at);
    let prepared = get_object_string(storage.as_ref(), &readable_after.s3_key)
        .await
        .unwrap();
    assert!(prepared.contains(r#"id="ind-toc-history""#));

    // Stored payload parses and matches what the commit reported.
    let toc_row = assets
        .find_by_document_and_kind(document.id, ArchiveAssetKind::ArticleToc)
        .await
        .unwrap()
        .unwrap();
    let payload = get_object_string(storage.as_ref(), &toc_row.s3_key)
        .await
        .unwrap();
    let parsed: StoredArticleToc = serde_json::from_str(&payload).unwrap();
    assert_eq!(parsed, stored);
}

#[tokio::test]
async fn prepared_content_commits_without_swap() {
    let db = TestDb::new().await;
    let pool = db.pool().clone();
    let storage = db.storage().await;
    let user = UserFactory::default().insert(&pool).await;
    let document = DocumentFactory::new(user.id)
        .with_title("T")
        .insert(&pool)
        .await;
    let assets = PgDocumentAssetRepository::new(pool.clone());
    let documents = PgDocumentRepository::new(pool.clone());

    let prepared = prepare_reader_html(LEGACY_HTML).unwrap();
    let key = format!("already/{}/readable_html.html", document.id.into_uuid());
    storage
        .upload(&key, "text/html", prepared.into_bytes().into())
        .await
        .unwrap();
    assets
        .upsert_document_asset(readable_asset(document.id, &key, db.bucket()))
        .await
        .unwrap();

    let outcome = ensure_article_toc(storage.as_ref(), &assets, &documents, document.id)
        .await
        .unwrap();
    assert!(matches!(outcome, EnsureOutcome::Committed(_)));

    let readable_after = assets
        .find_by_document_and_kind(document.id, ArchiveAssetKind::ReadableHtml)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(readable_after.s3_key, key, "no swap for prepared content");
}

#[tokio::test]
async fn missing_readable_reports_no_readable_html() {
    let db = TestDb::new().await;
    let pool = db.pool().clone();
    let storage = db.storage().await;
    let user = UserFactory::default().insert(&pool).await;
    let document = DocumentFactory::new(user.id).insert(&pool).await;
    let assets = PgDocumentAssetRepository::new(pool.clone());
    let documents = PgDocumentRepository::new(pool.clone());

    let outcome = ensure_article_toc(storage.as_ref(), &assets, &documents, document.id)
        .await
        .unwrap();
    assert!(matches!(outcome, EnsureOutcome::NoReadableHtml));
}

/// The race surface spans three boundaries: object upload, SQL CAS, and loser
/// cleanup. Worker A pauses after staging its uploads; a reprocess bumps the
/// readable version and worker B commits; A's resumed commit must lose, and
/// B's bytes must survive untouched (no deletes anywhere).
#[tokio::test]
async fn resumed_stale_worker_loses_cas_and_winner_bytes_survive() {
    let db = TestDb::new().await;
    let pool = db.pool().clone();
    let storage = db.storage().await;
    let user = UserFactory::default().insert(&pool).await;
    let document = DocumentFactory::new(user.id)
        .with_title("T")
        .insert(&pool)
        .await;
    let assets = PgDocumentAssetRepository::new(pool.clone());
    let documents = PgDocumentRepository::new(pool.clone());

    let stable_key = format!("legacy/{}/readable_html.html", document.id.into_uuid());
    storage
        .upload(
            &stable_key,
            "text/html",
            LEGACY_HTML.as_bytes().to_vec().into(),
        )
        .await
        .unwrap();
    let observed = assets
        .upsert_document_asset(readable_asset(document.id, &stable_key, db.bucket()))
        .await
        .unwrap();

    // Worker A: observe v1, stage prepared copy + payload, then pause pre-commit.
    let a_prepared = prepare_reader_html(LEGACY_HTML).unwrap();
    let a_prepared_key = prepared_readable_key(user.id, &a_prepared);
    storage
        .upload(&a_prepared_key, "text/html", a_prepared.into_bytes().into())
        .await
        .unwrap();
    let a_payload = br#"{"stale":"payload"}"#.to_vec();
    let a_toc_key = article_toc_key(user.id, &a_payload);
    storage
        .upload(&a_toc_key, "application/json", a_payload.into())
        .await
        .unwrap();

    // Reprocess: new content lands on the same stable key, stamp refreshes.
    let newer_html = format!("{LEGACY_HTML}<h2>Appendix</h2><p>eight</p>");
    storage
        .upload(&stable_key, "text/html", newer_html.into_bytes().into())
        .await
        .unwrap();
    let bumped = assets
        .upsert_document_asset(readable_asset(document.id, &stable_key, db.bucket()))
        .await
        .unwrap();
    assert_ne!(bumped.created_at, observed.created_at);

    // Worker B: full ensure on the newer version.
    let outcome = ensure_article_toc(storage.as_ref(), &assets, &documents, document.id)
        .await
        .unwrap();
    let EnsureOutcome::Committed(b_stored) = outcome else {
        panic!("worker B should commit, got {outcome:?}");
    };
    assert_eq!(b_stored.toc.entries.len(), 3);
    let b_toc_row = assets
        .find_by_document_and_kind(document.id, ArchiveAssetKind::ArticleToc)
        .await
        .unwrap()
        .unwrap();

    // Worker A resumes: its commit against the stale stamp must be rejected.
    let a_committed = assets
        .commit_article_toc(
            document.id,
            observed.created_at,
            Some(
                ind_application::repos::document_asset::PreparedReadableLocation {
                    s3_key: a_prepared_key.clone(),
                    size_bytes: 1,
                },
            ),
            NewDocumentAsset {
                document_id: document.id,
                asset_kind: ArchiveAssetKind::ArticleToc,
                s3_key: a_toc_key.clone(),
                s3_bucket: db.bucket().to_string(),
                content_type: "application/json".to_string(),
                size_bytes: a_payload_len(),
                status: ArchiveAssetStatus::Completed,
                failed_reason: None,
            },
        )
        .await
        .unwrap();
    assert!(!a_committed);

    // Final state: B's row and B's bytes, byte-for-byte; A's staged objects are
    // orphaned but present (nothing was deleted).
    let toc_row_after = assets
        .find_by_document_and_kind(document.id, ArchiveAssetKind::ArticleToc)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(toc_row_after.s3_key, b_toc_row.s3_key);
    let final_payload = get_object_string(storage.as_ref(), &toc_row_after.s3_key)
        .await
        .unwrap();
    let parsed: StoredArticleToc = serde_json::from_str(&final_payload).unwrap();
    assert_eq!(parsed, b_stored);
    assert_eq!(
        get_object_string(storage.as_ref(), &a_toc_key)
            .await
            .unwrap(),
        r#"{"stale":"payload"}"#
    );
}

fn a_payload_len() -> i64 {
    br#"{"stale":"payload"}"#.len() as i64
}
