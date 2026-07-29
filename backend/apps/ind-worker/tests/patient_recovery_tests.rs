#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use chrono::{Duration, SubsecRound, Utc};
use ind_application::repos::background_job_recovery::{
    BackgroundJobRecoveryRepository, RecoveryFailureInput,
};
use ind_domain::{
    BackgroundJobFailureClass, BackgroundJobRecoveryStatus, BackgroundJobSubjectKind,
    patient_backoff,
};
use ind_persistence::repos::PgBackgroundJobRecoveryRepository;
use ind_test_support::TestDb;
use ind_worker::recovery_handler::{RecordedFailure, record_patient_failure};
use ind_worker::recovery_sweeper::sweep_background_recoveries;

fn repo(db: &TestDb) -> Arc<dyn BackgroundJobRecoveryRepository> {
    Arc::new(PgBackgroundJobRecoveryRepository::new(db.pool().clone()))
}

fn patient_failure<'a>(
    recovery_key: &'a str,
    payload: &'a serde_json::Value,
    now: chrono::DateTime<Utc>,
) -> RecordedFailure<'a> {
    RecordedFailure {
        recovery_key,
        job_type: "document.ai.embed",
        payload: payload.clone(),
        dedupe_key: None,
        outbox_id: None,
        subject_kind: None,
        subject_id: None,
        failure_reason_code: ind_domain::AI_PROVIDER_UNAVAILABLE,
        error_message: "provider unavailable: connection refused",
        attempt: 1,
        now,
    }
}

async fn seed_waiting_row(
    repo: &Arc<dyn BackgroundJobRecoveryRepository>,
    recovery_key: &str,
    class: BackgroundJobFailureClass,
    due_at: chrono::DateTime<Utc>,
    now: chrono::DateTime<Utc>,
) {
    repo.upsert_waiting_failure(RecoveryFailureInput {
        recovery_key,
        job_type: "document.ai.embed",
        payload: serde_json::json!({"document_id": "doc_test"}),
        dedupe_key: None,
        outbox_id: None,
        subject_kind: Some(BackgroundJobSubjectKind::Document),
        subject_id: Some("doc_test"),
        failure_class: class,
        failure_reason_code: match class {
            BackgroundJobFailureClass::Patient => ind_domain::AI_PROVIDER_UNAVAILABLE,
            _ => "embed_timeout",
        },
        error_message: "scripted failure",
        apalis_attempts: 2,
        next_retry_at: Some(due_at),
        now,
    })
    .await
    .unwrap();
}

async fn force_recovery_attempts(db: &TestDb, key: &str, attempts: i32) {
    sqlx::query(
        "UPDATE background_job_recoveries SET recovery_attempts = $1 WHERE recovery_key = $2",
    )
    .bind(attempts)
    .bind(key)
    .execute(db.pool())
    .await
    .unwrap();
}

#[tokio::test]
async fn record_patient_failure_parks_a_waiting_patient_row_on_first_attempt() {
    let db = TestDb::new().await;
    let repo = repo(&db);
    let now = Utc::now();
    let payload = serde_json::json!({"document_id": "doc_test"});

    record_patient_failure(&repo, patient_failure("rk:patient-park", &payload, now))
        .await
        .unwrap();

    let rows = repo.list_active(Default::default(), 10).await.unwrap();
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.status, BackgroundJobRecoveryStatus::Waiting);
    assert_eq!(row.failure_class, BackgroundJobFailureClass::Patient);
    assert_eq!(row.failure_reason_code, ind_domain::AI_PROVIDER_UNAVAILABLE);
    assert_eq!(row.apalis_attempts, 1);
    assert_eq!(row.recovery_attempts, 0);
    let stored_next = row
        .next_retry_at
        .expect("patient row keeps a retry schedule");
    let expected = now + Duration::from_std(patient_backoff(0)).unwrap();
    assert!(
        (stored_next - expected).num_milliseconds().abs() < 5,
        "first park waits one minute"
    );
}

#[tokio::test]
async fn record_patient_failure_reparks_on_the_sparse_schedule() {
    let db = TestDb::new().await;
    let repo = repo(&db);
    let now = Utc::now();
    let payload = serde_json::json!({"document_id": "doc_test"});

    record_patient_failure(&repo, patient_failure("rk:patient-pace", &payload, now))
        .await
        .unwrap();
    force_recovery_attempts(&db, "rk:patient-pace", 3).await;

    let later = now + Duration::seconds(4_000);
    record_patient_failure(&repo, patient_failure("rk:patient-pace", &payload, later))
        .await
        .unwrap();

    let rows = repo.list_active(Default::default(), 10).await.unwrap();
    let stored_next = rows[0].next_retry_at.unwrap();
    let expected = later + Duration::from_std(patient_backoff(3)).unwrap();
    assert!(
        (stored_next - expected).num_milliseconds().abs() < 5,
        "re-park after 3 replays must wait one hour, got {stored_next} vs {expected}"
    );
}

#[tokio::test]
async fn sweeper_replays_patient_rows_past_the_cap_and_never_dead_letters() {
    let db = TestDb::new().await;
    let repo = repo(&db);
    let now = Utc::now();
    seed_waiting_row(
        &repo,
        "rk:patient-cap",
        BackgroundJobFailureClass::Patient,
        now - Duration::seconds(10),
        now,
    )
    .await;
    force_recovery_attempts(&db, "rk:patient-cap", 7).await;

    sweep_background_recoveries(&repo, "sweeper-test", 3, 50, 60, now).await;

    let (status, attempts): (String, i32) = sqlx::query_as(
        "SELECT status, recovery_attempts FROM background_job_recoveries WHERE recovery_key = $1",
    )
    .bind("rk:patient-cap")
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(status, "waiting", "patient rows must never terminalize");
    assert_eq!(attempts, 8);

    let dlq: i64 = sqlx::query_scalar("SELECT count(*) FROM dead_letter_jobs")
        .fetch_one(db.pool())
        .await
        .unwrap();
    assert_eq!(dlq, 0);

    let outbox: i64 =
        sqlx::query_scalar("SELECT count(*) FROM job_outbox WHERE job_type = 'document.ai.embed'")
            .fetch_one(db.pool())
            .await
            .unwrap();
    assert_eq!(outbox, 1, "patient replay still goes through the outbox");
}

#[tokio::test]
async fn sweeper_replay_defers_reclaim_beyond_the_job_execution_window() {
    let db = TestDb::new().await;
    let repo = repo(&db);
    // Postgres stores microseconds; an untruncated nanosecond clock loses its
    // remainder on the round-trip and the 900s delta reads back as 899.
    let now = Utc::now().trunc_subsecs(6);
    seed_waiting_row(
        &repo,
        "rk:patient-inflight",
        BackgroundJobFailureClass::Patient,
        now - Duration::seconds(10),
        now,
    )
    .await;
    seed_waiting_row(
        &repo,
        "rk:retryable-inflight",
        BackgroundJobFailureClass::Retryable,
        now - Duration::seconds(10),
        now,
    )
    .await;

    sweep_background_recoveries(&repo, "sweeper-test", 3, 50, 60, now).await;

    let rows: Vec<(String, Option<chrono::DateTime<Utc>>)> = sqlx::query_as(
        "SELECT recovery_key, next_retry_at FROM background_job_recoveries ORDER BY recovery_key",
    )
    .fetch_all(db.pool())
    .await
    .unwrap();
    for (key, next) in rows {
        let deferred_secs = (next.unwrap() - now).num_seconds();
        assert!(
            deferred_secs >= 900,
            "{key}: replay re-claim must wait out the 300s AI request timeout, got +{deferred_secs}s"
        );
    }
}

#[tokio::test]
async fn sweeper_still_dead_letters_non_patient_rows_at_the_cap() {
    let db = TestDb::new().await;
    let repo = repo(&db);
    let now = Utc::now();
    seed_waiting_row(
        &repo,
        "rk:retryable-cap",
        BackgroundJobFailureClass::Retryable,
        now - Duration::seconds(10),
        now,
    )
    .await;
    force_recovery_attempts(&db, "rk:retryable-cap", 3).await;

    sweep_background_recoveries(&repo, "sweeper-test", 3, 50, 60, now).await;

    let status: String =
        sqlx::query_scalar("SELECT status FROM background_job_recoveries WHERE recovery_key = $1")
            .bind("rk:retryable-cap")
            .fetch_one(db.pool())
            .await
            .unwrap();
    assert_eq!(status, "terminal");

    let dlq: i64 = sqlx::query_scalar("SELECT count(*) FROM dead_letter_jobs")
        .fetch_one(db.pool())
        .await
        .unwrap();
    assert_eq!(dlq, 1);
}
