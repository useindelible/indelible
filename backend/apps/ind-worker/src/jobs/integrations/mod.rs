pub mod notion;
mod obsidian;
mod readwise;

use ind_application::error::AppError;
use ind_domain::GenericJobEnvelope;

use crate::context::IntegrationJobDeps;

const HANDLED_JOB_TYPES: &[&str] = &[
    "integration.notion.export_document",
    "integration.obsidian.sync_connection",
    "integration.notion.sync_connection",
    "import.readwise",
];

pub fn handles_job_type(job_type: &str) -> bool {
    HANDLED_JOB_TYPES.contains(&job_type)
}

pub async fn dispatch_envelope(
    ctx: &IntegrationJobDeps,
    envelope: GenericJobEnvelope,
) -> Result<Option<()>, AppError> {
    if !handles_job_type(envelope.job_type.as_str()) {
        return Ok(None);
    }

    #[cfg(any(test, feature = "test-helpers"))]
    {
        test_hooks::record(envelope.job_type.as_str());
        if test_hooks::is_stub_mode() {
            return Ok(Some(()));
        }
    }

    let job_type = envelope.job_type.clone();
    match job_type.as_str() {
        "integration.notion.export_document" => {
            let Some(deps) = ctx.notion_job_deps.as_ref() else {
                return Err(AppError::ExternalService {
                    service: "notion".into(),
                    message:
                        "Notion export job received but Notion job dependencies are not configured"
                            .into(),
                });
            };
            let job: ind_domain::NotionExportDocumentJob = serde_json::from_value(envelope.payload)
                .map_err(|e| AppError::ExternalService {
                    service: "notion".into(),
                    message: format!("invalid export_document payload: {e}"),
                })?;
            notion::handle_export_document(deps, job).await?;
            Ok(Some(()))
        }
        "integration.notion.sync_connection" => {
            let Some(deps) = ctx.notion_job_deps.as_ref() else {
                return Err(AppError::ExternalService {
                    service: "notion".into(),
                    message:
                        "Notion sync job received but Notion job dependencies are not configured"
                            .into(),
                });
            };
            let job: ind_domain::NotionSyncConnectionJob = serde_json::from_value(envelope.payload)
                .map_err(|e| AppError::ExternalService {
                    service: "notion".into(),
                    message: format!("invalid sync_connection payload: {e}"),
                })?;
            notion::handle_sync_connection(deps, job).await?;
            Ok(Some(()))
        }
        "integration.obsidian.sync_connection" => {
            let job: ind_domain::ObsidianSyncConnectionJob =
                serde_json::from_value(envelope.payload).map_err(|e| {
                    AppError::ExternalService {
                        service: "obsidian".into(),
                        message: format!("invalid sync_connection payload: {e}"),
                    }
                })?;
            obsidian::handle_sync_connection(ctx, job).await?;
            Ok(Some(()))
        }
        "import.readwise" => {
            let job: readwise::ReadwiseImportJob = serde_json::from_value(envelope.payload)
                .map_err(|e| AppError::Repository(Box::new(e)))?;
            readwise::handle_readwise_import(ctx, job).await?;
            Ok(Some(()))
        }
        other => {
            // The job type passed `handles_job_type` (otherwise we returned
            // `Ok(None)` above), so reaching this arm means a registered job
            // type lacks a dispatch arm — a bug in this file. Returning an
            // error keeps the outbox row un-acked instead of silently
            // swallowing real work.
            tracing::error!(
                outbox_id = %envelope.outbox_id,
                job_type = other,
                "registered integration job type has no dispatch arm",
            );
            Err(AppError::ExternalService {
                service: "integrations".to_string(),
                message: format!("unhandled integration job type: {other}"),
            })
        }
    }
}

#[cfg(any(test, feature = "test-helpers"))]
#[allow(dead_code, clippy::unwrap_used)]
pub mod test_hooks {
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

    static STUB_CALLS: AtomicU64 = AtomicU64::new(0);
    static LAST_JOB_TYPE: Mutex<Option<String>> = Mutex::new(None);
    static STUB_MODE: AtomicBool = AtomicBool::new(false);

    pub fn record(job_type: &str) {
        STUB_CALLS.fetch_add(1, Ordering::SeqCst);
        *LAST_JOB_TYPE.lock().unwrap() = Some(job_type.to_string());
    }

    pub fn stub_calls() -> u64 {
        STUB_CALLS.load(Ordering::SeqCst)
    }

    pub fn last_job_type() -> Option<String> {
        LAST_JOB_TYPE.lock().unwrap().clone()
    }

    pub fn is_stub_mode() -> bool {
        STUB_MODE.load(Ordering::SeqCst)
    }

    pub fn enable_stub_mode() {
        STUB_MODE.store(true, Ordering::SeqCst);
    }

    pub fn disable_stub_mode() {
        STUB_MODE.store(false, Ordering::SeqCst);
    }

    pub fn reset() {
        STUB_CALLS.store(0, Ordering::SeqCst);
        STUB_MODE.store(false, Ordering::SeqCst);
        *LAST_JOB_TYPE.lock().unwrap() = None;
    }
}
