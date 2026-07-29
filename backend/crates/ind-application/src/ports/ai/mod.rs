use std::pin::Pin;

use futures::{Stream, future::BoxFuture};
use ind_domain::{
    AiPromptAction, AiPromptPresetId, CollectionId, DocumentId, FeedDeliveryId, MilaSessionId,
    MilaSessionType, PlaybackKind, PlaybackState, TtsElementTiming, TtsSessionId, TtsVoicePersona,
    TtsVoicePersonaId, UserId,
};

use crate::AppError;
use crate::outputs::mila::{
    MilaConfigOutput, MilaConversationOutput, MilaPromptPresetGroupOutput, MilaPromptPresetOutput,
    MilaSessionOutput, MilaSessionWithPreviewOutput, MilaStatusOutput, MilaStreamDeltaOutput,
};
use crate::services::tts::persona::CreatePersonaInput;
use crate::services::tts::{StartSessionInput, TtsResolvedChunk, TtsSessionManifest};
use crate::storage::{ByteRange, RangedObjectData};

mod chat;
mod config;
mod prompt_preset;
mod session;

pub use chat::*;
pub use config::*;
pub use prompt_preset::*;
pub use session::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateMilaConfigRequest {
    pub chat_api_base: String,
    pub chat_api_key: Option<String>,
    pub clear_chat_api_key: bool,
    pub chat_model: String,
    pub embedding_api_base: String,
    pub embedding_api_key: Option<String>,
    pub clear_embedding_api_key: bool,
    pub embedding_model: String,
    pub embedding_dim: i32,
    pub model_context_window: i32,
    pub chat_context_pct: i32,
    pub top_k: i32,
    pub cross_item_top_k: i32,
    pub cross_item_max_per_item: i32,
    pub enabled: bool,
    pub byo_enabled: Option<bool>,
    pub supports_structured_output: Option<bool>,
    pub supports_reasoning_effort: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestMilaConfigRequest {
    pub chat_api_base: String,
    pub chat_api_key: Option<String>,
    pub chat_model: String,
    pub embedding_api_base: String,
    pub embedding_api_key: Option<String>,
    pub embedding_model: String,
    pub embedding_dim: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MilaProviderTestResult {
    pub success: bool,
    pub embedding_dim: Option<i32>,
    pub chat_model_ok: bool,
    pub embedding_model_ok: bool,
    pub chat_error: Option<String>,
    pub embedding_error: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateMilaPromptPresetRequest {
    pub action: AiPromptAction,
    pub name: String,
    pub system_prompt: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateMilaPromptPresetRequest {
    pub name: Option<String>,
    pub system_prompt: Option<String>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateMilaSessionRequest {
    pub session_type: MilaSessionType,
    pub document_id: Option<DocumentId>,
    /// TASK-234: single-document chat from an (un)prepared feed delivery. Resolved by the
    /// MilaSessionService to a materialize-or-find chat identity (AC#2).
    pub delivery_id: Option<FeedDeliveryId>,
    pub collection_id: Option<CollectionId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MilaStreamRequest {
    pub session_id: MilaSessionId,
    pub question: String,
    pub highlight_text: Option<String>,
    pub highlight_offset: Option<usize>,
}

pub type MilaStreamOutputStream =
    Pin<Box<dyn Stream<Item = Result<MilaStreamDeltaOutput, AppError>> + Send + 'static>>;

pub trait TtsOperations: Send + Sync {
    fn list_personas(
        &self,
        user_id: UserId,
    ) -> BoxFuture<'_, Result<Vec<TtsVoicePersona>, AppError>>;

    fn create_persona(
        &self,
        user_id: UserId,
        input: CreatePersonaInput,
    ) -> BoxFuture<'_, Result<TtsVoicePersona, AppError>>;

    fn get_persona(
        &self,
        user_id: UserId,
        persona_id: TtsVoicePersonaId,
    ) -> BoxFuture<'_, Result<TtsVoicePersona, AppError>>;

    fn start_session(
        &self,
        user_id: UserId,
        input: StartSessionInput,
    ) -> BoxFuture<'_, Result<TtsSessionManifest, AppError>>;

    fn resolve_session_chunk(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        session_id: TtsSessionId,
        chunk_id: String,
    ) -> BoxFuture<'_, Result<Option<TtsResolvedChunk>, AppError>>;

    fn resolve_element_timestamp(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        session_id: TtsSessionId,
        chunk_id: String,
        element_index: i32,
    ) -> BoxFuture<'_, Result<Option<TtsElementTiming>, AppError>>;

    fn get_session_chunk_audio(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        session_id: TtsSessionId,
        chunk_id: String,
        range: Option<ByteRange>,
    ) -> BoxFuture<'_, Result<RangedObjectData, AppError>>;

    fn upsert_playback_state(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        input: UpsertPlaybackStateInput,
    ) -> BoxFuture<'_, Result<PlaybackState, AppError>>;

    fn get_playback_state(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        kind: PlaybackKind,
    ) -> BoxFuture<'_, Result<Option<PlaybackState>, AppError>>;
}

#[derive(Debug, Clone)]
pub struct UpsertPlaybackStateInput {
    pub playback_kind: PlaybackKind,
    pub position_seconds: f64,
    pub playback_speed: f64,
    pub element_index: Option<i32>,
    pub tts_chunk_id: Option<String>,
    pub tts_voice_persona_id: Option<TtsVoicePersonaId>,
    pub is_playing: bool,
}
