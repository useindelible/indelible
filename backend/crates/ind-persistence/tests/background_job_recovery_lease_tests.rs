#![allow(clippy::unwrap_used)]

use chrono::{Duration, Utc};
use ind_application::repos::background_job_recovery::{
    BackgroundJobRecoveryRepository, DeadLetterInsert, RecoveryFailureInput, RecoveryReplay,
};
use ind_domain::{BackgroundJobFailureClass, BackgroundJobRecoveryId, BackgroundJobSubjectKind};
use ind_persistence::repos::PgBackgroundJobRecoveryRepository;
use ind_test_support::TestDb;

fn failure_input<'a>(
    recovery_key: &'a str,
    class: BackgroundJobFailureClass,
    now: chrono::DateTime<Utc>,
) -> RecoveryFailureInput<'a> {
    RecoveryFailureInput {
        recovery_key,
        job_type: "document.ai.embed",
        payload: serde_json::json!({"document_id": "doc_test"}),
        dedupe_key: None,
        outbox_id: None,
        subject_kind: Some(BackgroundJobSubjectKind::Document),
        subject_id: Some("doc_test"),
        failure_class: class,
        failure_reason_code: match class {
            BackgroundJobFailureClass::Patient => "ai_provider_unavailable",
            _ => "embed_timeout",
        },
        error_message: "scripted failure",
        apalis_attempts: 2,
        next_retry_at: Some(now - Duration::seconds(10)),
        now,
    }
}

fn dead_letter(
    payload: serde_json::Value,
    failed_at: chrono::DateTime<Utc>,
) -> DeadLetterInsert<'static> {
    DeadLetterInsert {
        job_type: "document.ai.embed",
        payload,
        dedupe_key: None,
        failure_reason_code: Some("recovery_attempts_exhausted"),
        error_message: "cap",
        attempts: 3,
        failed_at,
    }
}

fn replay(
    id: BackgroundJobRecoveryId,
    lease_owner: &str,
    next_retry_at: chrono::DateTime<Utc>,
    now: chrono::DateTime<Utc>,
) -> RecoveryReplay<'_> {
    RecoveryReplay {
        id,
        job_type: "document.ai.embed",
        payload: serde_json::json!({"document_id": "doc_test"}),
        dedupe_key: None,
        lease_owner,
        next_retry_at,
        now,
    }
}

async fn outbox_count(db: &TestDb) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM job_outbox WHERE job_type = 'document.ai.embed'")
        .fetch_one(db.pool())
        .await
        .unwrap()
}

#[tokio::test]
async fn stale_claimant_cannot_replay_after_lease_takeover() {
    let db = TestDb::new().await;
    let repo = PgBackgroundJobRecoveryRepository::new(db.pool().clone());
    let now = Utc::now();
    let row = repo
        .upsert_waiting_failure(failure_input(
            "rk:takeover",
            BackgroundJobFailureClass::Patient,
            now,
        ))
        .await
        .unwrap();

    let claimed = repo
        .claim_due(now, "worker-a", now + Duration::seconds(30), 10)
        .await
        .unwrap();
    assert_eq!(claimed.len(), 1);

    let after_expiry = now + Duration::seconds(60);
    let reclaimed = repo
        .claim_due(
            after_expiry,
            "worker-b",
            after_expiry + Duration::seconds(30),
            10,
        )
        .await
        .unwrap();
    assert_eq!(reclaimed.len(), 1);

    let stale = repo
        .replay_via_outbox(replay(
            row.id,
            "worker-a",
            after_expiry + Duration::seconds(60),
            after_expiry,
        ))
        .await;
    assert!(stale.is_err(), "expired claimant must not replay");
    assert_eq!(outbox_count(&db).await, 0, "stale replay must roll back");

    let owner = repo
        .replay_via_outbox(replay(
            row.id,
            "worker-b",
            after_expiry + Duration::seconds(900),
            after_expiry,
        ))
        .await;
    assert!(owner.is_ok(), "current lease owner replays normally");
    assert_eq!(outbox_count(&db).await, 1);
}

#[tokio::test]
async fn resolved_rows_cannot_be_resurrected_by_a_stale_replay() {
    let db = TestDb::new().await;
    let repo = PgBackgroundJobRecoveryRepository::new(db.pool().clone());
    let now = Utc::now();
    let row = repo
        .upsert_waiting_failure(failure_input(
            "rk:resolved",
            BackgroundJobFailureClass::Retryable,
            now,
        ))
        .await
        .unwrap();
    repo.claim_due(now, "worker-a", now + Duration::seconds(30), 10)
        .await
        .unwrap();
    repo.mark_resolved("rk:resolved", now + Duration::seconds(5))
        .await
        .unwrap();

    let stale = repo
        .replay_via_outbox(replay(
            row.id,
            "worker-a",
            now + Duration::seconds(900),
            now + Duration::seconds(10),
        ))
        .await;
    assert!(stale.is_err());
    assert_eq!(outbox_count(&db).await, 0);

    let status: String =
        sqlx::query_scalar("SELECT status FROM background_job_recoveries WHERE id = $1")
            .bind(row.id.into_uuid())
            .fetch_one(db.pool())
            .await
            .unwrap();
    assert_eq!(status, "resolved");
}

#[tokio::test]
async fn stale_terminalize_cannot_dead_letter_a_row_that_turned_patient() {
    let db = TestDb::new().await;
    let repo = PgBackgroundJobRecoveryRepository::new(db.pool().clone());
    let now = Utc::now();
    let row = repo
        .upsert_waiting_failure(failure_input(
            "rk:turned-patient",
            BackgroundJobFailureClass::Retryable,
            now,
        ))
        .await
        .unwrap();
    repo.claim_due(now, "worker-a", now + Duration::seconds(30), 10)
        .await
        .unwrap();

    repo.upsert_waiting_failure(failure_input(
        "rk:turned-patient",
        BackgroundJobFailureClass::Patient,
        now + Duration::seconds(5),
    ))
    .await
    .unwrap();

    let stale = repo
        .mark_recovery_terminal(
            row.id,
            "recovery_attempts_exhausted",
            "cap",
            "worker-a",
            dead_letter(serde_json::json!({"document_id": "doc_test"}), now),
            now + Duration::seconds(10),
        )
        .await;
    assert!(stale.is_err(), "stale claimant must not terminalize");

    let (status, class): (String, String) =
        sqlx::query_as("SELECT status, failure_class FROM background_job_recoveries WHERE id = $1")
            .bind(row.id.into_uuid())
            .fetch_one(db.pool())
            .await
            .unwrap();
    assert_eq!(status, "waiting");
    assert_eq!(class, "patient");

    let dlq: i64 = sqlx::query_scalar("SELECT count(*) FROM dead_letter_jobs")
        .fetch_one(db.pool())
        .await
        .unwrap();
    assert_eq!(dlq, 0);
}
