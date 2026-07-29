pub mod element_source;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub use element_source::{
    TtsChunkHint, TtsElementFeed, TtsElementKind, TtsElementSource, TtsSpokenElement,
};

use crate::id::{
    DocumentId, TtsAudioAssetId, TtsChunkRecordId, TtsSessionId, TtsVoicePersonaId, UserId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TtsProvider {
    DashScope,
    ElevenLabs,
    UnrealSpeech,
    Inworld,
    Gemini,
    Polly,
    Mock,
}

impl TtsProvider {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DashScope => "dashscope",
            Self::ElevenLabs => "elevenlabs",
            Self::UnrealSpeech => "unreal_speech",
            Self::Inworld => "inworld",
            Self::Gemini => "gemini",
            Self::Polly => "polly",
            Self::Mock => "mock",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "dashscope" => Some(Self::DashScope),
            "elevenlabs" => Some(Self::ElevenLabs),
            "unreal_speech" => Some(Self::UnrealSpeech),
            "inworld" => Some(Self::Inworld),
            "gemini" => Some(Self::Gemini),
            "polly" => Some(Self::Polly),
            "mock" => Some(Self::Mock),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TtsTimingSource {
    ProviderTranscript,
    Heuristic,
}

impl TtsTimingSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ProviderTranscript => "provider_transcript",
            Self::Heuristic => "heuristic",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioFormat {
    Mp3,
    Wav,
    Ogg,
    Opus,
    Pcm,
}

impl AudioFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mp3 => "mp3",
            Self::Wav => "wav",
            Self::Ogg => "ogg",
            Self::Opus => "opus",
            Self::Pcm => "pcm",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "mp3" => Some(Self::Mp3),
            "wav" => Some(Self::Wav),
            "ogg" => Some(Self::Ogg),
            "opus" => Some(Self::Opus),
            "pcm" => Some(Self::Pcm),
            _ => None,
        }
    }

    pub fn content_type(self) -> &'static str {
        match self {
            Self::Mp3 => "audio/mpeg",
            Self::Wav => "audio/wav",
            Self::Ogg => "audio/ogg",
            Self::Opus => "audio/opus",
            Self::Pcm => "audio/L16",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TtsChunkStatus {
    Ready,
    Pending,
    Failed,
}

impl TtsChunkStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Pending => "pending",
            Self::Failed => "failed",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "ready" => Some(Self::Ready),
            "pending" => Some(Self::Pending),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TtsGenerationScope {
    SingleChunk,
    Section,
    Chapter,
}

impl TtsGenerationScope {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SingleChunk => "single_chunk",
            Self::Section => "section",
            Self::Chapter => "chapter",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "single_chunk" => Some(Self::SingleChunk),
            "section" => Some(Self::Section),
            "chapter" => Some(Self::Chapter),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TtsPersonaStatus {
    Active,
    PendingDesign,
    Failed,
    Archived,
}

impl TtsPersonaStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::PendingDesign => "pending_design",
            Self::Failed => "failed",
            Self::Archived => "archived",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "active" => Some(Self::Active),
            "pending_design" => Some(Self::PendingDesign),
            "failed" => Some(Self::Failed),
            "archived" => Some(Self::Archived),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TtsVoicePersona {
    pub id: TtsVoicePersonaId,
    pub user_id: Option<UserId>,
    pub display_name: String,
    pub description: Option<String>,
    pub provider: TtsProvider,
    pub provider_voice_id: Option<String>,
    pub provider_model: Option<String>,
    pub design_prompt: Option<String>,
    pub style_prompt: Option<String>,
    pub pace: Option<String>,
    pub energy: Option<String>,
    pub warmth: Option<String>,
    pub formality: Option<String>,
    pub pronunciation_prefs: serde_json::Value,
    pub status: TtsPersonaStatus,
    pub is_builtin: bool,
    pub prompt_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct TtsChunk {
    pub id: TtsChunkRecordId,
    pub user_id: UserId,
    pub document_id: DocumentId,
    pub chunk_id: String,
    pub cache_key: String,
    pub voice_persona_id: Option<TtsVoicePersonaId>,
    pub provider: TtsProvider,
    pub provider_model: Option<String>,
    pub provider_voice_id: Option<String>,
    pub pitch: f64,
    pub audio_format: AudioFormat,
    pub sample_rate: i32,
    pub pronunciation_version: i32,
    pub chunking_version: i32,
    pub normalized_text_hash: String,
    pub start_element_index: i32,
    pub end_element_index: i32,
    pub duration_seconds: Option<f64>,
    pub status: TtsChunkStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct TtsAudioAsset {
    pub id: TtsAudioAssetId,
    pub user_id: UserId,
    pub chunk_record_id: TtsChunkRecordId,
    pub s3_key: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct TtsElementTiming {
    pub chunk_record_id: TtsChunkRecordId,
    pub element_index: i32,
    pub start_timestamp: f64,
    pub end_timestamp: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct TtsSession {
    pub id: TtsSessionId,
    pub user_id: UserId,
    pub document_id: DocumentId,
    pub voice_persona_id: Option<TtsVoicePersonaId>,
    pub speed: f64,
    pub audio_format: AudioFormat,
    pub generation_scope: TtsGenerationScope,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct TtsSessionChunk {
    pub session_id: TtsSessionId,
    pub chunk_id: String,
    pub chunk_record_id: TtsChunkRecordId,
    pub position: i32,
}

/// Provider-reported actual usage for a synthesis request.
#[derive(Debug, Clone, Default)]
pub struct TtsProviderUsage {
    pub characters: Option<i64>,
    pub audio_seconds: Option<f64>,
    pub cost_units: Option<i64>,
}

/// Provider-reported per-element timing used as optional enrichment.
#[derive(Debug, Clone)]
pub struct ProviderElementTiming {
    pub element_index: i32,
    pub start_timestamp: f64,
    pub end_timestamp: Option<f64>,
}
