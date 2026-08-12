pub(crate) mod chat;
pub(crate) mod config;
pub(crate) mod dto;
pub(crate) mod presets;
pub(crate) mod retry;
pub(crate) mod sessions;

use axum::Router;
use axum::routing::{delete, get, patch, post};
use ind_application::ports::{
    MilaActionRetryPort, MilaChatPort, MilaConfigPort, MilaPromptPresetPort, MilaSessionPort,
};

use crate::error::ApiError;
use crate::state::AppState;

pub use chat::stream_chat;
pub use config::{get_config, get_status, reindex_config, test_config, upsert_config};
pub use presets::{
    create_prompt_preset, delete_prompt_preset, list_prompt_presets, update_prompt_preset,
};
pub use retry::retry_mila_document_action;
pub use sessions::{create_session, delete_session, get_session_messages, list_sessions};

pub(crate) use dto::{
    CreateMilaPromptPresetBody, CreateMilaSessionBody, ListSessionsParams, MilaConfigResponse,
    MilaConversationResponse, MilaDocumentProvenanceResponse, MilaMessageResponse,
    MilaPromptPresetGroupResponse, MilaPromptPresetResponse, MilaPromptPresetsResponse,
    MilaSessionListResponse, MilaSessionPreviewResponse, MilaSessionResponse, MilaStatusResponse,
    MilaStreamDeltaResponse, MilaStreamErrorResponse, MilaStreamParams, RetryMilaActionResponse,
    TestMilaConfigBody, TestMilaConfigResponse, UpdateMilaPromptPresetBody, UpsertMilaConfigBody,
};

fn require_mila_config_ops(state: &AppState) -> Result<&dyn MilaConfigPort, ApiError> {
    state
        .mila_config_ops
        .as_deref()
        .ok_or(ApiError::ServiceUnavailable {
            message: "mila service not configured".into(),
        })
}

fn require_mila_prompt_preset_ops(state: &AppState) -> Result<&dyn MilaPromptPresetPort, ApiError> {
    state
        .mila_prompt_preset_ops
        .as_deref()
        .ok_or(ApiError::ServiceUnavailable {
            message: "mila service not configured".into(),
        })
}

fn require_mila_session_ops(state: &AppState) -> Result<&dyn MilaSessionPort, ApiError> {
    state
        .mila_session_ops
        .as_deref()
        .ok_or(ApiError::ServiceUnavailable {
            message: "mila service not configured".into(),
        })
}

fn require_mila_chat_ops(state: &AppState) -> Result<&dyn MilaChatPort, ApiError> {
    state
        .mila_chat_ops
        .as_deref()
        .ok_or(ApiError::ServiceUnavailable {
            message: "mila service not configured".into(),
        })
}

fn require_mila_action_retry_ops(state: &AppState) -> Result<&dyn MilaActionRetryPort, ApiError> {
    state
        .mila_action_retry_ops
        .as_deref()
        .ok_or(ApiError::ServiceUnavailable {
            message: "mila service not configured".into(),
        })
}

fn parse_session_id(raw: &str) -> Result<ind_domain::MilaSessionId, ApiError> {
    raw.parse().map_err(|_| ApiError::NotFound {
        entity: "MilaSession",
        id: raw.to_string(),
    })
}

fn parse_prompt_preset_id(raw: &str) -> Result<ind_domain::AiPromptPresetId, ApiError> {
    raw.parse().map_err(|_| ApiError::NotFound {
        entity: "AiPromptPreset",
        id: raw.to_string(),
    })
}

fn validation_error(errors: Vec<crate::error::FieldError>) -> ApiError {
    ApiError::ValidationError { errors }
}

pub fn mila_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/mila/status", get(get_status))
        .route("/api/v1/mila/config", get(get_config).post(upsert_config))
        .route("/api/v1/mila/config/reindex", post(reindex_config))
        .route(
            "/api/v1/mila/presets",
            get(list_prompt_presets).post(create_prompt_preset),
        )
        .route(
            "/api/v1/mila/presets/{preset_id}",
            patch(update_prompt_preset).delete(delete_prompt_preset),
        )
        .route("/api/v1/mila/config/test", post(test_config))
        .route(
            "/api/v1/mila/sessions",
            get(list_sessions).post(create_session),
        )
        .route(
            "/api/v1/mila/sessions/{session_id}/messages",
            get(get_session_messages),
        )
        .route("/api/v1/mila/sessions/{session_id}", delete(delete_session))
        .route("/api/v1/mila/stream", get(stream_chat))
        .route(
            "/api/v1/mila/documents/{document_id}/actions/{action}/retry",
            post(retry_mila_document_action),
        )
}
