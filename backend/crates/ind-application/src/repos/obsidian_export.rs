use crate::AppError;
use ind_domain::{IntegrationConnectionId, LibraryEntryId, UserId};

#[derive(Debug, Clone)]
pub struct CreateObsidianRunInput {
    pub run_id: uuid::Uuid,
    pub connection_id: IntegrationConnectionId,
    pub user_id: UserId,
    pub requested_by_user: bool,
    pub auto: bool,
    pub parent_folder_deleted: bool,
    pub force_library_entry_ids: Vec<LibraryEntryId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObsidianRunStatusRecord {
    pub run_id: uuid::Uuid,
    pub status: String,
    pub total_documents: i32,
    pub documents_exported: i32,
    pub artifact_ids: Vec<uuid::Uuid>,
    pub error: Option<String>,
}

impl ObsidianRunStatusRecord {
    pub fn is_finished(&self) -> bool {
        matches!(
            self.status.as_str(),
            "artifact_ready" | "success" | "partial_success" | "failed"
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObsidianArtifactDownloadRecord {
    pub artifact_id: uuid::Uuid,
    pub content_type: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ObsidianAckSubjectRecord {
    pub library_entry_id: LibraryEntryId,
    pub status: String,
    pub error: Option<String>,
    pub last_content_hash: Option<String>,
    pub last_full_document_hash: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct AckObsidianRunInput {
    pub artifact_ids: Vec<uuid::Uuid>,
    pub subjects: Vec<ObsidianAckSubjectRecord>,
}

#[async_trait::async_trait]
pub trait ObsidianExportRepository: Send + Sync {
    async fn create_run(&self, input: CreateObsidianRunInput) -> Result<(), AppError>;

    async fn run_status(
        &self,
        user_id: UserId,
        run_id: uuid::Uuid,
    ) -> Result<Option<ObsidianRunStatusRecord>, AppError>;

    async fn artifact_download(
        &self,
        user_id: UserId,
        artifact_id: uuid::Uuid,
    ) -> Result<Option<ObsidianArtifactDownloadRecord>, AppError>;

    async fn ack_run(
        &self,
        user_id: UserId,
        run_id: uuid::Uuid,
        input: AckObsidianRunInput,
    ) -> Result<ObsidianRunStatusRecord, AppError>;

    async fn queue_refresh_subjects(
        &self,
        user_id: UserId,
        connection_id: IntegrationConnectionId,
        library_entry_ids: &[LibraryEntryId],
        reason: &str,
    ) -> Result<u32, AppError>;
}
