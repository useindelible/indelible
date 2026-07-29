use chrono::Utc;
use ind_application::error::AppError;
use ind_application::repos::integration_connection::NotionExportCursor;
use ind_domain::{DomainError, NotionExportDocumentJob, NotionSyncConnectionJob};
use ind_integrations::notion::{NotionClient, notion_settings_from_config};

use crate::context::NotionJobDeps;

use super::auth::load_notion_access_token;
use super::managed_target::{has_cached_managed_target, resolve_managed_target};
use super::support::map_notion_error;

const ITEM_EXPORT_BATCH_SIZE: i64 = 500;

pub async fn handle_sync_connection(
    deps: &NotionJobDeps,
    job: NotionSyncConnectionJob,
) -> Result<(), AppError> {
    handle_sync_connection_with_item_batch_size(deps, job, ITEM_EXPORT_BATCH_SIZE).await
}

#[cfg(any(test, feature = "test-helpers"))]
#[allow(dead_code)]
// allow: TASK-225 - exported for cross-crate integration tests via the test-helpers feature.
pub async fn handle_sync_connection_with_test_item_batch_size(
    deps: &NotionJobDeps,
    job: NotionSyncConnectionJob,
    item_batch_size: i64,
) -> Result<(), AppError> {
    handle_sync_connection_with_item_batch_size(deps, job, item_batch_size).await
}

async fn handle_sync_connection_with_item_batch_size(
    deps: &NotionJobDeps,
    job: NotionSyncConnectionJob,
    item_batch_size: i64,
) -> Result<(), AppError> {
    let Some(initial_connection) = deps
        .connection_repo
        .find_by_id(job.user_id, job.connection_id)
        .await?
    else {
        return Err(AppError::Domain(DomainError::NotFound {
            entity: "IntegrationConnection",
            id: job.connection_id.to_string(),
        }));
    };

    if !has_cached_managed_target(&initial_connection.config) {
        let access_token = load_notion_access_token(deps, job.user_id).await?;
        let client = NotionClient::new(
            access_token,
            deps.notion_api_base.clone(),
            deps.rate_limiters.for_connection(job.connection_id),
        );
        match resolve_managed_target(&initial_connection, &client, deps).await {
            Ok(_) => {}
            Err(e) => {
                deps.connection_repo
                    .set_last_error(job.connection_id, job.user_id, Some(e.to_string()))
                    .await?;
                return Err(map_notion_error(e));
            }
        }
    }

    let mut after: Option<NotionExportCursor> = None;

    loop {
        let Some(connection) = deps
            .connection_repo
            .find_by_id(job.user_id, job.connection_id)
            .await?
        else {
            return Err(AppError::Domain(DomainError::NotFound {
                entity: "IntegrationConnection",
                id: job.connection_id.to_string(),
            }));
        };
        let settings = notion_settings_from_config(&connection.config);
        let candidates = deps
            .connection_repo
            .list_notion_export_candidates(
                job.user_id,
                job.connection_id,
                settings.selection_enabled,
                after,
                item_batch_size,
            )
            .await?;

        if candidates.is_empty() {
            break;
        }

        let batch_len = candidates.len();
        let next_cursor = candidates.last().map(|c| NotionExportCursor {
            saved_at: c.saved_at,
            library_entry_id: c.library_entry_id,
        });

        for candidate in &candidates {
            let payload = serde_json::to_value(NotionExportDocumentJob {
                connection_id: job.connection_id,
                user_id: job.user_id,
                library_entry_id: candidate.library_entry_id,
                document_id: candidate.document_id,
            })
            .map_err(|e| AppError::ExternalService {
                service: "notion".into(),
                message: format!("failed to serialize export_document payload: {e}"),
            })?;
            let dedupe_key = format!(
                "export:{}:{}",
                job.connection_id.into_uuid(),
                candidate.library_entry_id.into_uuid()
            );
            deps.outbox_repo
                .enqueue(
                    "integration.notion.export_document",
                    payload,
                    Some(dedupe_key),
                    Utc::now(),
                )
                .await?;
        }

        after = next_cursor;

        if batch_len < item_batch_size as usize {
            break;
        }
    }

    deps.connection_repo
        .set_last_sync_at(job.connection_id, job.user_id, Utc::now())
        .await?;
    Ok(())
}
