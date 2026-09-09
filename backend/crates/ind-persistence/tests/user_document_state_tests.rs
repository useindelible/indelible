#![allow(clippy::unwrap_used, clippy::expect_used)]

use ind_application::repos::user_document_state::UserDocumentStateRepository;
use ind_domain::{DocumentId, UserDocumentState, UserId};
use ind_persistence::repos::PgUserDocumentStateRepository;
use ind_test_support::{DocumentFactory, TestDb, UserFactory};

async fn state(
    repo: &PgUserDocumentStateRepository,
    user_id: UserId,
    document_id: DocumentId,
) -> UserDocumentState {
    repo.find(user_id, document_id).await.unwrap().unwrap()
}

#[tokio::test]
async fn clear_read_state_resets_read_status_and_keeps_the_chapter_position() {
    let db = TestDb::new().await;
    let pool = db.pool().clone();
    let user = UserFactory::default().insert(&pool).await;
    let started = DocumentFactory::new(user.id).insert(&pool).await;
    let finished = DocumentFactory::new(user.id).insert(&pool).await;
    let repo = PgUserDocumentStateRepository::new(pool.clone());

    repo.record_progress(user.id, started.id, 60, Some("ch-2".into()), Some(12))
        .await
        .unwrap();
    repo.record_progress(user.id, finished.id, 100, None, None)
        .await
        .unwrap();
    assert!(
        state(&repo, user.id, finished.id)
            .await
            .finished_at
            .is_some()
    );

    repo.clear_read_state(user.id, started.id).await.unwrap();
    repo.clear_read_state(user.id, finished.id).await.unwrap();

    let cleared = state(&repo, user.id, started.id).await;
    assert_eq!(cleared.progress_percent, None);
    assert_eq!(cleared.max_progress_percent, None);
    assert_eq!(cleared.last_read_at, None);
    assert_eq!(cleared.finished_at, None);
    assert_eq!(cleared.chapter_locator.as_deref(), Some("ch-2"));
    assert_eq!(cleared.chapter_offset, Some(12));

    // The next autosave after unread must not resurrect the finished state.
    let reread = repo
        .record_progress(user.id, finished.id, 10, None, None)
        .await
        .unwrap();
    assert_eq!(reread.finished_at, None);
    assert_eq!(reread.max_progress_percent, Some(10));
}

#[tokio::test]
async fn clear_read_state_succeeds_without_a_state_row() {
    let db = TestDb::new().await;
    let pool = db.pool().clone();
    let user = UserFactory::default().insert(&pool).await;
    let document = DocumentFactory::new(user.id).insert(&pool).await;
    let repo = PgUserDocumentStateRepository::new(pool.clone());

    repo.clear_read_state(user.id, document.id).await.unwrap();

    assert!(repo.find(user.id, document.id).await.unwrap().is_none());
}
