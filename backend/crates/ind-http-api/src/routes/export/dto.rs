use serde::Serialize;
use utoipa::ToSchema;

use ind_application::outputs::export::{ObsidianArtifactDownload, ObsidianRunStatus};
use ind_application::ports::{ObsidianAckSubject, ObsidianRunAck, ObsidianRunCreate};

#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct CreateObsidianRunRequest {
    #[serde(default)]
    pub parent_folder_deleted: bool,
    #[serde(default)]
    pub auto: bool,
    #[serde(default)]
    pub force_subject_ids: Vec<String>,
}

impl TryFrom<CreateObsidianRunRequest> for ObsidianRunCreate {
    type Error = String;

    fn try_from(value: CreateObsidianRunRequest) -> Result<Self, Self::Error> {
        let force_subject_ids = value
            .force_subject_ids
            .into_iter()
            .map(|raw| raw.parse().map_err(|_| raw))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|raw| format!("invalid subject id: {raw}"))?;
        Ok(Self {
            parent_folder_deleted: value.parent_folder_deleted,
            auto: value.auto,
            force_subject_ids,
        })
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ObsidianRunStatusResponse {
    pub run_id: String,
    pub task_status: String,
    pub total_documents: i32,
    pub documents_exported: i32,
    pub is_finished: bool,
    pub artifact_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl From<ObsidianRunStatus> for ObsidianRunStatusResponse {
    fn from(value: ObsidianRunStatus) -> Self {
        Self {
            run_id: value.run_id.to_string(),
            task_status: value.task_status,
            total_documents: value.total_documents,
            documents_exported: value.documents_exported,
            is_finished: value.is_finished,
            artifact_ids: value
                .artifact_ids
                .into_iter()
                .map(|id| id.to_string())
                .collect(),
            error: value.error,
        }
    }
}

#[derive(Debug, serde::Deserialize, ToSchema, Default)]
pub struct AckObsidianRunRequest {
    #[serde(default)]
    pub artifact_ids: Vec<String>,
    #[serde(default)]
    pub subjects: Vec<AckObsidianSubjectDto>,
}

#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct AckObsidianSubjectDto {
    pub subject_id: String,
    pub status: String,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub last_content_hash: Option<String>,
    #[serde(default)]
    pub last_full_document_hash: Option<String>,
}

impl TryFrom<AckObsidianRunRequest> for ObsidianRunAck {
    type Error = String;

    fn try_from(value: AckObsidianRunRequest) -> Result<Self, Self::Error> {
        let artifact_ids = value
            .artifact_ids
            .into_iter()
            .map(|raw| uuid::Uuid::parse_str(&raw).map_err(|_| raw))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|raw| format!("invalid artifact id: {raw}"))?;
        let subjects = value
            .subjects
            .into_iter()
            .map(|subject| {
                Ok(ObsidianAckSubject {
                    subject_id: subject
                        .subject_id
                        .parse()
                        .map_err(|_| format!("invalid subject id: {}", subject.subject_id))?,
                    status: subject.status,
                    error: subject.error,
                    last_content_hash: subject.last_content_hash,
                    last_full_document_hash: subject.last_full_document_hash,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(Self {
            artifact_ids,
            subjects,
        })
    }
}

#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct RefreshObsidianSubjectsRequest {
    pub subject_ids: Vec<String>,
    #[serde(default = "default_refresh_reason")]
    pub reason: String,
}

fn default_refresh_reason() -> String {
    "manual".to_string()
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RefreshObsidianSubjectsResponse {
    pub queued: u32,
}

#[derive(Debug, serde::Deserialize, ToSchema, validator::Validate)]
pub struct RecordObsidianRenameRequest {
    #[validate(length(min = 1, max = 64))]
    pub subject_id: String,
    #[validate(length(min = 1, max = 1024))]
    pub new_path: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RecordObsidianRenameResponse {
    pub subject_id: String,
    pub new_path: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ObsidianArtifactDownloadMeta {
    pub artifact_id: String,
    pub content_type: String,
    pub byte_size: usize,
}

impl From<&ObsidianArtifactDownload> for ObsidianArtifactDownloadMeta {
    fn from(value: &ObsidianArtifactDownload) -> Self {
        Self {
            artifact_id: value.artifact_id.to_string(),
            content_type: value.content_type.clone(),
            byte_size: value.bytes.len(),
        }
    }
}
