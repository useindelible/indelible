use apalis_postgres::Config;
use chrono::{DateTime, Duration, Utc};
use ind_application::repos::outbox::JobOutboxRepository;
use ind_domain::{GenericJobEnvelope, SearchReindexDocumentJob, job_types};
use ind_test_support::TestDb;

use crate::repos::PgJobOutboxRepository;

use super::PgOutboxHandoff;

fn handoff(db: &TestDb) -> PgOutboxHandoff {
    PgOutboxHandoff::new(
        db.pool().clone(),
        Config::new(std::any::type_name::<GenericJobEnvelope>()),
    )
}

async fn enqueue(
    repo: &PgJobOutboxRepository,
    dedupe_key: Option<String>,
) -> ind_domain::JobOutboxId {
    repo.enqueue(
        job_types::SEARCH_REINDEX_DOCUMENT,
        serde_json::to_value(SearchReindexDocumentJob {
            document_id: ind_domain::DocumentId::new(),
        })
        .unwrap(),
        dedupe_key,
        Utc::now() - Duration::seconds(1),
    )
    .await
    .unwrap()
    .id
}

#[tokio::test]
async fn handoff_is_atomic_and_dedupes_only_active_jobs() {
    let db = TestDb::new().await;
    let repo = PgJobOutboxRepository::new(db.pool().clone());
    let handoff = handoff(&db);
    let dedupe_key = format!("search:{}", uuid::Uuid::now_v7());

    let first_id = enqueue(&repo, Some(dedupe_key.clone())).await;
    let first = handoff.handoff_batch(10).await.unwrap();
    assert_eq!((first.claimed, first.relayed, first.deduped), (1, 1, 0));

    let queued = sqlx::query_scalar::<_, Vec<u8>>(
        "SELECT job FROM apalis.jobs WHERE metadata->>'dedupe_key' = $1",
    )
    .bind(&dedupe_key)
    .fetch_one(db.pool())
    .await
    .unwrap();
    let envelope: GenericJobEnvelope = serde_json::from_slice(&queued).unwrap();
    assert_eq!(envelope.outbox_id, first_id);
    assert_eq!(envelope.job_type, job_types::SEARCH_REINDEX_DOCUMENT);

    let duplicate_id = enqueue(&repo, Some(dedupe_key.clone())).await;
    let duplicate = handoff.handoff_batch(10).await.unwrap();
    assert_eq!(
        (duplicate.claimed, duplicate.relayed, duplicate.deduped),
        (1, 0, 1)
    );
    let deferred = sqlx::query_as::<_, (Option<DateTime<Utc>>, bool)>(
        "SELECT dispatched_at, available_at > now() FROM job_outbox WHERE id = $1",
    )
    .bind(duplicate_id.into_uuid())
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert!(deferred.0.is_none() && deferred.1);

    sqlx::query("UPDATE apalis.jobs SET status = 'Done', done_at = now() WHERE metadata->>'dedupe_key' = $1")
        .bind(&dedupe_key)
        .execute(db.pool())
        .await
        .unwrap();
    sqlx::query("UPDATE job_outbox SET available_at = now() - INTERVAL '1 second' WHERE id = $1")
        .bind(duplicate_id.into_uuid())
        .execute(db.pool())
        .await
        .unwrap();

    let redelivery = handoff.handoff_batch(10).await.unwrap();
    assert_eq!(
        (redelivery.claimed, redelivery.relayed, redelivery.deduped),
        (1, 1, 0)
    );
    assert_eq!(handoff.handoff_batch(10).await.unwrap().claimed, 0);
}

#[tokio::test]
async fn handoff_failure_rolls_back_the_dispatch_stamp() {
    let db = TestDb::new().await;
    let repo = PgJobOutboxRepository::new(db.pool().clone());
    let job_id = enqueue(&repo, None).await;
    sqlx::query("DROP TABLE apalis.jobs CASCADE")
        .execute(db.pool())
        .await
        .unwrap();

    handoff(&db).handoff_batch(10).await.unwrap_err();
    let dispatched_at = sqlx::query_scalar::<_, Option<DateTime<Utc>>>(
        "SELECT dispatched_at FROM job_outbox WHERE id = $1",
    )
    .bind(job_id.into_uuid())
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert!(dispatched_at.is_none());
}
