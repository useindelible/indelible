#![allow(clippy::unwrap_used)]

use ind_application::repos::document_asset::{DocumentAssetRepository, PreparedReadableLocation};
use ind_domain::{ArchiveAssetKind, ArchiveAssetStatus, DocumentId, NewDocumentAsset};
use ind_persistence::repos::PgDocumentAssetRepository;
use ind_test_support::{DocumentFactory, TestDb, UserFactory};

fn asset(document_id: DocumentId, kind: ArchiveAssetKind, key: &str) -> NewDocumentAsset {
    NewDocumentAsset {
        document_id,
        asset_kind: kind,
        s3_key: key.to_string(),
        s3_bucket: "indelible".to_string(),
        content_type: if kind == ArchiveAssetKind::ArticleToc {
            "application/json".to_string()
        } else {
            "text/html".to_string()
        },
        size_bytes: 100,
        status: ArchiveAssetStatus::Completed,
        failed_reason: None,
    }
}

#[tokio::test]
async fn fresh_commit_swaps_pointer_and_writes_toc_without_refreshing_stamp() {
    let db = TestDb::new().await;
    let pool = db.pool().clone();
    let user = UserFactory::default().insert(&pool).await;
    let document = DocumentFactory::new(user.id).insert(&pool).await;
    let repo = PgDocumentAssetRepository::new(pool.clone());

    let readable = repo
        .upsert_document_asset(asset(
            document.id,
            ArchiveAssetKind::ReadableHtml,
            "u/d/readable_html.html",
        ))
        .await
        .unwrap();

    let committed = repo
        .commit_article_toc(
            document.id,
            readable.created_at,
            Some(PreparedReadableLocation {
                s3_key: "u/d/readable_html.abc123.html".to_string(),
                size_bytes: 222,
            }),
            asset(
                document.id,
                ArchiveAssetKind::ArticleToc,
                "u/d/article_toc.def.json",
            ),
        )
        .await
        .unwrap();
    assert!(committed);

    let readable_after = repo
        .find_by_document_and_kind(document.id, ArchiveAssetKind::ReadableHtml)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(readable_after.s3_key, "u/d/readable_html.abc123.html");
    assert_eq!(readable_after.size_bytes, 222);
    // The swap changes representation, not content version: the stamp must
    // survive so the pre-uploaded ToC payload's source version still matches.
    assert_eq!(readable_after.created_at, readable.created_at);

    let toc = repo
        .find_by_document_and_kind(document.id, ArchiveAssetKind::ArticleToc)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(toc.s3_key, "u/d/article_toc.def.json");
}

#[tokio::test]
async fn stale_guard_rejects_commit_and_writes_nothing() {
    let db = TestDb::new().await;
    let pool = db.pool().clone();
    let user = UserFactory::default().insert(&pool).await;
    let document = DocumentFactory::new(user.id).insert(&pool).await;
    let repo = PgDocumentAssetRepository::new(pool.clone());

    let observed = repo
        .upsert_document_asset(asset(
            document.id,
            ArchiveAssetKind::ReadableHtml,
            "u/d/readable_html.html",
        ))
        .await
        .unwrap();

    // A reprocess-equivalent upsert refreshes created_at on the same stable key.
    let newer = repo
        .upsert_document_asset(asset(
            document.id,
            ArchiveAssetKind::ReadableHtml,
            "u/d/readable_html.html",
        ))
        .await
        .unwrap();
    assert_ne!(newer.created_at, observed.created_at);

    let committed = repo
        .commit_article_toc(
            document.id,
            observed.created_at,
            Some(PreparedReadableLocation {
                s3_key: "u/d/readable_html.stale.html".to_string(),
                size_bytes: 100,
            }),
            asset(
                document.id,
                ArchiveAssetKind::ArticleToc,
                "u/d/article_toc.stale.json",
            ),
        )
        .await
        .unwrap();
    assert!(!committed);

    let readable_after = repo
        .find_by_document_and_kind(document.id, ArchiveAssetKind::ReadableHtml)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(readable_after.s3_key, "u/d/readable_html.html");
    assert!(
        repo.find_by_document_and_kind(document.id, ArchiveAssetKind::ArticleToc)
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn no_swap_commit_guards_version_and_writes_toc_only() {
    let db = TestDb::new().await;
    let pool = db.pool().clone();
    let user = UserFactory::default().insert(&pool).await;
    let document = DocumentFactory::new(user.id).insert(&pool).await;
    let repo = PgDocumentAssetRepository::new(pool.clone());

    let readable = repo
        .upsert_document_asset(asset(
            document.id,
            ArchiveAssetKind::ReadableHtml,
            "u/d/readable_html.html",
        ))
        .await
        .unwrap();

    let committed = repo
        .commit_article_toc(
            document.id,
            readable.created_at,
            None,
            asset(
                document.id,
                ArchiveAssetKind::ArticleToc,
                "u/d/article_toc.xyz.json",
            ),
        )
        .await
        .unwrap();
    assert!(committed);

    let readable_after = repo
        .find_by_document_and_kind(document.id, ArchiveAssetKind::ReadableHtml)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(readable_after.s3_key, "u/d/readable_html.html");
    assert!(
        repo.find_by_document_and_kind(document.id, ArchiveAssetKind::ArticleToc)
            .await
            .unwrap()
            .is_some()
    );

    // Re-commit against a bumped version must fail even with no swap.
    let newer = repo
        .upsert_document_asset(asset(
            document.id,
            ArchiveAssetKind::ReadableHtml,
            "u/d/readable_html.html",
        ))
        .await
        .unwrap();
    assert_ne!(newer.created_at, readable.created_at);
    let recommit = repo
        .commit_article_toc(
            document.id,
            readable.created_at,
            None,
            asset(
                document.id,
                ArchiveAssetKind::ArticleToc,
                "u/d/article_toc.old.json",
            ),
        )
        .await
        .unwrap();
    assert!(!recommit);
}
