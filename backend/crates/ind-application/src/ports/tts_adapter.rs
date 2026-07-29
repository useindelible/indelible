use async_trait::async_trait;
use bytes::Bytes;
use thiserror::Error;

use ind_domain::{
    AudioFormat, ProviderElementTiming, TtsProvider, TtsProviderUsage, TtsSpokenElement,
    TtsTimingSource, TtsVoicePersona,
};

/// Port the application layer uses to issue synthesis and voice-design calls
/// to a TTS provider. Concrete implementations live in `ind-ai::tts` so the
/// application crate never pulls reqwest-specific provider code.
#[async_trait]
pub trait TtsAdapter: Send + Sync {
    fn provider(&self) -> TtsProvider;

    async fn synthesize(
        &self,
        request: TtsSynthesisRequest<'_>,
    ) -> Result<TtsSynthesisResult, TtsAdapterError>;

    async fn design_voice(
        &self,
        request: TtsDesignRequest<'_>,
    ) -> Result<TtsDesignResult, TtsAdapterError>;

    /// Whether this adapter supports provider-side voice design.
    fn supports_voice_design(&self) -> bool {
        false
    }

    /// Declares whether element timing comes from a provider transcript or
    /// must be estimated locally. Transcript adapters must return complete,
    /// provider-authoritative timing for every requested element.
    fn timing_source(&self) -> TtsTimingSource {
        TtsTimingSource::Heuristic
    }
}

#[derive(Debug, Clone)]
pub struct TtsSynthesisRequest<'a> {
    pub persona: &'a TtsVoicePersona,
    pub provider_model: Option<&'a str>,
    pub provider_voice_id: Option<&'a str>,
    pub text: &'a str,
    pub normalized_text: &'a str,
    pub elements: &'a [TtsSpokenElement],
    pub pitch: f64,
    pub audio_format: AudioFormat,
    pub sample_rate: i32,
    pub api_key: Option<&'a str>,
    pub api_base: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct TtsSynthesisResult {
    pub audio: Bytes,
    pub content_type: String,
    pub duration_seconds: Option<f64>,
    pub usage: TtsProviderUsage,
    pub element_timings: Vec<ProviderElementTiming>,
    /// Effective audio format of the returned `audio` bytes when the provider
    /// has a fixed documented format that differs from the caller's preferred
    /// format. `None` means the bytes match `request.audio_format`.
    pub audio_format: Option<AudioFormat>,
}

#[derive(Debug, Clone)]
pub struct TtsDesignRequest<'a> {
    pub persona: &'a TtsVoicePersona,
    pub design_prompt: &'a str,
    pub style_prompt: Option<&'a str>,
    pub api_key: Option<&'a str>,
    pub api_base: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct TtsDesignResult {
    pub provider_job_id: Option<String>,
    pub provider_voice_id: Option<String>,
    /// Provider-side synthesis model identifier the caller should persist on
    /// the persona so subsequent `synthesize` calls reuse the same model.
    /// For DashScope/Qwen this is the resolved `qwen3-tts-vd-YYYY-MM-DD`
    /// target model returned by the voice-design customization endpoint.
    pub provider_model: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TtsAdapterError {
    #[error("tts adapter unsupported operation: {0}")]
    Unsupported(String),
    #[error("tts provider unreachable: {0}")]
    ProviderUnreachable(String),
    #[error("tts provider authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("tts provider returned status {status_code}: {message}")]
    ProviderError { status_code: u16, message: String },
    #[error("tts provider malformed response: {0}")]
    MalformedResponse(String),
    #[error("tts adapter invalid request: {0}")]
    InvalidRequest(String),
    /// Provider-side request throttling. `retry_after_ms` is populated when
    /// the provider reported a structured retry hint (`Retry-After` header or
    /// JSON body), otherwise callers should treat it as an opaque rate-limit
    /// signal.
    #[error("tts provider rate limited (retry_after_ms={retry_after_ms:?})")]
    RateLimited { retry_after_ms: Option<u64> },
    /// Provider-side monthly/account quota exhausted. Distinct from
    /// `RateLimited` because retrying shortly will still fail.
    #[error("tts provider quota exhausted")]
    QuotaExhausted,
    /// The adapter's request to the provider timed out before a response was
    /// received.
    #[error("tts provider request timed out")]
    Timeout,
}
