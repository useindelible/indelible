use std::sync::Arc;

use ind_application::error::AppError;
use ind_application::storage::ObjectStorage;
use ind_domain::{AccountStoragePurgeJob, GenericJobEnvelope, job_types};
use sqlx::PgPool;
use tracing::info;

const LIST_PAGE_SIZE: i32 = 1000;
/// Upper bound on listing rounds per prefix. A prefix that keeps returning
/// objects past this many delete rounds is not converging (deletes silently
/// failing, or a writer racing the purge) and must surface as an error rather
/// than loop forever.
const MAX_SWEEP_ROUNDS: usize = 10_000;

pub async fn dispatch_generic_job(
    ctx: &crate::context::WorkerContext,
    envelope: GenericJobEnvelope,
) -> Result<Option<()>, AppError> {
    match envelope.job_type.as_str() {
        job_types::ACCOUNT_STORAGE_PURGE => {
            let job: AccountStoragePurgeJob = serde_json::from_value(envelope.payload)
                .map_err(|e| AppError::Repository(Box::new(e)))?;
            handle_account_storage_purge(
                ctx.object_storage.as_ref(),
                &ctx.pool,
                &job,
                LIST_PAGE_SIZE,
            )
            .await?;
            Ok(Some(()))
        }
        _ => Ok(None),
    }
}

/// Removes every object under the purged account's prefixes, plus harvested
/// legacy keys no prefix covers, then removes its own outbox row so the last
/// payload naming the account does not outlive the cleanup.
///
/// Each prefix is re-listed from scratch after deleting a page: continuation
/// tokens are not guaranteed to paginate correctly over a listing that is
/// being mutated, so the only trustworthy terminal state is an empty listing.
/// Deleting an absent key is a no-op in S3, which keeps retries idempotent.
///
/// Storage being unavailable or any delete failing is an error: the account's
/// database rows are already gone and the keys cannot be re-harvested, so this
/// job must retry (or land in recovery) rather than report a clean bucket it
/// did not produce.
pub async fn handle_account_storage_purge(
    storage: Option<&Arc<dyn ObjectStorage>>,
    pool: &PgPool,
    job: &AccountStoragePurgeJob,
    page_size: i32,
) -> Result<(), AppError> {
    let storage = storage.ok_or_else(|| AppError::ExternalService {
        service: "object-storage".into(),
        message: "object storage is not configured; cannot purge account keys".into(),
    })?;

    let mut deleted = 0usize;
    for prefix in &job.prefixes {
        let mut rounds = 0usize;
        loop {
            let page = storage.list_objects_page(prefix, None, page_size).await?;
            if page.objects.is_empty() {
                break;
            }
            for object in &page.objects {
                storage.delete(&object.key).await?;
                deleted += 1;
            }
            rounds += 1;
            if rounds >= MAX_SWEEP_ROUNDS {
                return Err(AppError::ExternalService {
                    service: "object-storage".into(),
                    message: format!(
                        "prefix {prefix} still lists objects after {MAX_SWEEP_ROUNDS} delete rounds"
                    ),
                });
            }
        }
    }
    for key in &job.residual_keys {
        storage.delete(key).await?;
        deleted += 1;
    }

    // The bucket is clean; the job that carried the prefixes has served its
    // purpose and is the last row naming the purged account.
    sqlx::query("DELETE FROM job_outbox WHERE dedupe_key = $1")
        .bind(format!("account-storage-purge:{}", job.user_id))
        .execute(pool)
        .await
        .map_err(|e| AppError::Repository(Box::new(e)))?;

    info!(
        user_id = %job.user_id,
        objects_deleted = deleted,
        prefixes = job.prefixes.len(),
        residual_keys = job.residual_keys.len(),
        "purged account object storage"
    );
    Ok(())
}
