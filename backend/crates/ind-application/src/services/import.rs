use std::sync::Arc;

use futures::future::BoxFuture;
use ind_domain::{UserId, job_types};

use crate::AppError;
use crate::outputs::import::ImportStatusOutput;
use crate::ports::{ImportOperations, ReadwiseImportUpload};
use crate::repos::outbox::JobOutboxRepository;

// -- ImportOperations --

pub struct ImportService {
    import_job_repo: Arc<dyn crate::repos::import_job::ImportJobRepository>,
    storage: Arc<dyn crate::storage::ObjectStorage>,
    outbox_repo: Arc<dyn JobOutboxRepository>,
}

impl ImportService {
    pub fn new(
        import_job_repo: Arc<dyn crate::repos::import_job::ImportJobRepository>,
        storage: Arc<dyn crate::storage::ObjectStorage>,
        outbox_repo: Arc<dyn JobOutboxRepository>,
    ) -> Self {
        Self {
            import_job_repo,
            storage,
            outbox_repo,
        }
    }
}

impl ImportOperations for ImportService {
    fn upload_readwise(
        &self,
        user_id: UserId,
        upload: ReadwiseImportUpload,
    ) -> BoxFuture<'_, Result<ind_domain::ImportJob, AppError>> {
        Box::pin(async move {
            let method = if upload.archive_zip.is_some() {
                ind_domain::ImportMethod::Zip
            } else {
                ind_domain::ImportMethod::Csv
            };
            let job = self
                .import_job_repo
                .create(
                    user_id,
                    ind_domain::ImportSource::ReadwiseImport,
                    method,
                    None,
                )
                .await?;

            let uid = user_id.into_uuid();
            let jid = job.id.into_uuid();
            let mut artifact_keys = serde_json::Map::new();

            if let Some(csv) = upload.library_csv {
                let key = format!("imports/{uid}/{jid}/library.csv");
                self.storage
                    .upload(
                        &key,
                        csv.content_type.as_deref().unwrap_or("text/csv"),
                        bytes::Bytes::from(csv.bytes),
                    )
                    .await?;
                artifact_keys.insert("csv_key".into(), serde_json::Value::String(key));
            }
            if let Some(zip) = upload.archive_zip {
                let key = format!("imports/{uid}/{jid}/archive.zip");
                self.storage
                    .upload(
                        &key,
                        zip.content_type.as_deref().unwrap_or("application/zip"),
                        bytes::Bytes::from(zip.bytes),
                    )
                    .await?;
                artifact_keys.insert("zip_key".into(), serde_json::Value::String(key));
            }
            if let Some(opml) = upload.feeds_opml {
                let key = format!("imports/{uid}/{jid}/feeds.opml");
                self.storage
                    .upload(
                        &key,
                        opml.content_type.as_deref().unwrap_or("application/xml"),
                        bytes::Bytes::from(opml.bytes),
                    )
                    .await?;
                artifact_keys.insert("opml_key".into(), serde_json::Value::String(key));
            }

            let artifact_json = serde_json::Value::Object(artifact_keys).to_string();
            let job = self
                .import_job_repo
                .set_raw_artifact_key(user_id, job.id, artifact_json)
                .await?;

            self.outbox_repo
                .enqueue(
                    job_types::IMPORT_READWISE,
                    serde_json::json!({ "import_job_id": job.id.to_string() }),
                    Some(format!("{}:{}", job_types::IMPORT_READWISE, job.id)),
                    chrono::Utc::now(),
                )
                .await?;

            Ok(job)
        })
    }

    fn get_status(
        &self,
        user_id: UserId,
        id: ind_domain::ImportJobId,
    ) -> BoxFuture<'_, Result<ImportStatusOutput, AppError>> {
        Box::pin(async move {
            let job = self
                .import_job_repo
                .find_by_id(user_id, id)
                .await?
                .ok_or_else(|| {
                    AppError::Domain(ind_domain::DomainError::NotFound {
                        entity: "ImportJob",
                        id: id.to_string(),
                    })
                })?;
            let items = self.import_job_repo.list_item_outcomes(id, 500, 0).await?;
            Ok(ImportStatusOutput { job, items })
        })
    }

    fn rollback(
        &self,
        user_id: UserId,
        id: ind_domain::ImportJobId,
    ) -> BoxFuture<'_, Result<(), AppError>> {
        Box::pin(async move {
            let job = self
                .import_job_repo
                .find_by_id(user_id, id)
                .await?
                .ok_or_else(|| {
                    AppError::Domain(ind_domain::DomainError::NotFound {
                        entity: "ImportJob",
                        id: id.to_string(),
                    })
                })?;
            self.import_job_repo
                .rollback_imported_library_entries(user_id, job.id)
                .await?;
            Ok(())
        })
    }

    fn list_recent(
        &self,
        user_id: UserId,
        limit: i64,
    ) -> BoxFuture<'_, Result<Vec<ImportStatusOutput>, AppError>> {
        Box::pin(async move {
            let jobs = self
                .import_job_repo
                .list_by_user(user_id, limit, None)
                .await?;
            let outputs = jobs
                .into_iter()
                .map(|job| ImportStatusOutput { job, items: vec![] })
                .collect();
            Ok(outputs)
        })
    }
}
