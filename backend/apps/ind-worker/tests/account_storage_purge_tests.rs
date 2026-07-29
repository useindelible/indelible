#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use bytes::Bytes;
use ind_application::error::AppError;
use ind_application::storage::{ObjectStorage, UploadResult};
use ind_domain::{AccountStoragePurgeJob, UserId};
use ind_test_support::TestDb;
use ind_worker::jobs::account_storage_purge::handle_account_storage_purge;

fn user_id(suffix: &str) -> UserId {
    format!("usr_01890000-0000-7000-8000-0000000000{suffix}")
        .parse()
        .unwrap()
}

async fn put(storage: &Arc<dyn ObjectStorage>, key: &str) {
    storage
        .upload(key, "text/plain", Bytes::from_static(b"x"))
        .await
        .unwrap();
}

#[tokio::test]
async fn purge_job_removes_prefixed_and_residual_keys_and_spares_others() {
    let db = TestDb::new().await;
    let storage = db.storage().await;
    let victim = user_id("aa");
    let other = user_id("bb");

    put(&storage, &format!("{victim}/avatars/a.webp")).await;
    put(&storage, &format!("tts/{victim}/x.mp3")).await;
    put(&storage, "legacy/one/readable.html").await;
    put(&storage, &format!("{other}/avatars/keep.webp")).await;

    let job = AccountStoragePurgeJob {
        user_id: victim,
        prefixes: vec![format!("{victim}/"), format!("tts/{victim}/")],
        residual_keys: vec!["legacy/one/readable.html".to_string()],
    };
    handle_account_storage_purge(Some(&storage), db.pool(), &job, 1000)
        .await
        .unwrap();

    assert!(
        storage
            .list_keys(&format!("{victim}/"))
            .await
            .unwrap()
            .is_empty(),
        "victim avatar prefix must be empty"
    );
    assert!(
        storage
            .list_keys(&format!("tts/{victim}/"))
            .await
            .unwrap()
            .is_empty(),
        "victim tts prefix must be empty"
    );
    assert!(
        storage.list_keys("legacy/one/").await.unwrap().is_empty(),
        "residual key must be deleted"
    );
    assert_eq!(
        storage.list_keys(&format!("{other}/")).await.unwrap().len(),
        1,
        "another user's objects must survive"
    );
}

#[tokio::test]
async fn purge_job_is_idempotent_under_retry() {
    let db = TestDb::new().await;
    let storage = db.storage().await;
    let victim = user_id("cc");
    put(&storage, &format!("tts/{victim}/x.mp3")).await;

    let job = AccountStoragePurgeJob {
        user_id: victim,
        prefixes: vec![format!("tts/{victim}/")],
        residual_keys: vec!["already/gone.html".to_string()],
    };
    handle_account_storage_purge(Some(&storage), db.pool(), &job, 1000)
        .await
        .unwrap();
    // A retry over an already-clean bucket must succeed, not error.
    handle_account_storage_purge(Some(&storage), db.pool(), &job, 1000)
        .await
        .unwrap();
}

#[tokio::test]
async fn purge_job_fails_when_storage_is_unavailable() {
    let db = TestDb::new().await;
    let victim = user_id("dd");
    let job = AccountStoragePurgeJob {
        user_id: victim,
        prefixes: vec![format!("tts/{victim}/")],
        residual_keys: vec![],
    };
    let err = handle_account_storage_purge(None, db.pool(), &job, 1000)
        .await
        .unwrap_err();
    assert!(
        matches!(err, AppError::ExternalService { .. }),
        "missing storage must be a retryable external-service error, got {err:?}"
    );
}

struct FailingDelete;

#[async_trait::async_trait]
impl ObjectStorage for FailingDelete {
    async fn upload(&self, _: &str, _: &str, _: Bytes) -> Result<UploadResult, AppError> {
        Err(AppError::ExternalService {
            service: "object-storage".into(),
            message: "not used by this test double".into(),
        })
    }
    async fn presigned_url(&self, _: &str, _: std::time::Duration) -> Result<String, AppError> {
        Err(AppError::ExternalService {
            service: "object-storage".into(),
            message: "not used by this test double".into(),
        })
    }
    async fn exists(&self, _: &str) -> Result<bool, AppError> {
        Err(AppError::ExternalService {
            service: "object-storage".into(),
            message: "not used by this test double".into(),
        })
    }
    async fn get_object(&self, _: &str) -> Result<ind_application::storage::ObjectData, AppError> {
        Err(AppError::ExternalService {
            service: "object-storage".into(),
            message: "not used by this test double".into(),
        })
    }
    async fn delete(&self, _: &str) -> Result<(), AppError> {
        Err(AppError::ExternalService {
            service: "object-storage".into(),
            message: "delete refused".into(),
        })
    }
    async fn list_objects(
        &self,
        _: &str,
    ) -> Result<Vec<ind_application::storage::ObjectListEntry>, AppError> {
        Ok(vec![ind_application::storage::ObjectListEntry {
            key: "tts/usr_x/stuck.mp3".into(),
            last_modified: None,
        }])
    }
}

#[tokio::test]
async fn purge_job_propagates_delete_failures() {
    let db = TestDb::new().await;
    let storage: Arc<dyn ObjectStorage> = Arc::new(FailingDelete);
    let job = AccountStoragePurgeJob {
        user_id: user_id("ee"),
        prefixes: vec!["tts/usr_x/".to_string()],
        residual_keys: vec![],
    };
    let err = handle_account_storage_purge(Some(&storage), db.pool(), &job, 1000)
        .await
        .unwrap_err();
    assert!(
        matches!(err, AppError::ExternalService { .. }),
        "storage failure must propagate for retry, got {err:?}"
    );
}

#[tokio::test]
async fn purge_job_sweeps_across_page_boundaries_until_the_prefix_is_empty() {
    let db = TestDb::new().await;
    let storage = db.storage().await;
    let victim = user_id("f0");
    for i in 0..5 {
        put(&storage, &format!("tts/{victim}/chunk-{i}.mp3")).await;
    }

    let job = AccountStoragePurgeJob {
        user_id: victim,
        prefixes: vec![format!("tts/{victim}/")],
        residual_keys: vec![],
    };
    // Page size two forces multiple listing rounds while deletions mutate the
    // listing underneath — the loop must restart from scratch until empty.
    handle_account_storage_purge(Some(&storage), db.pool(), &job, 2)
        .await
        .unwrap();

    assert!(
        storage
            .list_keys(&format!("tts/{victim}/"))
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn purge_job_removes_its_own_outbox_row_after_success() {
    let db = TestDb::new().await;
    let storage = db.storage().await;
    let victim = user_id("f1");
    let dedupe = format!("account-storage-purge:{victim}");
    sqlx::query(
        "INSERT INTO job_outbox (id, job_type, payload, dedupe_key, available_at, created_at) \
         VALUES (gen_random_uuid(), 'account.storage_purge', '{}'::jsonb, $1, now(), now())",
    )
    .bind(&dedupe)
    .execute(db.pool())
    .await
    .unwrap();

    let job = AccountStoragePurgeJob {
        user_id: victim,
        prefixes: vec![format!("tts/{victim}/")],
        residual_keys: vec![],
    };
    handle_account_storage_purge(Some(&storage), db.pool(), &job, 1000)
        .await
        .unwrap();

    let remaining: i64 =
        sqlx::query_scalar("SELECT count(*) FROM job_outbox WHERE dedupe_key = $1")
            .bind(&dedupe)
            .fetch_one(db.pool())
            .await
            .unwrap();
    assert_eq!(
        remaining, 0,
        "the cleanup job must not outlive its own success"
    );
}
