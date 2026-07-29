use std::sync::Arc;

use chrono::Utc;
use ind_domain::{
    AudioFormat, DocumentId, DomainError, TtsChunkRecordId, TtsElementSource, TtsElementTiming,
    TtsGenerationScope, TtsSession, TtsSessionChunk, TtsSessionId, TtsTimingSource,
    TtsVoicePersona, TtsVoicePersonaId, UserId,
};
use uuid::Uuid;

use super::credentials::TtsProviderCredentialResolver;
use super::elements::{TtsChunkingProfile, replan_feed_chunks};
use super::entitlements::TtsEntitlements;
use super::synthesis::{SynthesisService, SynthesizeChunkInput, SynthesizeChunkOutcome};
use crate::AppError;
use crate::repos::tts_session::TtsSessionRepository;
use crate::repos::tts_voice_persona::TtsVoicePersonaRepository;
use crate::storage::{ByteRange, RangedObjectData};

const LOOKAHEAD_READY_CHUNKS: usize = 2;

/// Input for starting a playback session on an document.
#[derive(Debug, Clone)]
pub struct StartSessionInput {
    pub document_id: DocumentId,
    pub voice_persona_id: Option<TtsVoicePersonaId>,
    pub speed: f64,
    pub audio_format: AudioFormat,
    pub sample_rate: i32,
    pub generation_scope: TtsGenerationScope,
    pub pronunciation_version: i32,
    pub chunking_version: i32,
    /// Element index to resume playback from. TASK-195 currently synthesizes
    /// all planned chunks synchronously, so this is used only to preserve the
    /// client-facing resume contract.
    pub start_element_index: Option<i32>,
}

/// Planned chunk the session manifest will describe.
#[derive(Debug, Clone)]
pub struct PlannedChunk {
    pub chunk_id: String,
    pub position: i32,
    pub start_element_index: i32,
    pub end_element_index: i32,
    pub state: PlannedChunkState,
}

#[derive(Debug, Clone)]
pub enum PlannedChunkState {
    /// Chunk audio is ready — the client may fetch the audio asset directly.
    Ready {
        chunk_record_id: TtsChunkRecordId,
        duration_seconds: Option<f64>,
        cache_hit: bool,
        timings: Vec<TtsElementTiming>,
        timing_source: TtsTimingSource,
        /// Effective format of the persisted chunk audio. May differ from
        /// `TtsSession::audio_format` when the adapter had to coerce the wire
        /// format (e.g. Unreal multi-window falls back from MP3 to PCM). The
        /// manifest builder uses this for the audio URL extension so clients
        /// download bytes that match the extension they requested.
        audio_format: AudioFormat,
    },
}

/// The full manifest returned when a playback session is created.
#[derive(Debug, Clone)]
pub struct TtsSessionManifest {
    pub session: TtsSession,
    pub persona: TtsVoicePersona,
    pub document_title: String,
    pub start: TtsSessionStart,
    pub chunks: Vec<PlannedChunk>,
}

#[derive(Debug, Clone)]
pub struct TtsSessionStart {
    pub chunk_id: String,
    pub chunk_record_id: TtsChunkRecordId,
    pub element_index: i32,
    pub start_timestamp: f64,
}

#[derive(Debug, Clone)]
pub struct TtsResolvedChunk {
    pub session_chunk: TtsSessionChunk,
    pub chunk: ind_domain::TtsChunk,
}

pub struct TtsSessionService {
    personas: Arc<dyn TtsVoicePersonaRepository>,
    sessions: Arc<dyn TtsSessionRepository>,
    element_source: Arc<dyn TtsElementSource>,
    synthesis: Arc<SynthesisService>,
    credentials: Arc<dyn TtsProviderCredentialResolver>,
    entitlements: Arc<TtsEntitlements>,
}

impl TtsSessionService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        personas: Arc<dyn TtsVoicePersonaRepository>,
        sessions: Arc<dyn TtsSessionRepository>,
        element_source: Arc<dyn TtsElementSource>,
        synthesis: Arc<SynthesisService>,
        credentials: Arc<dyn TtsProviderCredentialResolver>,
        entitlements: Arc<TtsEntitlements>,
    ) -> Self {
        Self {
            personas,
            sessions,
            element_source,
            synthesis,
            credentials,
            entitlements,
        }
    }

    /// Creates a session and synchronously synthesizes the planned chunks for
    /// the requested playback scope so clients can advance without issuing a
    /// new session request after each chunk finishes.
    pub async fn start_session(
        &self,
        user_id: UserId,
        input: StartSessionInput,
    ) -> Result<TtsSessionManifest, AppError> {
        let persona = self
            .resolve_persona(user_id, input.voice_persona_id)
            .await?;

        let credentials = self.credentials.resolve(persona.provider).await?;
        let api_key = credentials.api_key;
        let api_base = credentials.api_base;
        let managed_character_limit = Some(self.entitlements.managed_monthly_character_limit());
        let session_audio_format = match persona.provider {
            // Qwen3-TTS non-streaming returns WAV per the provider contract.
            ind_domain::TtsProvider::DashScope => AudioFormat::Wav,
            _ => input.audio_format,
        };

        let feed = self
            .element_source
            .elements(user_id, input.document_id)
            .await
            .map_err(AppError::Domain)?;
        let feed = if self.synthesis.provider_timing_source(persona.provider)
            == TtsTimingSource::ProviderTranscript
        {
            feed
        } else {
            replan_feed_chunks(feed, TtsChunkingProfile::duration_only_provider())
        };
        if feed.chunk_hints.is_empty() || feed.elements.is_empty() {
            return Err(AppError::Domain(DomainError::Validation {
                field: "document_id".into(),
                message: "document produced no TTS-playable content".into(),
            }));
        }

        let session_id = TtsSessionId::from_uuid(Uuid::now_v7());
        let session = TtsSession {
            id: session_id,
            user_id,
            document_id: input.document_id,
            voice_persona_id: Some(persona.id),
            speed: input.speed,
            audio_format: session_audio_format,
            generation_scope: input.generation_scope,
            created_at: Utc::now(),
        };

        let provider_model = persona.provider_model.clone();
        let provider_voice_id = persona.provider_voice_id.clone();

        let requested_element_index = input
            .start_element_index
            .unwrap_or_else(|| feed.chunk_hints[0].start_element_index);
        let start_position = feed
            .chunk_hints
            .iter()
            .position(|h| {
                h.start_element_index <= requested_element_index
                    && requested_element_index <= h.end_element_index
            })
            .ok_or_else(|| {
                AppError::Domain(DomainError::Validation {
                    field: "start_element_index".into(),
                    message: "does not match a playable TTS chunk".into(),
                })
            })?;

        let end_position = match input.generation_scope {
            TtsGenerationScope::SingleChunk => start_position + 1,
            TtsGenerationScope::Section | TtsGenerationScope::Chapter => {
                (start_position + 1 + LOOKAHEAD_READY_CHUNKS).min(feed.chunk_hints.len())
            }
        };

        let mut planned: Vec<PlannedChunk> = Vec::with_capacity(end_position - start_position);
        let mut session_chunks: Vec<TtsSessionChunk> = Vec::new();
        let mut start: Option<TtsSessionStart> = None;

        for (position, hint) in feed
            .chunk_hints
            .iter()
            .enumerate()
            .skip(start_position)
            .take(end_position - start_position)
        {
            let chunk_elements = collect_chunk_elements(&feed.elements, hint);
            let chunk_text = collect_chunk_text(&chunk_elements);
            let normalized = chunk_text.trim().to_string();
            if normalized.is_empty() {
                continue;
            }

            let outcome: SynthesizeChunkOutcome = self
                .synthesis
                .synthesize_chunk(SynthesizeChunkInput {
                    user_id,
                    document_id: input.document_id,
                    chunk_id: &hint.chunk_id,
                    text: &chunk_text,
                    normalized_text: &normalized,
                    elements: &chunk_elements,
                    start_element_index: hint.start_element_index,
                    end_element_index: hint.end_element_index,
                    persona: &persona,
                    provider_model: provider_model.as_deref(),
                    provider_voice_id: provider_voice_id.as_deref(),
                    pitch: 1.0,
                    audio_format: session_audio_format,
                    sample_rate: input.sample_rate,
                    pronunciation_version: input.pronunciation_version,
                    chunking_version: input.chunking_version,
                    managed_character_limit,
                    api_key: api_key.as_deref(),
                    api_base: api_base.as_deref(),
                })
                .await?;

            if start.is_none() {
                let start_element_index =
                    requested_element_index.clamp(hint.start_element_index, hint.end_element_index);
                let start_timestamp = outcome
                    .element_timings
                    .iter()
                    .find(|timing| timing.element_index == start_element_index)
                    .map(|timing| timing.start_timestamp)
                    .unwrap_or(0.0);
                start = Some(TtsSessionStart {
                    chunk_id: hint.chunk_id.clone(),
                    chunk_record_id: outcome.chunk.id,
                    element_index: start_element_index,
                    start_timestamp,
                });
            }

            session_chunks.push(TtsSessionChunk {
                session_id: session.id,
                chunk_id: hint.chunk_id.clone(),
                chunk_record_id: outcome.chunk.id,
                position: position as i32,
            });
            planned.push(PlannedChunk {
                chunk_id: hint.chunk_id.clone(),
                position: position as i32,
                start_element_index: hint.start_element_index,
                end_element_index: hint.end_element_index,
                state: PlannedChunkState::Ready {
                    chunk_record_id: outcome.chunk.id,
                    duration_seconds: outcome.chunk.duration_seconds,
                    cache_hit: outcome.cache_hit,
                    timings: outcome.element_timings,
                    timing_source: outcome.timing_source,
                    audio_format: outcome.audio_format,
                },
            });
        }

        let session = self.sessions.insert(&session).await?;
        if !session_chunks.is_empty() {
            self.sessions
                .insert_session_chunks(session.id, &session_chunks)
                .await?;
        }
        let start = start.ok_or_else(|| {
            AppError::Domain(DomainError::Validation {
                field: "start_element_index".into(),
                message: "no playable TTS chunk found for requested element".into(),
            })
        })?;

        Ok(TtsSessionManifest {
            session,
            persona,
            document_title: feed.title,
            start,
            chunks: planned,
        })
    }

    pub async fn resolve_chunk(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        session_id: TtsSessionId,
        chunk_id: &str,
    ) -> Result<Option<TtsResolvedChunk>, AppError> {
        let Some(session_chunk) = self
            .sessions
            .resolve_chunk(user_id, document_id, session_id, chunk_id)
            .await?
        else {
            return Ok(None);
        };
        let Some(chunk) = self
            .synthesis
            .get_chunk(user_id, session_chunk.chunk_record_id)
            .await?
        else {
            return Ok(None);
        };

        Ok(Some(TtsResolvedChunk {
            session_chunk,
            chunk,
        }))
    }

    pub async fn resolve_element_timestamp(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        session_id: TtsSessionId,
        chunk_id: &str,
        element_index: i32,
    ) -> Result<Option<TtsElementTiming>, AppError> {
        let Some(resolved) = self
            .resolve_chunk(user_id, document_id, session_id, chunk_id)
            .await?
        else {
            return Ok(None);
        };

        self.synthesis
            .get_timing(resolved.session_chunk.chunk_record_id, element_index)
            .await
    }

    pub async fn get_session_chunk_audio(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        session_id: TtsSessionId,
        chunk_id: &str,
        range: Option<ByteRange>,
    ) -> Result<RangedObjectData, AppError> {
        let Some(resolved) = self
            .resolve_chunk(user_id, document_id, session_id, chunk_id)
            .await?
        else {
            return Err(AppError::Domain(DomainError::NotFound {
                entity: "TtsSessionChunk",
                id: chunk_id.to_string(),
            }));
        };

        self.synthesis
            .get_audio(user_id, resolved.session_chunk.chunk_record_id, range)
            .await
    }

    async fn resolve_persona(
        &self,
        user_id: UserId,
        requested: Option<TtsVoicePersonaId>,
    ) -> Result<TtsVoicePersona, AppError> {
        if let Some(id) = requested {
            return self.personas.get(id, user_id).await?.ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "tts_voice_persona",
                    id: id.to_string(),
                })
            });
        }
        self.personas
            .list_for_user(user_id)
            .await?
            .into_iter()
            .next()
            .ok_or_else(|| {
                AppError::Domain(DomainError::Validation {
                    field: "voice_persona_id".into(),
                    message: "no_voice_configured".into(),
                })
            })
    }
}

fn collect_chunk_elements(
    elements: &[ind_domain::TtsSpokenElement],
    hint: &ind_domain::TtsChunkHint,
) -> Vec<ind_domain::TtsSpokenElement> {
    elements
        .iter()
        .filter(|e| {
            e.element_index >= hint.start_element_index && e.element_index <= hint.end_element_index
        })
        .cloned()
        .collect()
}

/// Collect the raw text for a planned chunk's elements.
fn collect_chunk_text(elements: &[ind_domain::TtsSpokenElement]) -> String {
    elements
        .iter()
        .map(|e| e.text.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}
