#![allow(clippy::unwrap_used, clippy::expect_used)]

use chrono::{Duration, Utc};
use ind_application::repos::user_document_state::UserDocumentStateRepository;
use ind_domain::{
    BasisPoints, ClientId, ClientType, DocumentId, DocumentType, EventOrigin, NewReadingEvent,
    ReadingAnchor, ReadingCause, ReadingEventId, ReadingEventKind, ReadingPosition, UserId,
};
use ind_persistence::repos::PgUserDocumentStateRepository;
use ind_test_support::{DocumentFactory, TestDb, UserFactory};
use uuid::Uuid;

/// Deterministic per-label origin: same label yields the same `EventOrigin` within and
/// across calls, so tests can express "same device" / "different device" by string.
fn origin(client: &str) -> EventOrigin {
    EventOrigin::Device(ClientId::from_uuid(Uuid::new_v5(
        &Uuid::NAMESPACE_OID,
        client.as_bytes(),
    )))
}

fn progress(client: &str, seq: i64, percent: i32) -> NewReadingEvent {
    NewReadingEvent {
        id: ReadingEventId::new(),
        origin: origin(client),
        origin_seq: Some(seq),
        kind: ReadingEventKind::Progress,
        cause: ReadingCause::Reader,
        session_id: None,
        attempt: 1,
        progress: Some(BasisPoints::from_percent(percent).unwrap()),
        asset_kind: None,
        position_version: NewReadingEvent::CURRENT_POSITION_VERSION,
        position: Some(ReadingPosition {
            anchor: Some(ReadingAnchor::Page { page: percent }),
            fraction: Some(f64::from(percent) / 100.0),
            ..ReadingPosition::default()
        }),
        active_ms: Some(1_000),
        recorded_at: Utc::now(),
    }
}

async fn setup() -> (TestDb, PgUserDocumentStateRepository, UserId, DocumentId) {
    let db = TestDb::new().await;
    let pool = db.pool().clone();
    let user = UserFactory::default().insert(&pool).await;
    let document = DocumentFactory::new(user.id)
        .with_document_type(DocumentType::Pdf)
        .insert(&pool)
        .await;
    (
        db,
        PgUserDocumentStateRepository::new(pool),
        user.id,
        document.id,
    )
}

async fn percent(
    repo: &PgUserDocumentStateRepository,
    user: UserId,
    doc: DocumentId,
) -> Option<i32> {
    repo.find(user, doc)
        .await
        .unwrap()
        .and_then(|s| s.progress_percent)
}

#[tokio::test]
async fn exact_replay_is_counted_and_leaves_state_unchanged() {
    let (_db, repo, user, doc) = setup().await;
    let batch = vec![progress("cli_a", 1, 10), progress("cli_a", 2, 20)];

    let first = repo.append_reading_events(user, doc, &batch).await.unwrap();
    let second = repo.append_reading_events(user, doc, &batch).await.unwrap();

    assert_eq!((first.accepted, first.replayed), (2, 0));
    assert_eq!((second.accepted, second.replayed), (0, 2));
    assert_eq!(percent(&repo, user, doc).await, Some(20));
}

#[tokio::test]
async fn divergent_replay_conflicts_and_rolls_back_the_batch() {
    let (db, repo, user, doc) = setup().await;
    let original = progress("cli_a", 1, 10);
    repo.append_reading_events(user, doc, std::slice::from_ref(&original))
        .await
        .unwrap();

    let mut altered = original.clone();
    altered.progress = Some(BasisPoints::from_percent(11).unwrap());
    let err = repo
        .append_reading_events(user, doc, &[progress("cli_a", 2, 20), altered])
        .await
        .unwrap_err();
    assert!(err.to_string().contains("conflict"), "{err}");

    let mut reused_seq = progress("cli_a", 1, 12);
    reused_seq.id = ReadingEventId::new();
    assert!(
        repo.append_reading_events(user, doc, &[reused_seq])
            .await
            .is_err()
    );

    let count: i64 = sqlx::query_scalar!(
        "SELECT count(*) AS \"count!\" FROM reading_events WHERE document_id = $1",
        doc.into_uuid()
    )
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(count, 1);
    assert_eq!(percent(&repo, user, doc).await, Some(10));
}

#[tokio::test]
async fn same_client_lower_sequence_is_stored_but_not_projected() {
    let (db, repo, user, doc) = setup().await;
    repo.append_reading_events(user, doc, &[progress("cli_a", 5, 50)])
        .await
        .unwrap();
    repo.append_reading_events(user, doc, &[progress("cli_a", 3, 30)])
        .await
        .unwrap();

    let state = repo.find(user, doc).await.unwrap().unwrap();
    assert_eq!(state.progress_percent, Some(50));
    assert_eq!(state.max_progress_percent, Some(50));
    let stored: i64 = sqlx::query_scalar!(
        "SELECT count(*) AS \"count!\" FROM reading_events WHERE document_id = $1",
        doc.into_uuid()
    )
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(stored, 2);
}

#[tokio::test]
async fn stale_device_reconnecting_later_does_not_win() {
    let (_db, repo, user, doc) = setup().await;
    let mut phone = progress("cli_phone", 1, 60);
    phone.recorded_at = Utc::now() - Duration::minutes(1);
    repo.append_reading_events(user, doc, &[phone])
        .await
        .unwrap();

    let mut stale_tablet = progress("cli_tablet", 9, 40);
    stale_tablet.recorded_at = Utc::now() - Duration::hours(3);
    repo.append_reading_events(user, doc, &[stale_tablet])
        .await
        .unwrap();

    let state = repo.find(user, doc).await.unwrap().unwrap();
    assert_eq!(state.progress_percent, Some(60));
    assert_eq!(state.chapter_locator.as_deref(), Some("page:60"));
}

#[tokio::test]
async fn later_reading_on_another_device_wins_and_future_clocks_are_clamped() {
    let (_db, repo, user, doc) = setup().await;
    let mut phone = progress("cli_phone", 1, 60);
    phone.recorded_at = Utc::now() + Duration::days(2);
    repo.append_reading_events(user, doc, &[phone])
        .await
        .unwrap();

    let tablet = progress("cli_tablet", 1, 45);
    repo.append_reading_events(user, doc, &[tablet])
        .await
        .unwrap();

    let state = repo.find(user, doc).await.unwrap().unwrap();
    assert_eq!(
        state.progress_percent,
        Some(45),
        "clamped phone clock must not outrank a real later read"
    );
    assert_eq!(state.max_progress_percent, Some(60));
    let position: ReadingPosition = serde_json::from_value(state.scroll_position.unwrap()).unwrap();
    assert_eq!(position.anchor, Some(ReadingAnchor::Page { page: 45 }));
}

#[tokio::test]
async fn opening_the_document_between_writes_does_not_block_the_next_write() {
    let (_db, repo, user, doc) = setup().await;
    repo.append_reading_events(user, doc, &[progress("cli_a", 1, 10)])
        .await
        .unwrap();
    repo.record_document_opened(user, doc).await.unwrap();
    repo.append_reading_events(user, doc, &[progress("cli_a", 2, 20)])
        .await
        .unwrap();
    assert_eq!(percent(&repo, user, doc).await, Some(20));
}

#[tokio::test]
async fn surface_progress_writes_order_by_arrival() {
    let (_db, repo, user, doc) = setup().await;
    let web = EventOrigin::Surface(ClientType::Web);
    let at = |page: i32| {
        Some(ReadingPosition {
            anchor: Some(ReadingAnchor::Page { page }),
            ..ReadingPosition::default()
        })
    };
    repo.record_progress(user, doc, 70, at(7), web.clone())
        .await
        .unwrap();
    repo.record_progress(user, doc, 30, at(3), web)
        .await
        .unwrap();
    let state = repo.find(user, doc).await.unwrap().unwrap();
    assert_eq!(state.progress_percent, Some(30));
    assert_eq!(state.max_progress_percent, Some(70));
}

#[tokio::test]
async fn foreign_document_is_not_found() {
    let (db, repo, _user, doc) = setup().await;
    let stranger = UserFactory::default().insert(db.pool()).await;
    let err = repo
        .append_reading_events(stranger.id, doc, &[progress("cli_a", 1, 10)])
        .await
        .unwrap_err();
    assert!(err.to_string().contains("Document not found"), "{err}");
}

/// A stale pass must contribute nothing to the current one. Position loss was already guarded,
/// but the ceiling and the completion latch were not: an old attempt-1 event could raise
/// attempt 2's max back to 100 and relatch a finish the reader never declared.
#[tokio::test]
async fn a_stale_attempt_cannot_raise_or_relatch_the_current_one() {
    let (_db, repo, user, doc) = setup().await;
    let device = origin("cli_reread");

    let mut done = progress("cli_reread", 1, 100);
    done.kind = ReadingEventKind::Finished;
    repo.append_reading_events(user, doc, std::slice::from_ref(&done))
        .await
        .unwrap();

    let mut restart = progress("cli_reread", 2, 3);
    restart.attempt = 2;
    repo.append_reading_events(user, doc, std::slice::from_ref(&restart))
        .await
        .unwrap();
    let after_restart = repo.find(user, doc).await.unwrap().unwrap();
    assert_eq!(after_restart.max_progress_percent, Some(3));
    assert!(after_restart.finished_at.is_none());

    let stale = NewReadingEvent {
        id: ReadingEventId::new(),
        origin: device,
        origin_seq: Some(3),
        kind: ReadingEventKind::Finished,
        cause: ReadingCause::Reader,
        session_id: None,
        attempt: 1,
        progress: Some(BasisPoints::from_percent(100).unwrap()),
        position: None,
        asset_kind: None,
        position_version: NewReadingEvent::CURRENT_POSITION_VERSION,
        active_ms: None,
        recorded_at: Utc::now(),
    };
    repo.append_reading_events(user, doc, std::slice::from_ref(&stale))
        .await
        .unwrap();

    let state = repo.find(user, doc).await.unwrap().unwrap();
    assert_eq!(
        state.progress_percent,
        Some(3),
        "a stale attempt must not move the position"
    );
    assert_eq!(
        state.max_progress_percent,
        Some(3),
        "a stale attempt must not raise the current attempt's ceiling"
    );
    assert!(
        state.finished_at.is_none(),
        "a stale attempt must not relatch a completion the current attempt never declared"
    );
}
