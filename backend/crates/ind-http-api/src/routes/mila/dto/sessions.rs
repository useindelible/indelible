use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::FieldError;
use crate::extract::Validate;
use ind_application::outputs::mila::{
    MilaConversationOutput, MilaMessageOutput, MilaSessionOutput, MilaSessionWithPreviewOutput,
};
use ind_application::ports::{CreateMilaSessionRequest, MilaStreamRequest};
use ind_domain::{MilaSessionId, MilaSessionType};

use super::{
    VALID_MILA_SESSION_TYPES, format_message_role, format_session_type, parse_mila_session_type,
    parse_optional_id, validate_required,
};

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateMilaSessionBody {
    pub session_type: String,
    #[serde(default)]
    pub document_id: Option<String>,
    #[serde(default)]
    pub delivery_id: Option<String>,
    #[serde(default)]
    pub collection_id: Option<String>,
}

impl Validate for CreateMilaSessionBody {
    fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut errors = Vec::new();
        let parsed_session_type = parse_mila_session_type(&self.session_type);

        if parsed_session_type.is_none() {
            errors.push(FieldError {
                field: "session_type".into(),
                message: format!("must be one of: {}", VALID_MILA_SESSION_TYPES.join(", ")),
            });
        }

        match parsed_session_type {
            Some(MilaSessionType::SingleDocument) => {
                let has_document = self
                    .document_id
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty());
                let has_delivery = self
                    .delivery_id
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty());
                if has_document == has_delivery {
                    errors.push(FieldError {
                        field: "document_id".into(),
                        message: "single_document sessions require exactly one of document_id or \
                                  delivery_id"
                            .into(),
                    });
                }
                for (field, present) in [("collection_id", self.collection_id.as_deref())] {
                    if present.is_some_and(|value| !value.trim().is_empty()) {
                        errors.push(FieldError {
                            field: field.into(),
                            message: "must not be provided for single_document sessions".into(),
                        });
                    }
                }
            }
            Some(MilaSessionType::CrossItem) => {
                if self
                    .collection_id
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
                {
                    errors.push(FieldError {
                        field: "collection_id".into(),
                        message: "must not be provided for cross_item sessions".into(),
                    });
                }
            }
            Some(MilaSessionType::Collection) => {
                if self
                    .collection_id
                    .as_deref()
                    .is_none_or(|value| value.trim().is_empty())
                {
                    errors.push(FieldError {
                        field: "collection_id".into(),
                        message: "is required for collection sessions".into(),
                    });
                }
            }
            None => {}
        }

        // Document/delivery identity is single-document-only; reject it on every other session
        // type so the request cannot carry a contradictory scope (TASK-234).
        if !matches!(parsed_session_type, Some(MilaSessionType::SingleDocument)) {
            for (field, present) in [
                ("document_id", self.document_id.as_deref()),
                ("delivery_id", self.delivery_id.as_deref()),
            ] {
                if present.is_some_and(|value| !value.trim().is_empty()) {
                    errors.push(FieldError {
                        field: field.into(),
                        message: "must not be provided unless session_type is single_document"
                            .into(),
                    });
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl CreateMilaSessionBody {
    pub fn into_state_request(self) -> Result<CreateMilaSessionRequest, Vec<FieldError>> {
        let session_type = parse_mila_session_type(&self.session_type).ok_or_else(|| {
            vec![FieldError {
                field: "session_type".into(),
                message: format!("must be one of: {}", VALID_MILA_SESSION_TYPES.join(", ")),
            }]
        })?;

        let document_id = parse_optional_id("document_id", self.document_id)?;
        let delivery_id = parse_optional_id("delivery_id", self.delivery_id)?;
        let collection_id = parse_optional_id("collection_id", self.collection_id)?;

        Ok(CreateMilaSessionRequest {
            session_type,
            document_id,
            delivery_id,
            collection_id,
        })
    }
}

/// Composed document provenance (TASK-234 AC#6). Concretely typed (no `serde_json::Value`) so
/// generated clients get a real type. Distinguishes Library-backed documents from prepared
/// unsaved documents with durable capability rows.
#[derive(Debug, Serialize, ToSchema)]
pub struct MilaDocumentProvenanceResponse {
    pub is_saved: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub library_source: Option<String>,
    pub origins: Vec<String>,
    pub has_highlights: bool,
    pub has_note: bool,
    pub has_mila_session: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MilaSessionResponse {
    pub id: String,
    pub session_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<MilaDocumentProvenanceResponse>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub last_active: chrono::DateTime<chrono::Utc>,
}

fn project_provenance(
    provenance: ind_domain::DocumentProvenance,
) -> MilaDocumentProvenanceResponse {
    MilaDocumentProvenanceResponse {
        is_saved: provenance.is_saved,
        library_source: provenance.library_source.map(|s| s.as_str().to_string()),
        origins: provenance
            .origins
            .into_iter()
            .map(|o| o.as_str().to_string())
            .collect(),
        has_highlights: provenance.has_highlights,
        has_note: provenance.has_note,
        has_mila_session: provenance.has_mila_session,
    }
}

pub fn project_mila_session(output: MilaSessionOutput) -> MilaSessionResponse {
    MilaSessionResponse {
        id: output.id.to_string(),
        session_type: format_session_type(output.session_type).into(),
        document_id: output.document_id.map(|id| id.to_string()),
        collection_id: output.collection_id.map(|id| id.to_string()),
        provenance: output.provenance.map(project_provenance),
        created_at: output.created_at,
        last_active: output.last_active,
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MilaSourceRef {
    pub source_label: String,
    pub document_id: String,
    pub item_title: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MilaMessageResponse {
    pub id: String,
    pub role: String,
    pub content: String,
    pub source_refs: Vec<MilaSourceRef>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub fn project_mila_message(output: MilaMessageOutput) -> MilaMessageResponse {
    MilaMessageResponse {
        id: output.id.to_string(),
        role: format_message_role(output.role).into(),
        content: output.content,
        source_refs: output
            .source_refs
            .into_iter()
            .map(|r| MilaSourceRef {
                source_label: r.source_label,
                document_id: r.document_id.to_string(),
                item_title: r.item_title,
            })
            .collect(),
        created_at: output.created_at,
    }
}

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct ListSessionsParams {
    #[serde(default = "default_sessions_limit")]
    pub limit: i64,
}

fn default_sessions_limit() -> i64 {
    50
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MilaSessionPreviewResponse {
    pub id: String,
    pub session_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_id: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub last_active: chrono::DateTime<chrono::Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_role: Option<String>,
}

pub fn project_mila_session_preview(
    output: MilaSessionWithPreviewOutput,
) -> MilaSessionPreviewResponse {
    MilaSessionPreviewResponse {
        id: output.id.to_string(),
        session_type: format_session_type(output.session_type).into(),
        document_id: output.document_id.map(|id| id.to_string()),
        collection_id: output.collection_id.map(|id| id.to_string()),
        created_at: output.created_at,
        last_active: output.last_active,
        preview_content: output.preview_content,
        preview_role: output.preview_role,
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MilaSessionListResponse {
    pub sessions: Vec<MilaSessionPreviewResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MilaConversationResponse {
    pub session: MilaSessionResponse,
    pub messages: Vec<MilaMessageResponse>,
}

pub fn project_mila_conversation(output: MilaConversationOutput) -> MilaConversationResponse {
    MilaConversationResponse {
        session: project_mila_session(output.session),
        messages: output
            .messages
            .into_iter()
            .map(project_mila_message)
            .collect(),
    }
}

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct MilaStreamParams {
    pub session_id: String,
    pub question: String,
    #[serde(default)]
    pub highlight_text: Option<String>,
    #[serde(default)]
    pub highlight_offset: Option<usize>,
}

impl Validate for MilaStreamParams {
    fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut errors = Vec::new();

        validate_required("session_id", &self.session_id, &mut errors);
        validate_required("question", &self.question, &mut errors);

        if self.highlight_offset.is_some()
            && self
                .highlight_text
                .as_deref()
                .is_none_or(|value| value.trim().is_empty())
        {
            errors.push(FieldError {
                field: "highlight_text".into(),
                message: "is required when highlight_offset is provided".into(),
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl MilaStreamParams {
    pub fn into_state_request(self) -> Result<MilaStreamRequest, Vec<FieldError>> {
        let session_id = self
            .session_id
            .trim()
            .parse::<MilaSessionId>()
            .map_err(|_| {
                vec![FieldError {
                    field: "session_id".into(),
                    message: "must be a valid Mila session id".into(),
                }]
            })?;

        Ok(MilaStreamRequest {
            session_id,
            question: self.question.trim().to_string(),
            highlight_text: self.highlight_text.and_then(|value| {
                let trimmed = value.trim();
                (!trimmed.is_empty()).then(|| trimmed.to_string())
            }),
            highlight_offset: self.highlight_offset,
        })
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MilaStreamDeltaResponse {
    pub delta: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retrieval_degraded: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MilaStreamErrorResponse {
    pub error: String,
}
