use std::time::Duration;

use chrono::{DateTime, Utc};
use ind_domain::{DocumentId, ReprocessDocumentJob, UserId};

use crate::error::AppError;
use crate::repos::document_upload::StagedDocumentAsset;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentReprocessAdmission {
    pub queued: bool,
    pub retry_after_seconds: Option<u64>,
}

pub struct CompleteUploadReprocess {
    pub document_id: DocumentId,
    pub user_id: UserId,
    pub assets: Vec<StagedDocumentAsset>,
    pub word_count: Option<i32>,
    pub reading_time_minutes: Option<i32>,
}

#[async_trait::async_trait]
pub trait DocumentReprocessRepository: Send + Sync {
    async fn admit(
        &self,
        job: ReprocessDocumentJob,
        requested_at: DateTime<Utc>,
        cooldown: Duration,
    ) -> Result<DocumentReprocessAdmission, AppError>;

    async fn complete_upload(&self, request: CompleteUploadReprocess) -> Result<(), AppError>;
}
