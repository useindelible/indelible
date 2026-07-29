use std::sync::Arc;

use futures::future::BoxFuture;
use ind_application::AppError;
use ind_application::outputs::export::{
    ObsidianArtifactDownload, ObsidianRefreshResult, ObsidianRunStatus,
};
use ind_application::ports::{
    ExportOperations, ObsidianRefreshRequest, ObsidianRunAck, ObsidianRunCreate,
};
use ind_application::repos::export_cursor::ExportCursorRepository;
use ind_application::repos::integration_connection::IntegrationConnectionRepository;
use ind_application::repos::obsidian_export::ObsidianExportRepository;
use ind_application::repos::outbox::JobOutboxRepository;
use ind_domain::{LibraryEntryId, UserId};

pub struct ExportOperationsService {
    workflow: crate::obsidian_workflow::ObsidianRunWorkflow,
}

impl ExportOperationsService {
    pub fn new(
        connection_repo: Arc<dyn IntegrationConnectionRepository>,
        outbox_repo: Arc<dyn JobOutboxRepository>,
        export_cursor_repo: Arc<dyn ExportCursorRepository>,
        obsidian_export_repo: Arc<dyn ObsidianExportRepository>,
    ) -> Self {
        Self {
            workflow: crate::obsidian_workflow::ObsidianRunWorkflow::new(
                connection_repo,
                outbox_repo,
                export_cursor_repo,
                obsidian_export_repo,
            ),
        }
    }
}

impl ExportOperations for ExportOperationsService {
    fn create_obsidian_run(
        &self,
        user_id: UserId,
        input: ObsidianRunCreate,
    ) -> BoxFuture<'_, Result<ObsidianRunStatus, AppError>> {
        Box::pin(self.workflow.create_run(user_id, input))
    }

    fn get_obsidian_run(
        &self,
        user_id: UserId,
        run_id: uuid::Uuid,
    ) -> BoxFuture<'_, Result<ObsidianRunStatus, AppError>> {
        Box::pin(self.workflow.get_run(user_id, run_id))
    }

    fn get_obsidian_artifact(
        &self,
        user_id: UserId,
        artifact_id: uuid::Uuid,
    ) -> BoxFuture<'_, Result<ObsidianArtifactDownload, AppError>> {
        Box::pin(self.workflow.get_artifact(user_id, artifact_id))
    }

    fn ack_obsidian_run(
        &self,
        user_id: UserId,
        run_id: uuid::Uuid,
        input: ObsidianRunAck,
    ) -> BoxFuture<'_, Result<ObsidianRunStatus, AppError>> {
        Box::pin(self.workflow.ack_run(user_id, run_id, input))
    }

    fn refresh_obsidian_subjects(
        &self,
        user_id: UserId,
        input: ObsidianRefreshRequest,
    ) -> BoxFuture<'_, Result<ObsidianRefreshResult, AppError>> {
        Box::pin(async move {
            self.workflow
                .refresh_subjects(user_id, &input.subject_ids, &input.reason)
                .await
        })
    }

    fn record_obsidian_path_rename(
        &self,
        user_id: UserId,
        subject_id: LibraryEntryId,
        new_path: String,
    ) -> BoxFuture<'_, Result<(), AppError>> {
        Box::pin(
            self.workflow
                .record_path_rename(user_id, subject_id, new_path),
        )
    }
}
