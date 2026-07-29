use chrono::{DateTime, Utc};
use ind_application::ports::TtsResolvedChunk;
use ind_application::services::tts::TtsSessionManifest;
use ind_application::services::tts::session::PlannedChunkState;
use ind_domain::{
    AudioFormat, PlaybackKind, PlaybackState, TtsChunkStatus, TtsElementTiming, TtsGenerationScope,
    TtsProvider, TtsSession, TtsTimingSource, TtsVoicePersona, TtsVoicePersonaId,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::{ApiError, FieldError};
use crate::extract::Validate;

/// Persona response — represents a voice persona owned by the user (or a
/// built-in persona shipped with the product).
#[derive(Debug, Serialize, ToSchema)]
pub struct VoicePersonaResponse {
    pub id: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub provider: String,
    pub status: String,
    pub is_builtin: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_voice_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub design_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub energy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warmth: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formality: Option<String>,
    /// Opaque string->string map of pronunciation overrides (regex ->
    /// replacement is the typical shape). Treated as an opaque bag at the
    /// API layer.
    #[schema(value_type = std::collections::HashMap<String, String>)]
    pub pronunciation_prefs: serde_json::Value,
    pub prompt_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl VoicePersonaResponse {
    pub fn from_domain(persona: TtsVoicePersona) -> Self {
        Self {
            id: persona.id.to_string(),
            display_name: persona.display_name,
            description: persona.description,
            provider: persona.provider.as_str().to_string(),
            status: persona.status.as_str().to_string(),
            is_builtin: persona.is_builtin,
            provider_voice_id: persona.provider_voice_id,
            provider_model: persona.provider_model,
            design_prompt: persona.design_prompt,
            style_prompt: persona.style_prompt,
            pace: persona.pace,
            energy: persona.energy,
            warmth: persona.warmth,
            formality: persona.formality,
            pronunciation_prefs: persona.pronunciation_prefs,
            prompt_hash: persona.prompt_hash,
            created_at: persona.created_at,
            updated_at: persona.updated_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct VoicePersonaListResponse {
    pub personas: Vec<VoicePersonaResponse>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateVoicePersonaBody {
    pub display_name: String,
    #[serde(default)]
    pub description: Option<String>,
    /// Provider string: one of dashscope, elevenlabs, unreal_speech, inworld,
    /// gemini, polly, mock.
    pub provider: String,
    #[serde(default)]
    pub provider_voice_id: Option<String>,
    #[serde(default)]
    pub provider_model: Option<String>,
    #[serde(default)]
    pub design_prompt: Option<String>,
    #[serde(default)]
    pub style_prompt: Option<String>,
    #[serde(default)]
    pub pace: Option<String>,
    #[serde(default)]
    pub energy: Option<String>,
    #[serde(default)]
    pub warmth: Option<String>,
    #[serde(default)]
    pub formality: Option<String>,
    #[serde(default)]
    #[schema(value_type = Option<std::collections::HashMap<String, String>>)]
    pub pronunciation_prefs: Option<serde_json::Value>,
}

impl Validate for CreateVoicePersonaBody {
    fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut errors = Vec::new();
        if self.display_name.trim().is_empty() {
            errors.push(FieldError {
                field: "display_name".into(),
                message: "must not be empty".into(),
            });
        }
        if TtsProvider::parse(&self.provider).is_none() {
            errors.push(FieldError {
                field: "provider".into(),
                message: "unknown provider".into(),
            });
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct StartSessionBody {
    #[serde(default)]
    pub voice_persona_id: Option<String>,
    #[serde(default = "default_speed")]
    pub speed: f64,
    #[serde(default = "default_audio_format")]
    pub audio_format: String,
    #[serde(default = "default_sample_rate")]
    pub sample_rate: i32,
    #[serde(default = "default_generation_scope")]
    pub generation_scope: String,
    #[serde(default = "default_version")]
    pub pronunciation_version: i32,
    #[serde(default = "default_chunking_version")]
    pub chunking_version: i32,
    #[serde(default)]
    pub start_element_index: Option<i32>,
}

fn default_speed() -> f64 {
    1.0
}
fn default_audio_format() -> String {
    "mp3".into()
}
fn default_sample_rate() -> i32 {
    24000
}
fn default_generation_scope() -> String {
    "single_chunk".into()
}
fn default_version() -> i32 {
    1
}
fn default_chunking_version() -> i32 {
    2
}

impl Validate for StartSessionBody {
    fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut errors = Vec::new();
        if AudioFormat::parse(&self.audio_format).is_none() {
            errors.push(FieldError {
                field: "audio_format".into(),
                message: "unknown audio format".into(),
            });
        }
        if TtsGenerationScope::parse(&self.generation_scope).is_none() {
            errors.push(FieldError {
                field: "generation_scope".into(),
                message: "unknown generation scope".into(),
            });
        }
        if self.speed <= 0.0 {
            errors.push(FieldError {
                field: "speed".into(),
                message: "must be greater than 0".into(),
            });
        }
        if self.sample_rate <= 0 {
            errors.push(FieldError {
                field: "sample_rate".into(),
                message: "must be greater than 0".into(),
            });
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SessionManifestResponse {
    pub session: SessionResponse,
    pub persona: VoicePersonaResponse,
    pub document_title: String,
    pub start: SessionStartResponse,
    pub chunks: Vec<PlannedChunkResponse>,
}

impl SessionManifestResponse {
    pub fn from_manifest(manifest: TtsSessionManifest) -> Self {
        let document_id = manifest.session.document_id;
        let session_id = manifest.session.id;
        Self {
            session: SessionResponse::from_domain(manifest.session),
            persona: VoicePersonaResponse::from_domain(manifest.persona),
            document_title: manifest.document_title,
            start: SessionStartResponse {
                chunk_id: manifest.start.chunk_id,
                chunk_record_id: manifest.start.chunk_record_id.to_string(),
                element_index: manifest.start.element_index,
                start_timestamp: manifest.start.start_timestamp,
            },
            chunks: manifest
                .chunks
                .into_iter()
                .map(|chunk| PlannedChunkResponse::from_planned(chunk, document_id, session_id))
                .collect(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SessionStartResponse {
    pub chunk_id: String,
    pub chunk_record_id: String,
    pub element_index: i32,
    pub start_timestamp: f64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SessionResponse {
    pub id: String,
    pub document_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_persona_id: Option<String>,
    pub speed: f64,
    pub audio_format: String,
    pub generation_scope: String,
    pub created_at: DateTime<Utc>,
}

impl SessionResponse {
    pub fn from_domain(session: TtsSession) -> Self {
        Self {
            id: session.id.to_string(),
            document_id: session.document_id.to_string(),
            voice_persona_id: session.voice_persona_id.map(|v| v.to_string()),
            speed: session.speed,
            audio_format: session.audio_format.as_str().to_string(),
            generation_scope: session.generation_scope.as_str().to_string(),
            created_at: session.created_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PlannedChunkResponse {
    pub chunk_id: String,
    pub position: i32,
    pub start_element_index: i32,
    pub end_element_index: i32,
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk_record_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_hit: Option<bool>,
    pub timing_source: TtsTimingSourceDto,
    pub timings: Vec<PlannedChunkTimingResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TtsTimingSourceDto {
    ProviderTranscript,
    Heuristic,
}

impl From<TtsTimingSource> for TtsTimingSourceDto {
    fn from(source: TtsTimingSource) -> Self {
        match source {
            TtsTimingSource::ProviderTranscript => Self::ProviderTranscript,
            TtsTimingSource::Heuristic => Self::Heuristic,
        }
    }
}

impl PlannedChunkResponse {
    fn from_planned(
        chunk: ind_application::services::tts::session::PlannedChunk,
        document_id: ind_domain::DocumentId,
        session_id: ind_domain::TtsSessionId,
    ) -> Self {
        match chunk.state {
            PlannedChunkState::Ready {
                chunk_record_id,
                duration_seconds,
                cache_hit,
                timings,
                timing_source,
                audio_format,
            } => Self {
                audio_url: Some(audio_url(
                    document_id,
                    session_id,
                    &chunk.chunk_id,
                    audio_format,
                )),
                chunk_id: chunk.chunk_id,
                position: chunk.position,
                start_element_index: chunk.start_element_index,
                end_element_index: chunk.end_element_index,
                state: "ready".into(),
                chunk_record_id: Some(chunk_record_id.to_string()),
                duration_seconds,
                cache_hit: Some(cache_hit),
                timing_source: timing_source.into(),
                timings: timings
                    .into_iter()
                    .map(PlannedChunkTimingResponse::from_domain)
                    .collect(),
            },
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PlannedChunkTimingResponse {
    pub element_index: i32,
    pub start_timestamp: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_timestamp: Option<f64>,
}

impl PlannedChunkTimingResponse {
    fn from_domain(timing: TtsElementTiming) -> Self {
        Self {
            element_index: timing.element_index,
            start_timestamp: timing.start_timestamp,
            end_timestamp: timing.end_timestamp,
        }
    }
}

/// Response for resolving a single chunk in a session. When the chunk has
/// audio ready, `audio_url` is populated with the document/session-scoped API
/// proxy path.
#[derive(Debug, Serialize, ToSchema)]
pub struct ResolveChunkResponse {
    pub chunk_id: String,
    pub status: String,
    pub position: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk_record_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_url: Option<String>,
}

impl ResolveChunkResponse {
    pub fn from_resolved(
        resolved: TtsResolvedChunk,
        document_id: ind_domain::DocumentId,
        session_id: ind_domain::TtsSessionId,
    ) -> Self {
        let ready = resolved.chunk.status == TtsChunkStatus::Ready;
        Self {
            chunk_id: resolved.session_chunk.chunk_id.clone(),
            status: resolved.chunk.status.as_str().to_string(),
            position: resolved.session_chunk.position,
            chunk_record_id: ready.then(|| resolved.chunk.id.to_string()),
            duration_seconds: ready.then_some(resolved.chunk.duration_seconds).flatten(),
            audio_url: ready.then(|| {
                audio_url(
                    document_id,
                    session_id,
                    &resolved.session_chunk.chunk_id,
                    resolved.chunk.audio_format,
                )
            }),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ElementTimestampResponse {
    pub chunk_record_id: String,
    pub element_index: i32,
    pub start_timestamp: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_timestamp: Option<f64>,
}

impl ElementTimestampResponse {
    pub fn from_domain(timing: TtsElementTiming) -> Self {
        Self {
            chunk_record_id: timing.chunk_record_id.to_string(),
            element_index: timing.element_index,
            start_timestamp: timing.start_timestamp,
            end_timestamp: timing.end_timestamp,
        }
    }
}

#[derive(Debug, Deserialize, validator::Validate, ToSchema)]
pub struct UpsertPlaybackStateBody {
    pub playback_kind: String,
    #[validate(range(min = 0.0))]
    pub position_seconds: f64,
    #[validate(range(min = 0.5, max = 3.0))]
    pub playback_speed: f64,
    #[validate(range(min = 0))]
    pub element_index: Option<i32>,
    pub tts_chunk_id: Option<String>,
    #[schema(value_type = Option<String>)]
    pub tts_voice_persona_id: Option<String>,
    pub is_playing: bool,
}

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct GetPlaybackStateParams {
    pub kind: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackKindDto {
    Tts,
    Audio,
    Video,
}

impl From<PlaybackKind> for PlaybackKindDto {
    fn from(kind: PlaybackKind) -> Self {
        match kind {
            PlaybackKind::Tts => Self::Tts,
            PlaybackKind::Audio => Self::Audio,
            PlaybackKind::Video => Self::Video,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PlaybackStateResponse {
    pub playback_kind: PlaybackKindDto,
    pub position_seconds: f64,
    pub playback_speed: f64,
    pub element_index: Option<i32>,
    pub tts_chunk_id: Option<String>,
    #[schema(value_type = Option<String>)]
    pub tts_voice_persona_id: Option<String>,
    pub is_playing: bool,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

impl PlaybackStateResponse {
    pub fn from_domain(state: PlaybackState) -> Self {
        Self {
            playback_kind: state.playback_kind.into(),
            position_seconds: state.position_seconds,
            playback_speed: state.playback_speed,
            element_index: state.element_index,
            tts_chunk_id: state.tts_chunk_id,
            tts_voice_persona_id: state.tts_voice_persona_id.map(|id| id.to_string()),
            is_playing: state.is_playing,
            updated_at: state.updated_at,
        }
    }
}

fn audio_url(
    document_id: ind_domain::DocumentId,
    session_id: ind_domain::TtsSessionId,
    chunk_id: &str,
    audio_format: AudioFormat,
) -> String {
    format!(
        "/api/v1/assets/documents/{document_id}/tts/{session_id}/{chunk_id}.{}",
        audio_format.as_str()
    )
}

pub fn parse_persona_id(raw: &str) -> Result<TtsVoicePersonaId, ApiError> {
    raw.parse::<TtsVoicePersonaId>()
        .map_err(|_| ApiError::NotFound {
            entity: "VoicePersona",
            id: raw.to_string(),
        })
}
