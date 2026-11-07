#![allow(clippy::unwrap_used, clippy::expect_used)]

use chrono::Utc;
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

/// The high-water mark is read in a separate statement, so two concurrent appends from one
/// origin both see the pre-write value. Only a sequence comparison inside the projection's
/// atomic update keeps the lower sequence from landing last and winning.
#[tokio::test]
async fn concurrent_same_origin_writes_never_regress_to_a_lower_sequence() {
    let (_db, repo, user, doc) = setup().await;
    for round in 0..8 {
        let ahead = progress("cli_race", 2 * round + 2, 50);
        let behind = progress("cli_race", 2 * round + 1, 30);
        let (a, b) = tokio::join!(
            repo.append_reading_events(user, doc, std::slice::from_ref(&ahead)),
            repo.append_reading_events(user, doc, std::slice::from_ref(&behind)),
        );
        a.unwrap();
        b.unwrap();
        let state = repo.find(user, doc).await.unwrap().unwrap();
        assert_eq!(
            state.progress_percent,
            Some(50),
            "round {round}: the lower sequence must not win, whichever commits last"
        );
    }
}

/// Two identical retries can both miss a pre-insert lookup. Insert-first with
/// `ON CONFLICT DO NOTHING` makes the loser compare the stored row instead of racing the key.
#[tokio::test]
async fn concurrent_identical_replays_are_both_accepted() {
    let (_db, repo, user, doc) = setup().await;
    for round in 0..8 {
        let event = progress("cli_replay", round + 1, 40);
        let (a, b) = tokio::join!(
            repo.append_reading_events(user, doc, std::slice::from_ref(&event)),
            repo.append_reading_events(user, doc, std::slice::from_ref(&event)),
        );
        let a = a.expect("first identical append must succeed");
        let b = b.expect("a concurrent identical replay must not conflict");
        assert_eq!(
            a.accepted + b.accepted,
            1,
            "exactly one of the two may be counted as new"
        );
        assert_eq!(
            a.replayed + b.replayed,
            1,
            "the other must count as replayed"
        );
    }
}

/// A caller with no device counter sends no sequence; the server assigns one, so PATCH and
/// POST for the same surface origin share a single ordering authority and cannot collide.
#[tokio::test]
async fn surface_writers_get_server_assigned_sequences_that_do_not_collide() {
    let (db, repo, user, doc) = setup().await;
    let web = EventOrigin::Surface(ClientType::Web);
    repo.record_progress(user, doc, 20, None, web.clone())
        .await
        .unwrap();
    let event = NewReadingEvent {
        id: ReadingEventId::new(),
        origin: web,
        origin_seq: None,
        kind: ReadingEventKind::Progress,
        cause: ReadingCause::Reader,
        session_id: None,
        attempt: 1,
        progress: Some(BasisPoints::from_percent(60).unwrap()),
        position: None,
        asset_kind: None,
        position_version: NewReadingEvent::CURRENT_POSITION_VERSION,
        active_ms: None,
        recorded_at: Utc::now(),
    };
    repo.append_reading_events(user, doc, std::slice::from_ref(&event))
        .await
        .unwrap();

    let seqs: Vec<i64> = sqlx::query_scalar(
        "SELECT origin_seq FROM reading_events WHERE document_id = $1 AND origin = 'surface:web' \
         ORDER BY origin_seq",
    )
    .bind(doc.into_uuid())
    .fetch_all(db.pool())
    .await
    .unwrap();
    assert_eq!(seqs.len(), 2, "both writers must be recorded");
    assert!(seqs[0] < seqs[1], "sequences must be distinct and ordered");
    assert_eq!(
        repo.find(user, doc)
            .await
            .unwrap()
            .unwrap()
            .progress_percent,
        Some(60)
    );
}
