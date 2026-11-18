#![allow(clippy::unwrap_used, clippy::expect_used)]

use chrono::{DateTime, Utc};
use ind_application::repos::document_note::{DocumentNoteRepository, NoteWrite};
use ind_application::repos::lifecycle_outbox::{OutboxEntry, search_reindex_document_outbox};
use ind_domain::{DocumentId, DocumentNote, UserId};
use ind_persistence::repos::PgDocumentNoteRepository;
use ind_test_support::{DocumentFactory, TestDb, UserFactory};
use sqlx::PgPool;

struct Notes {
    pool: PgPool,
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
            repo: PgDocumentNoteRepository::new(pool.clone()),
            pool,
            user_id: user.id,
            document_id: document.id,
        }
    }

    async fn write(&self, body: &str, base: Option<DateTime<Utc>>) -> NoteWrite {
        self.write_with(body, base, vec![]).await
    }

    async fn write_with(
        &self,
        body: &str,
        base: Option<DateTime<Utc>>,
        outbox: Vec<OutboxEntry>,
    ) -> NoteWrite {
        self.repo
            .upsert_for_document(self.user_id, self.document_id, body, base, outbox)
            .await
            .unwrap()
    }

    fn reindex(&self) -> OutboxEntry {
        search_reindex_document_outbox(self.document_id, Utc::now())
    }

    async fn outbox_rows(&self) -> i64 {
        sqlx::query_scalar!("SELECT COUNT(*) FROM job_outbox")
            .fetch_one(&self.pool)
            .await
            .unwrap()
            .unwrap_or_default()
    }

    async fn move_stamp_ahead(&self, note: &DocumentNote) -> DateTime<Utc> {
        sqlx::query_scalar!(
            "UPDATE item_notes SET updated_at = updated_at + interval '1 hour' WHERE id = $1 \
             RETURNING updated_at",
            note.id.into_uuid()
        )
        .fetch_one(&self.pool)
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

#[tokio::test]
async fn a_write_after_a_stamp_ahead_of_the_clock_stays_ahead_of_it() {
    let db = TestDb::new().await;
    let notes = Notes::new(&db).await;
    let first = notes.written("one", None).await;
    let ahead = notes.move_stamp_ahead(&first).await;

    let second = notes.written("two", None).await;

    assert!(
        second.updated_at > ahead,
        "{} <= {ahead}",
        second.updated_at
    );
}

#[tokio::test]
async fn a_refused_write_commits_no_outbox_row() {
    let db = TestDb::new().await;
    let notes = Notes::new(&db).await;
    let first = notes.written("one", None).await;
    notes.written("theirs", None).await;
    let before = notes.outbox_rows().await;

    let refused = notes
        .write_with("mine", Some(first.updated_at), vec![notes.reindex()])
        .await;

    assert!(matches!(refused, NoteWrite::Conflict(_)));
    assert_eq!(notes.outbox_rows().await, before);
}

#[tokio::test]
async fn a_written_note_commits_its_outbox_rows() {
    let db = TestDb::new().await;
    let notes = Notes::new(&db).await;
    let before = notes.outbox_rows().await;

    notes.write_with("one", None, vec![notes.reindex()]).await;

    assert_eq!(notes.outbox_rows().await, before + 1);
}
