#![allow(clippy::unwrap_used)]

use chrono::Utc;
use ind_application::repos::background_job_recovery::{
    BackgroundJobRecoveryRepository, DeadLetterInsert, RecoveryFailureInput,
};
use ind_application::repos::content_vector::ContentVectorRepository;
use ind_application::repos::embedding_backfill::{
    EffectiveEmbeddingTarget, EmbeddingBackfillRepository,
};
use ind_domain::{
    BackgroundJobFailureClass, BackgroundJobSubjectKind, ContentVector, ContentVectorId,
    SearchSectionKind, job_types,
};
use ind_persistence::repos::{
    PgBackgroundJobRecoveryRepository, PgContentVectorRepository, PgEmbeddingBackfillRepository,
};
use ind_test_support::{
    DocumentFactory, LibraryEntryFactory, TestDb, UserFactory, test_mila_defaults,
};

#[tokio::test]
async fn automatic_repair_selects_a_document_with_vectors_for_the_wrong_model() {
    let db = TestDb::new().await;
    let user = UserFactory::new().insert(db.pool()).await;
    let document = DocumentFactory::new(user.id).insert(db.pool()).await;
    LibraryEntryFactory::new(user.id, document.id)
        .insert(db.pool())
        .await;
    insert_readable_asset(&db, document.id).await;

    PgContentVectorRepository::new(db.pool().clone())
        .replace_for_document(
            document.id,
            &[ContentVector {
                id: ContentVectorId::new(),
                document_id: document.id,
                user_id: user.id,
                embedding_model: "old-model".into(),
                embedding_dim: 768,
                section_kind: SearchSectionKind::Item,
                section_key: String::new(),
                chunk_index: 0,
                content: "existing vector".into(),
                token_count: 2,
                search_config: "english".into(),
                embedding: vec![0.0; 768],
                created_at: Utc::now(),
            }],
        )
        .await
        .unwrap();

    let repo = PgEmbeddingBackfillRepository::new(db.pool().clone());
    let mut defaults = test_mila_defaults();
    defaults.enabled = true;
    let queued = repo
        .enqueue_target_vector_repairs(&defaults, 10)
        .await
        .unwrap();

    assert_eq!(queued, 1);
    let queued_document: String = sqlx::query_scalar(
        "SELECT payload->>'document_id' FROM job_outbox WHERE job_type = 'document.ai.embed'",
    )
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(queued_document, document.id.to_string());
}

#[tokio::test]
async fn explicit_retry_replays_a_suppressed_embedding_failure_atomically() {
    let db = TestDb::new().await;
    let user = UserFactory::new().insert(db.pool()).await;
    let document = DocumentFactory::new(user.id).insert(db.pool()).await;
    LibraryEntryFactory::new(user.id, document.id)
        .insert(db.pool())
        .await;
    insert_readable_asset(&db, document.id).await;

    let now = Utc::now();
    let payload = serde_json::json!({"document_id": document.id});
    let dedupe_key = format!("{}:{}", job_types::DOCUMENT_AI_EMBED, document.id);
    let recovery_key = format!("dedupe:{dedupe_key}");
    let subject_id = document.id.to_string();
    PgBackgroundJobRecoveryRepository::new(db.pool().clone())
        .upsert_terminal_failure(
            RecoveryFailureInput {
                recovery_key: &recovery_key,
                job_type: job_types::DOCUMENT_AI_EMBED,
                payload: payload.clone(),
                dedupe_key: Some(&dedupe_key),
                outbox_id: None,
                subject_kind: Some(BackgroundJobSubjectKind::Document),
                subject_id: Some(&subject_id),
                failure_class: BackgroundJobFailureClass::Terminal,
                failure_reason_code: "provider_rejected",
                error_message: "provider rejected the embedding",
                apalis_attempts: 2,
                next_retry_at: None,
                now,
            },
            DeadLetterInsert {
                job_type: job_types::DOCUMENT_AI_EMBED,
                payload,
                dedupe_key: Some(&dedupe_key),
                failure_reason_code: Some("provider_rejected"),
                error_message: "provider rejected the embedding",
                attempts: 2,
                failed_at: now,
            },
        )
        .await
        .unwrap();

    let repo = PgEmbeddingBackfillRepository::new(db.pool().clone());
    let target = EffectiveEmbeddingTarget {
        embedding_model: "test-embedding".into(),
        embedding_dim: 768,
    };
    assert_eq!(
        repo.enqueue_user_vector_repairs(user.id, &target, 10)
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        repo.retry_user_vector_repairs(user.id, &target, 10)
            .await
            .unwrap(),
        1
    );

    let replay: (String, uuid::Uuid, uuid::Uuid) = sqlx::query_as(
        "SELECT recovery.status, dead_letter.replay_outbox_id, outbox.id \
         FROM background_job_recoveries recovery \
         JOIN dead_letter_jobs dead_letter ON dead_letter.original_dedupe_key = recovery.dedupe_key \
         JOIN job_outbox outbox ON outbox.id = dead_letter.replay_outbox_id \
         WHERE recovery.recovery_key = $1",
    )
    .bind(recovery_key)
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(replay.0, "resolved");
    assert_eq!(replay.1, replay.2);
}

#[tokio::test]
async fn explicit_retry_prioritizes_suppressed_failures_within_its_batch() {
    let db = TestDb::new().await;
    let user = UserFactory::new().insert(db.pool()).await;
    let failed = DocumentFactory::new(user.id).insert(db.pool()).await;
    LibraryEntryFactory::new(user.id, failed.id)
        .insert(db.pool())
        .await;
    insert_readable_asset(&db, failed.id).await;
    let recovery_key = insert_terminal_embedding_failure(&db, failed.id).await;

    for _ in 0..10 {
        let document = DocumentFactory::new(user.id).insert(db.pool()).await;
        LibraryEntryFactory::new(user.id, document.id)
            .insert(db.pool())
            .await;
        insert_readable_asset(&db, document.id).await;
    }

    let repo = PgEmbeddingBackfillRepository::new(db.pool().clone());
    let target = EffectiveEmbeddingTarget {
        embedding_model: "test-embedding".into(),
        embedding_dim: 768,
    };
    assert_eq!(
        repo.retry_user_vector_repairs(user.id, &target, 10)
            .await
            .unwrap(),
        10
    );

    let status: String =
        sqlx::query_scalar("SELECT status FROM background_job_recoveries WHERE recovery_key = $1")
            .bind(recovery_key)
            .fetch_one(db.pool())
            .await
            .unwrap();
    assert_eq!(status, "resolved");
}

#[tokio::test]
async fn explicit_retry_locks_dead_letters_before_rearming_outbox() {
    let db = TestDb::new().await;
    let user = UserFactory::new().insert(db.pool()).await;
    let document = DocumentFactory::new(user.id).insert(db.pool()).await;
    LibraryEntryFactory::new(user.id, document.id)
        .insert(db.pool())
        .await;
    insert_readable_asset(&db, document.id).await;
    insert_terminal_embedding_failure(&db, document.id).await;

    let dedupe_key = format!("{}:{}", job_types::DOCUMENT_AI_EMBED, document.id);
    sqlx::query(
        "INSERT INTO job_outbox (id, job_type, payload, dedupe_key, available_at, created_at) \
         VALUES ($1, $2, $3, $4, now(), now())",
    )
    .bind(uuid::Uuid::now_v7())
    .bind(job_types::DOCUMENT_AI_EMBED)
    .bind(serde_json::json!({"document_id": document.id}))
    .bind(&dedupe_key)
    .execute(db.pool())
    .await
    .unwrap();

    let mut replay_tx = db.pool().begin().await.unwrap();
    sqlx::query("SELECT id FROM dead_letter_jobs WHERE original_dedupe_key = $1 FOR UPDATE")
        .bind(&dedupe_key)
        .fetch_one(&mut *replay_tx)
        .await
        .unwrap();

    let repo = PgEmbeddingBackfillRepository::new(db.pool().clone());
    let target = EffectiveEmbeddingTarget {
        embedding_model: "test-embedding".into(),
        embedding_dim: 768,
    };
    let retry = tokio::spawn(async move {
        repo.retry_user_vector_repairs(user.id, &target, 10)
            .await
            .unwrap()
    });
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    tokio::time::timeout(
        std::time::Duration::from_millis(500),
        sqlx::query("UPDATE job_outbox SET available_at = now() WHERE dedupe_key = $1")
            .bind(&dedupe_key)
            .execute(&mut *replay_tx),
    )
    .await
    .expect("retry must wait for the dead-letter lock before locking outbox")
    .unwrap();
    replay_tx.commit().await.unwrap();
    assert_eq!(retry.await.unwrap(), 1);
}

#[tokio::test]
async fn status_counts_only_repairable_library_documents() {
    let db = TestDb::new().await;
    let user = UserFactory::new().insert(db.pool()).await;
    let active = DocumentFactory::new(user.id).insert(db.pool()).await;
    LibraryEntryFactory::new(user.id, active.id)
        .insert(db.pool())
        .await;
    insert_readable_asset(&db, active.id).await;

    let deleted = DocumentFactory::new(user.id).insert(db.pool()).await;
    let deleted_entry = LibraryEntryFactory::new(user.id, deleted.id)
        .insert(db.pool())
        .await;
    insert_readable_asset(&db, deleted.id).await;
    sqlx::query("UPDATE library_entries SET deleted_at = now() WHERE id = $1")
        .bind(deleted_entry.id.into_uuid())
        .execute(db.pool())
        .await
        .unwrap();

    let repo = PgEmbeddingBackfillRepository::new(db.pool().clone());
    assert_eq!(repo.count_eligible_items(user.id).await.unwrap(), 1);
    assert_eq!(
        repo.count_indexed_items(user.id, "test-embedding", 768)
            .await
            .unwrap(),
        0
    );
}

async fn insert_readable_asset(db: &TestDb, document_id: ind_domain::DocumentId) {
    sqlx::query(
        "INSERT INTO archive_assets \
         (id, document_id, asset_kind, s3_key, s3_bucket, content_type, size_bytes, created_at, status) \
         VALUES ($1, $2, 'readable_html', $3, $4, 'text/html', 64, now(), 'completed')",
    )
    .bind(uuid::Uuid::now_v7())
    .bind(document_id.into_uuid())
    .bind(format!("documents/{document_id}/readable.html"))
    .bind(db.bucket())
    .execute(db.pool())
    .await
    .unwrap();
}

async fn insert_terminal_embedding_failure(
    db: &TestDb,
    document_id: ind_domain::DocumentId,
) -> String {
    let now = Utc::now();
    let payload = serde_json::json!({"document_id": document_id});
    let dedupe_key = format!("{}:{}", job_types::DOCUMENT_AI_EMBED, document_id);
    let recovery_key = format!("dedupe:{dedupe_key}");
    let subject_id = document_id.to_string();
    PgBackgroundJobRecoveryRepository::new(db.pool().clone())
        .upsert_terminal_failure(
            RecoveryFailureInput {
                recovery_key: &recovery_key,
                job_type: job_types::DOCUMENT_AI_EMBED,
                payload: payload.clone(),
                dedupe_key: Some(&dedupe_key),
                outbox_id: None,
                subject_kind: Some(BackgroundJobSubjectKind::Document),
                subject_id: Some(&subject_id),
                failure_class: BackgroundJobFailureClass::Terminal,
                failure_reason_code: "provider_rejected",
                error_message: "provider rejected the embedding",
                apalis_attempts: 2,
                next_retry_at: None,
                now,
            },
            DeadLetterInsert {
                job_type: job_types::DOCUMENT_AI_EMBED,
                payload,
                dedupe_key: Some(&dedupe_key),
                failure_reason_code: Some("provider_rejected"),
                error_message: "provider rejected the embedding",
                attempts: 2,
                failed_at: now,
            },
        )
        .await
        .unwrap();
    recovery_key
}
