#![allow(clippy::unwrap_used)]

use chrono::{Duration, Utc};
use ind_application::repos::background_job_recovery::{
    BackgroundJobRecoveryRepository, DeadLetterInsert, RecoveryFailureInput, RecoveryReplay,
};
use ind_domain::{BackgroundJobFailureClass, BackgroundJobSubjectKind};
use ind_persistence::repos::PgBackgroundJobRecoveryRepository;
use ind_test_support::TestDb;

fn waiting_input<'a>(
    recovery_key: &'a str,
    payload: serde_json::Value,
    now: chrono::DateTime<Utc>,
) -> RecoveryFailureInput<'a> {
    RecoveryFailureInput {
        recovery_key,
        job_type: "document.ai.embed",
        payload,
        dedupe_key: None,
        outbox_id: None,
        subject_kind: Some(BackgroundJobSubjectKind::Document),
        subject_id: Some("doc_test"),
        failure_class: BackgroundJobFailureClass::Retryable,
        failure_reason_code: "embed_timeout",
        error_message: "embedding service timed out",
        apalis_attempts: 5,
        next_retry_at: Some(now - Duration::seconds(10)),
        now,
    }
}

fn dead_letter<'a>(
    payload: &'a serde_json::Value,
    message: &'a str,
    failed_at: chrono::DateTime<Utc>,
) -> DeadLetterInsert<'a> {
    DeadLetterInsert {
        job_type: "document.ai.embed",
        payload: payload.clone(),
        dedupe_key: None,
        failure_reason_code: Some("test_failure"),
        error_message: message,
        attempts: 1,
        failed_at,
    }
}

fn patient_input<'a>(
    recovery_key: &'a str,
    payload: serde_json::Value,
    now: chrono::DateTime<Utc>,
) -> RecoveryFailureInput<'a> {
    RecoveryFailureInput {
        failure_class: BackgroundJobFailureClass::Patient,
        failure_reason_code: "ai_provider_unavailable",
        error_message: "provider unavailable: connection refused",
        apalis_attempts: 1,
        ..waiting_input(recovery_key, payload, now)
    }
}

#[tokio::test]
async fn failure_class_transitions_reset_recovery_attempts() {
    let db = TestDb::new().await;
    let repo = PgBackgroundJobRecoveryRepository::new(db.pool().clone());
    let now = Utc::now();
    let payload = serde_json::json!({"document_id": "doc_test"});

    let row = repo
        .upsert_waiting_failure(waiting_input("rk:transition", payload.clone(), now))
        .await
        .unwrap();
    assert_eq!(row.failure_class, BackgroundJobFailureClass::Retryable);

    let lease_until = now + Duration::seconds(30);
    repo.claim_due(now, "worker-a", lease_until, 10)
        .await
        .unwrap();
    repo.defer_recovery(row.id, now + Duration::seconds(60), now)
        .await
        .unwrap();

    let parked = repo
        .upsert_waiting_failure(patient_input(
            "rk:transition",
            payload.clone(),
            now + Duration::seconds(90),
        ))
        .await
        .unwrap();
    assert_eq!(parked.failure_class, BackgroundJobFailureClass::Patient);
    assert_eq!(
        parked.recovery_attempts, 0,
        "entering patience must reset the attempt budget"
    );
    assert_eq!(parked.failure_reason_code, "ai_provider_unavailable");

    let recovered = repo
        .upsert_waiting_failure(waiting_input(
            "rk:transition",
            payload,
            now + Duration::seconds(500),
        ))
        .await
        .unwrap();
    assert_eq!(
        recovered.failure_class,
        BackgroundJobFailureClass::Retryable
    );
    assert_eq!(
        recovered.recovery_attempts, 0,
        "exiting patience must grant a fresh finite budget"
    );
}

async fn embed_outbox_count(db: &TestDb) -> Option<i64> {
    sqlx::query_scalar!(
        "SELECT COUNT(*) FROM job_outbox WHERE job_type = $1",
        "document.ai.embed",
    )
    .fetch_one(db.pool())
    .await
    .unwrap()
}

#[tokio::test]
async fn replay_via_outbox_commits_outbox_and_ledger_in_one_transaction() {
    let db = TestDb::new().await;
    let repo = PgBackgroundJobRecoveryRepository::new(db.pool().clone());
    let now = Utc::now();
    let payload = serde_json::json!({"document_id": "doc_test"});

    let row = repo
        .upsert_waiting_failure(waiting_input("rk:atomic", payload.clone(), now))
        .await
        .unwrap();
    repo.claim_due(now, "worker-a", now + Duration::seconds(60), 10)
        .await
        .unwrap();

    let next_retry_at = now + Duration::seconds(300);
    let outbox_id = repo
        .replay_via_outbox(RecoveryReplay {
            id: row.id,
            job_type: "document.ai.embed",
            payload: payload.clone(),
            dedupe_key: None,
            lease_owner: "worker-a",
            next_retry_at,
            now,
        })
        .await
        .unwrap();

    assert_eq!(embed_outbox_count(&db).await, Some(1));

    let ledger = sqlx::query!(
        "SELECT outbox_id, recovery_attempts, status FROM background_job_recoveries WHERE id = $1",
        row.id.into_uuid(),
    )
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(ledger.outbox_id, Some(outbox_id.into_uuid()));
    assert_eq!(ledger.recovery_attempts, 1);
    assert_eq!(ledger.status, "waiting");

    let missing = ind_domain::BackgroundJobRecoveryId::new();
    let result = repo
        .replay_via_outbox(RecoveryReplay {
            id: missing,
            job_type: "document.ai.embed",
            payload,
            dedupe_key: None,
            lease_owner: "worker-a",
            next_retry_at,
            now,
        })
        .await;
    assert!(result.is_err(), "replaying a vanished row must fail");
    assert_eq!(
        embed_outbox_count(&db).await,
        Some(1),
        "failed ledger update must roll back the outbox insert"
    );
}

#[tokio::test]
async fn patient_reparks_advance_the_sparse_schedule_by_accumulated_attempts() {
    let db = TestDb::new().await;
    let repo = PgBackgroundJobRecoveryRepository::new(db.pool().clone());
    let now = Utc::now();
    let payload = serde_json::json!({"document_id": "doc_test"});

    let parked = repo
        .upsert_waiting_failure(patient_input("rk:pacing", payload.clone(), now))
        .await
        .unwrap();
    repo.claim_due(now, "worker-a", now + Duration::seconds(60), 10)
        .await
        .unwrap();

    repo.replay_via_outbox(RecoveryReplay {
        id: parked.id,
        job_type: "document.ai.embed",
        payload: payload.clone(),
        dedupe_key: None,
        lease_owner: "worker-a",
        next_retry_at: now + Duration::seconds(60),
        now,
    })
    .await
    .unwrap();

    let reparked = repo
        .upsert_waiting_failure(patient_input(
            "rk:pacing",
            payload,
            now + Duration::seconds(70),
        ))
        .await
        .unwrap();
    assert_eq!(reparked.recovery_attempts, 1);

    let rescheduled_to = now + Duration::seconds(370);
    repo.reschedule_waiting(reparked.id, rescheduled_to, now + Duration::seconds(70))
        .await
        .unwrap();
    let row = sqlx::query!(
        "SELECT recovery_attempts, next_retry_at FROM background_job_recoveries WHERE id = $1",
        reparked.id.into_uuid(),
    )
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(
        row.recovery_attempts, 1,
        "rescheduling must not consume attempt budget"
    );
    let moved = row.next_retry_at.unwrap() - rescheduled_to;
    assert!(
        moved.num_milliseconds().abs() < 5,
        "reschedule must move next_retry_at"
    );
}

#[tokio::test]
async fn mark_recovery_terminal_does_not_double_write_dlq() {
    let db = TestDb::new().await;
    let repo = PgBackgroundJobRecoveryRepository::new(db.pool().clone());
    let now = Utc::now();
    let payload = serde_json::json!({"document_id": "doc_test"});
    let row = repo
        .upsert_waiting_failure(waiting_input("rk:cap2", payload.clone(), now))
        .await
        .unwrap();

    repo.claim_due(now, "worker-a", now + Duration::seconds(120), 10)
        .await
        .unwrap();
    let later = now + Duration::seconds(60);
    repo.mark_recovery_terminal(
        row.id,
        "recovery_attempts_exhausted",
        "first",
        "worker-a",
        dead_letter(&payload, "first cap", later),
        later,
    )
    .await
    .unwrap();
    let second = repo
        .mark_recovery_terminal(
            row.id,
            "recovery_attempts_exhausted",
            "second",
            "worker-a",
            dead_letter(&payload, "second cap", later),
            later,
        )
        .await;
    assert!(
        second.is_err(),
        "a terminal row is no longer leased, so the fence rejects the repeat"
    );

    assert_eq!(
        sqlx::query_scalar!(
            "SELECT COUNT(*) FROM dead_letter_jobs WHERE original_job_type = $1",
            "document.ai.embed",
        )
        .fetch_one(db.pool())
        .await
        .unwrap(),
        Some(1)
    );
}

#[tokio::test]
async fn recovery_claims_honor_leases_backoff_and_expiry() {
    let db = TestDb::new().await;
    let repo = PgBackgroundJobRecoveryRepository::new(db.pool().clone());
    let now = Utc::now();
    let row = repo
        .upsert_waiting_failure(waiting_input(
            "rk:lease",
            serde_json::json!({"document_id": "doc_test"}),
            now,
        ))
        .await
        .unwrap();

    let lease_until = now + Duration::seconds(30);
    let first = repo
        .claim_due(now, "worker-a", lease_until, 10)
        .await
        .unwrap();
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].lease_owner.as_deref(), Some("worker-a"));
    assert!(
        repo.claim_due(now, "worker-b", lease_until, 10)
            .await
            .unwrap()
            .is_empty()
    );

    let next_retry_at = lease_until + Duration::seconds(60);
    repo.defer_recovery(row.id, next_retry_at, lease_until)
        .await
        .unwrap();
    assert!(
        repo.claim_due(lease_until, "worker-b", next_retry_at, 10)
            .await
            .unwrap()
            .is_empty()
    );

    let reclaimed = repo
        .claim_due(
            next_retry_at,
            "worker-c",
            next_retry_at + Duration::seconds(30),
            10,
        )
        .await
        .unwrap();
    assert_eq!(reclaimed.len(), 1);
    assert_eq!(reclaimed[0].recovery_attempts, 1);
    assert_eq!(reclaimed[0].lease_owner.as_deref(), Some("worker-c"));
}
