use std::sync::Arc;

use ind_application::AppError;
use ind_application::ports::IntegrationSyncEnqueued;
use ind_application::repos::integration_connection::IntegrationConnectionRepository;
use ind_application::repos::outbox::JobOutboxRepository;
use ind_domain::{
    DomainError, IntegrationConnectionId, IntegrationProvider, NotionSyncConnectionJob,
    ObsidianSyncConnectionJob, UserId, job_types,
};

pub struct IntegrationSyncService {
    connection_repo: Arc<dyn IntegrationConnectionRepository>,
    outbox_repo: Arc<dyn JobOutboxRepository>,
}

impl IntegrationSyncService {
    pub fn new(
        connection_repo: Arc<dyn IntegrationConnectionRepository>,
        outbox_repo: Arc<dyn JobOutboxRepository>,
    ) -> Self {
        Self {
            connection_repo,
            outbox_repo,
        }
    }

    pub async fn sync_now(
        &self,
        user_id: UserId,
        connection_id: IntegrationConnectionId,
    ) -> Result<IntegrationSyncEnqueued, AppError> {
        let connection = self
            .connection_repo
            .find_by_id(user_id, connection_id)
            .await?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "IntegrationConnection",
                    id: connection_id.to_string(),
                })
            })?;

        #[expect(
            clippy::expect_used,
            reason = "sync job payloads are plain owned structs; serde_json::to_value is infallible for them"
        )]
        let (job_type, payload) = match connection.provider {
            IntegrationProvider::Obsidian => (
                job_types::INTEGRATION_OBSIDIAN_SYNC_CONNECTION,
                serde_json::to_value(ObsidianSyncConnectionJob {
                    connection_id: connection.id,
                    user_id,
                    requested_by_user: true,
                    run_id: None,
                })
                .expect("ObsidianSyncConnectionJob is serializable"),
            ),
            IntegrationProvider::Notion => (
                job_types::INTEGRATION_NOTION_SYNC_CONNECTION,
                serde_json::to_value(NotionSyncConnectionJob {
                    connection_id: connection.id,
                    user_id,
                    requested_by_user: true,
                })
                .expect("NotionSyncConnectionJob is serializable"),
            ),
            provider => {
                return Err(AppError::Domain(DomainError::Validation {
                    field: "provider".to_string(),
                    message: format!("provider {provider:?} does not support manual sync"),
                }));
            }
        };

        let dedupe_key = format!("manual_sync:{}", connection.id.into_uuid());
        let outbox = self
            .outbox_repo
            .enqueue(job_type, payload, Some(dedupe_key), chrono::Utc::now())
            .await?;

        Ok(IntegrationSyncEnqueued {
            job_id: outbox.id.to_string(),
        })
    }
}
