#![allow(clippy::unwrap_used, clippy::expect_used)]

use ind_application::repos::document_lifecycle::{
    DocumentLifecycle, MaterializeIdentity, SaveToLibraryRequest,
};
use ind_domain::{ContentSource, DocumentId, DocumentType, NewUrlDocument, UserId};
use ind_persistence::repos::PgDocumentLifecycle;
use ind_test_support::{TestDb, UserFactory};

fn save_url(user_id: UserId, canonical_url: &str) -> SaveToLibraryRequest {
    SaveToLibraryRequest {
        identity: MaterializeIdentity::Url {
            document: NewUrlDocument {
                id: DocumentId::new(),
                user_id,
                document_type: DocumentType::Article,
                canonical_url: canonical_url.to_string(),
                original_url: None,
                content_hash: None,
                title: "Saved Title".into(),
                author: None,
                excerpt: None,
                published_at: None,
                language: None,
                domain: None,
                lead_image_url: None,
                thumbnail_url: None,
            },
            origin: None,
        },
        source: ContentSource::Manual,
        source_delivery_id: None,
        hide_deliveries: false,
        enqueue_engaged_ai: false,
        restore_policy: Default::default(),
        side_effects: None,
    }
}

#[tokio::test]
async fn concurrent_first_save_resolves_to_one_active_entry() {
    let db = TestDb::new().await;
    let pool = db.pool().clone();
    let user = UserFactory::default().insert(&pool).await;
    let canonical = format!("https://example.com/{}", uuid::Uuid::now_v7().simple());

    let first_repo = PgDocumentLifecycle::new(pool.clone());
    let second_repo = PgDocumentLifecycle::new(pool.clone());
    let (first, second) = tokio::join!(
        first_repo.save_to_library(save_url(user.id, &canonical)),
        second_repo.save_to_library(save_url(user.id, &canonical)),
    );
    let first = first.unwrap();
    let second = second.unwrap();

    assert_eq!(first.entry.id, second.entry.id);
    assert!(first.already_active ^ second.already_active);
    assert_eq!(
        sqlx::query_scalar!(
            "SELECT COUNT(*) FROM documents WHERE user_id = $1 AND canonical_url = $2",
            user.id.into_uuid(),
            canonical,
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        Some(1)
    );
    assert_eq!(
        sqlx::query_scalar!(
            "SELECT COUNT(*) FROM library_entries \
             WHERE user_id = $1 AND document_id = $2 AND deleted_at IS NULL",
            user.id.into_uuid(),
            first.document.id.into_uuid(),
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        Some(1)
    );
}
