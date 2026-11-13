#![allow(clippy::unwrap_used, clippy::expect_used)]

use chrono::{DateTime, Utc};
use ind_application::repos::document_note::{DocumentNoteRepository, NoteWrite};
use ind_domain::{DocumentId, DocumentNote, UserId};
use ind_persistence::repos::PgDocumentNoteRepository;
use ind_test_support::{DocumentFactory, TestDb, UserFactory};

struct Notes {
    repo: PgDocumentNoteRepository,
    user_id: UserId,
    document_id: DocumentId,
}

impl Notes {
    async fn new(db: &TestDb) -> Self {
        let pool = db.pool().clone();
        let user = UserFactory::default().insert(&pool).await;
        let document = DocumentFactory::new(user.id).insert(&pool).await;
        Self {
            repo: PgDocumentNoteRepository::new(pool),
            user_id: user.id,
            document_id: document.id,
        }
    }

    async fn write(&self, body: &str, base: Option<DateTime<Utc>>) -> NoteWrite {
        self.repo
            .upsert_for_document(self.user_id, self.document_id, body, base, vec![])
            .await
            .unwrap()
    }

    async fn written(&self, body: &str, base: Option<DateTime<Utc>>) -> DocumentNote {
        match self.write(body, base).await {
            NoteWrite::Written(note) => Some(note),
            NoteWrite::Conflict(_) => None,
        }
        .expect("the write was refused")
    }

    async fn stored_body(&self) -> String {
        self.repo
            .find_by_document(self.user_id, self.document_id)
            .await
            .unwrap()
            .unwrap()
            .body
    }
}

#[tokio::test]
async fn a_write_without_a_base_overwrites() {
    let db = TestDb::new().await;
    let notes = Notes::new(&db).await;

    notes.written("one", None).await;
    notes.written("two", None).await;

    assert_eq!(notes.stored_body().await, "two");
}

#[tokio::test]
async fn a_write_on_the_seen_version_is_applied() {
    let db = TestDb::new().await;
    let notes = Notes::new(&db).await;
    let first = notes.written("one", None).await;

    let second = notes.written("two", Some(first.updated_at)).await;

    assert!(second.updated_at > first.updated_at);
    assert_eq!(notes.stored_body().await, "two");
}

#[tokio::test]
async fn a_write_on_an_older_version_is_refused_with_the_current_note() {
    let db = TestDb::new().await;
    let notes = Notes::new(&db).await;
    let first = notes.written("one", None).await;
    let theirs = notes.written("theirs", None).await;

    let refused = notes.write("mine", Some(first.updated_at)).await;

    let current = match refused {
        NoteWrite::Conflict(current) => Some(current),
        NoteWrite::Written(_) => None,
    }
    .expect("a stale base was written");
    assert_eq!(current.body, "theirs");
    assert_eq!(current.updated_at, theirs.updated_at);
    assert_eq!(notes.stored_body().await, "theirs");
}

#[tokio::test]
async fn a_base_at_millisecond_precision_matches_the_stored_microseconds() {
    let db = TestDb::new().await;
    let notes = Notes::new(&db).await;
    let first = notes.written("one", None).await;
    let base = DateTime::from_timestamp_millis(first.updated_at.timestamp_millis()).unwrap();

    notes.written("two", Some(base)).await;

    assert_eq!(notes.stored_body().await, "two");
}

#[tokio::test]
async fn a_first_note_with_a_base_is_created() {
    let db = TestDb::new().await;
    let notes = Notes::new(&db).await;

    notes.written("one", Some(DateTime::UNIX_EPOCH)).await;
    let refused = notes.write("two", Some(DateTime::UNIX_EPOCH)).await;

    assert!(matches!(refused, NoteWrite::Conflict(_)));
    assert_eq!(notes.stored_body().await, "one");
}
