use std::sync::Arc;

use chrono::Utc;
use ind_domain::{
    AudioFormat, BillingUsageEvent, DocumentId, TtsAudioAsset, TtsChunk, TtsElementTiming,
    TtsProvider, TtsSpokenElement, TtsTimingSource, TtsVoicePersona, UserId,
};

use crate::ports::TtsAdapter;

/// Quota names emitted for managed-billing TTS usage. These align with the
/// managed-entitlement matrix consumed by the billing crate.
pub const TTS_MANAGED_CHARS_QUOTA: &str = "tts_managed_chars";
pub const TTS_MANAGED_SECONDS_QUOTA: &str = "tts_managed_seconds";
pub const TTS_MANAGED_COST_UNITS_QUOTA: &str = "tts_managed_cost_units";

/// Quota limits applied per month when the billing layer has no user-specific
/// override. The managed entitlement check uses these as defaults so a manager
/// never has to plumb per-user limits into synthesis.
#[derive(Debug, Clone, Copy)]
pub struct TtsManagedLimits {
    pub monthly_characters: i64,
    pub monthly_seconds: i64,
    pub monthly_cost_units: i64,
}

impl Default for TtsManagedLimits {
    fn default() -> Self {
        Self {
            // Large defaults — these are only enforced when no per-user
            // override is wired in. The billing layer is the source of truth.
            monthly_characters: 1_000_000,
            monthly_seconds: 1_000_000,
            monthly_cost_units: 1_000_000,
        }
    }
}

/// Input to a single-chunk synthesis call. The caller is responsible for
/// selecting the chunk text and the element range it covers — the service
/// takes it from here, dedupes via cache, and persists all resulting rows.
#[derive(Debug, Clone)]
pub struct SynthesizeChunkInput<'a> {
    pub user_id: UserId,
    pub document_id: DocumentId,
    pub chunk_id: &'a str,
    pub text: &'a str,
    pub normalized_text: &'a str,
    pub elements: &'a [TtsSpokenElement],
    pub start_element_index: i32,
    pub end_element_index: i32,
    pub persona: &'a TtsVoicePersona,
    pub provider_model: Option<&'a str>,
    pub provider_voice_id: Option<&'a str>,
    pub pitch: f64,
    pub audio_format: AudioFormat,
    pub sample_rate: i32,
    pub pronunciation_version: i32,
    pub chunking_version: i32,
    pub managed_character_limit: Option<i64>,
    pub api_key: Option<&'a str>,
    pub api_base: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct SynthesizeChunkOutcome {
    pub chunk: TtsChunk,
    pub audio_asset: TtsAudioAsset,
    pub element_timings: Vec<TtsElementTiming>,
    pub timing_source: TtsTimingSource,
    pub cache_hit: bool,
    pub usage: Option<BillingUsageEvent>,
    /// Effective format of the persisted chunk audio. This may differ from
    /// `SynthesizeChunkInput::audio_format` when a provider has a documented
    /// fixed output format for the chosen API path.
    pub audio_format: AudioFormat,
}

#[derive(Debug, Clone)]
pub(super) struct ManagedQuotaReservation {
    pub(super) period_start: chrono::DateTime<Utc>,
    pub(super) period_end: chrono::DateTime<Utc>,
    pub(super) character_limit: i64,
    pub(super) seconds_limit: i64,
    pub(super) cost_units_limit: i64,
    pub(super) characters: i64,
    pub(super) seconds: i64,
    pub(super) cost_units: i64,
}

/// Maps (provider) → adapter. The registry type lives in `ind-ai` but the
/// application layer cannot depend on it; this thin resolver avoids that.
pub type TtsAdapterResolver = Arc<dyn Fn(TtsProvider) -> Option<Arc<dyn TtsAdapter>> + Send + Sync>;
