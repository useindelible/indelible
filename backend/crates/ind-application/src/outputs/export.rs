#[derive(Debug, Clone)]
pub struct ObsidianRunStatus {
    pub run_id: uuid::Uuid,
    pub task_status: String,
    pub total_documents: i32,
    pub documents_exported: i32,
    pub is_finished: bool,
    pub artifact_ids: Vec<uuid::Uuid>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ObsidianArtifactDownload {
    pub artifact_id: uuid::Uuid,
    pub content_type: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ObsidianRefreshResult {
    pub queued: u32,
}

#[derive(Debug, Clone)]
pub struct ObsidianExportPreview {
    pub file_path: String,
    pub full_content: String,
    pub append_only_content: Option<String>,
    pub full_document_text_path: Option<String>,
    pub full_document_text: Option<String>,
}
