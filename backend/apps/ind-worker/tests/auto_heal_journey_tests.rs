#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use ind_domain::job_types;
use ind_test_support::{
    DocumentFactory, LibraryEntryFactory, StorageBackedMockRenderer, TestDb, UserFactory,
    test_mila_defaults,
};
use ind_worker::context::WorkerServicesBuilder;

async fn recovery_context(db: &TestDb) -> ind_worker::context::RecoveryJobDeps {
    let renderer = Arc::new(StorageBackedMockRenderer::new(db.storage().await));
    WorkerServicesBuilder::new(
        db.pool().clone(),
        renderer,
        None,
        db.bucket().to_string(),
        test_mila_defaults(),
        ind_egress::EgressPolicy::permissive(),
        None,
    )
    .expect("worker services build")
    .with_worker_id("auto-heal-journey")
    .without_email_services()
    .build()
    .recovery_jobs()
}

#[tokio::test]
async fn auto_heal_leases_real_maintenance_and_repairs_missing_embeddings_once() {
    let db = TestDb::new().await;
    let user = UserFactory::new().insert(db.pool()).await;
    let document = DocumentFactory::new(user.id).insert(db.pool()).await;
    LibraryEntryFactory::new(user.id, document.id)
        .insert(db.pool())
        .await;
    sqlx::query(
        "INSERT INTO archive_assets \
         (id, document_id, asset_kind, s3_key, s3_bucket, content_type, size_bytes, created_at, status) \
         VALUES ($1, $2, 'readable_html', $3, $4, 'text/html', 64, now(), 'completed')",
    )
    .bind(uuid::Uuid::now_v7())
    .bind(document.id.into_uuid())
    .bind(format!("documents/{}/readable.html", document.id))
    .bind(db.bucket())
    .execute(db.pool())
    .await
    .unwrap();

    let context = recovery_context(&db).await;
    ind_worker::auto_heal::run_auto_heal_once(&context).await;

    let queued: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM job_outbox WHERE job_type = $1 AND payload->>'document_id' = $2",
    )
    .bind(job_types::DOCUMENT_AI_EMBED)
    .bind(document.id.to_string())
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(queued, 1);

    let completed: Vec<(String, bool, bool, bool)> = sqlx::query_as(
        "SELECT task_name, last_completed_at IS NOT NULL, lease_owner IS NULL, \
         next_run_at > now() FROM maintenance_tasks ORDER BY task_name",
    )
    .fetch_all(db.pool())
    .await
    .unwrap();
    assert_eq!(
        completed,
        vec![
            ("embedding.repair".into(), true, true, true),
            ("integrity.check".into(), true, true, true),
        ]
    );

    ind_worker::auto_heal::run_auto_heal_once(&context).await;
    let queued_after_second_sweep: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM job_outbox WHERE job_type = $1 AND payload->>'document_id' = $2",
    )
    .bind(job_types::DOCUMENT_AI_EMBED)
    .bind(document.id.to_string())
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(queued_after_second_sweep, 1);
}
