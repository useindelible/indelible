use async_trait::async_trait;
use bytes::Bytes;
use ind_domain::{AudioFormat, TtsProvider, TtsProviderUsage, TtsTimingSource};
use reqwest::header::{
    AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, RETRY_AFTER,
};
use serde_json::json;

use super::adapter::{
    TtsAdapter, TtsAdapterError, TtsDesignRequest, TtsDesignResult, TtsSynthesisRequest,
    TtsSynthesisResult,
};
use super::http::{
    TtsHttpTimeouts, build_client, classify_status_error, classify_transport_error,
    normalize_dashscope_base, parse_retry_after_ms,
};

/// Default DashScope international endpoint. The US endpoint is
/// `https://dashscope.aliyuncs.com`; deployments can override via
/// `tts.dashscope.api_base`, but the adapter normalizes the form so both
/// `/api/v1` and the bare host resolve identically.
pub const DASHSCOPE_DEFAULT_BASE: &str = "https://dashscope-intl.aliyuncs.com";

/// Default flash-tier synthesis model used when the persona does not pin a
/// specific `provider_model` (e.g. no `qwen3-tts-vd-*` from a design flow).
pub const DEFAULT_SYNTHESIS_MODEL: &str = "qwen3-tts-flash";

/// Default target model for voice design. Used when callers pass no explicit
/// model via `provider_model` on the persona design request.
pub const DEFAULT_VOICE_DESIGN_TARGET_MODEL: &str = "qwen3-tts-vd-2026-01-26";

/// Default voice ID used as a fallback for a persona that never resolved a
/// provider voice. Cherry is one of the documented Qwen3-TTS preset voices.
const DEFAULT_VOICE_ID: &str = "Cherry";

const MULTIMODAL_GENERATION_PATH: &str = "/api/v1/services/aigc/multimodal-generation/generation";
const VOICE_CUSTOMIZATION_PATH: &str = "/api/v1/services/audio/tts/customization";
const VOICE_DESIGN_MODEL: &str = "qwen-voice-design";

/// Fixed neutral preview sample. Sent as `input.preview_text` in voice-design
/// requests. The display name is intentionally excluded because DashScope reads
/// the preview aloud and user-controlled fields (like display_name that may
/// include an email address) must not leak into the outgoing prompt.
const VOICE_DESIGN_PREVIEW_TEXT: &str = "Hello, this is a preview of the selected reading voice.";

/// DashScope header that selects synchronous mode. Streaming/async is out of
/// scope for TASK-196; sending "disable" matches the proof script and keeps
/// the adapter on the single-response code path. The header name itself is
/// baked into `HeaderName::from_static` so only the value travels through a
/// shared constant.
const X_DASHSCOPE_ASYNC_DISABLE: &str = "disable";

#[derive(Debug, Clone)]
pub struct DashScopeAdapter {
    client: reqwest::Client,
    transcript_supported: bool,
}

impl DashScopeAdapter {
    pub fn new() -> Result<Self, TtsAdapterError> {
        Ok(Self {
            client: build_client(TtsHttpTimeouts::default())?,
            transcript_supported: false,
        })
    }

    pub fn with_client(client: reqwest::Client) -> Self {
        Self {
            client,
            transcript_supported: false,
        }
    }

    pub fn with_transcript_support(mut self, transcript_supported: bool) -> Self {
        self.transcript_supported = transcript_supported;
        self
    }

    fn resolved_base(&self, api_base: Option<&str>) -> String {
        let base = api_base.unwrap_or(DASHSCOPE_DEFAULT_BASE);
        normalize_dashscope_base(base)
    }

    fn auth_headers(api_key: &str) -> Result<HeaderMap, TtsAdapterError> {
        let mut headers = HeaderMap::new();
        let bearer = format!("Bearer {api_key}");
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&bearer).map_err(|_| {
                TtsAdapterError::InvalidRequest("api key contains invalid header bytes".into())
            })?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            HeaderName::from_static("x-dashscope-async"),
            HeaderValue::from_static(X_DASHSCOPE_ASYNC_DISABLE),
        );
        Ok(headers)
    }

    fn require_api_key(key: Option<&str>) -> Result<&str, TtsAdapterError> {
        key.and_then(|k| if k.trim().is_empty() { None } else { Some(k) })
            .ok_or_else(|| {
                TtsAdapterError::AuthenticationFailed("api key is required for DashScope".into())
            })
    }

    /// DashScope's synthesis request requires a `language_type` that matches
    /// the voice's native language. Preset Chinese voices (`longhua`,
    /// `longyue`, `longxiaochun`, `loongstella`) must use `"Chinese"`; all
    /// others default to `"English"`. Custom voices from the design flow are
    /// English-by-default in this deployment — future multi-language support
    /// is a separate persona-model change.
    fn synthesis_language_type(voice_id: &str) -> &'static str {
        match voice_id {
            "longhua" | "longyue" | "longxiaochun" | "loongstella" => "Chinese",
            _ => "English",
        }
    }

    /// DashScope's voice-design endpoint uses a short two-letter `language`
    /// code (`"en"` or `"zh"`). The two endpoints historically disagree on
    /// the casing, so the adapter maintains them separately rather than
    /// sharing one string.
    fn design_language_code(voice_id: &str) -> &'static str {
        match voice_id {
            "longhua" | "longyue" | "longxiaochun" | "loongstella" => "zh",
            _ => "en",
        }
    }
}

#[async_trait]
impl TtsAdapter for DashScopeAdapter {
    fn provider(&self) -> TtsProvider {
        TtsProvider::DashScope
    }

    fn timing_source(&self) -> TtsTimingSource {
        if self.transcript_supported {
            TtsTimingSource::ProviderTranscript
        } else {
            TtsTimingSource::Heuristic
        }
    }

    #[tracing::instrument(
        skip(self, request),
        fields(provider = %self.provider().as_str())
    )]
    async fn synthesize(
        &self,
        request: TtsSynthesisRequest<'_>,
    ) -> Result<TtsSynthesisResult, TtsAdapterError> {
        let api_key = Self::require_api_key(request.api_key)?;
        let base = self.resolved_base(request.api_base);
        let url = format!("{base}{MULTIMODAL_GENERATION_PATH}");

        let model = request
            .provider_model
            .or(request.persona.provider_model.as_deref())
            .unwrap_or(DEFAULT_SYNTHESIS_MODEL);
        let voice = request
            .provider_voice_id
            .or(request.persona.provider_voice_id.as_deref())
            .unwrap_or(DEFAULT_VOICE_ID);

        let language_type = Self::synthesis_language_type(voice);
        let payload = json!({
            "model": model,
            "input": {
                "text": request.normalized_text,
                "voice": voice,
                "language_type": language_type,
            },
            "parameters": {}
        });

        let response = self
            .client
            .post(&url)
            .headers(Self::auth_headers(api_key)?)
            .json(&payload)
            .send()
            .await
            .map_err(classify_transport_error)?;

        let status = response.status();
        if !status.is_success() {
            let retry_after_ms = parse_retry_after_ms(
                response
                    .headers()
                    .get(RETRY_AFTER)
                    .and_then(|v| v.to_str().ok()),
            );
            let body = response.text().await.unwrap_or_default();
            return Err(classify_status_error(status, retry_after_ms, body));
        }

        let parsed: DashScopeSynthesisResponse = response.json().await.map_err(|e| {
            TtsAdapterError::MalformedResponse(format!("failed to decode dashscope body: {e}"))
        })?;

        let (audio_bytes, _response_content_type) =
            self.extract_audio(&parsed.output.audio).await?;
        // Official Qwen3-TTS docs define non-streaming synthesis as WAV output.
        // Keep production routing contract-driven rather than sniffing bytes.
        let audio_format = AudioFormat::Wav;
        let content_type = audio_format.content_type().to_string();
        let audio_bytes = normalize_wav_header(audio_bytes);
        let duration_seconds = wav_duration_seconds(&audio_bytes);

        let characters = request.normalized_text.chars().count() as i64;
        let usage = TtsProviderUsage {
            characters: Some(characters),
            audio_seconds: duration_seconds,
            cost_units: parsed.usage.and_then(|u| u.tts_tokens.or(u.output_tokens)),
        };

        Ok(TtsSynthesisResult {
            audio: Bytes::from(audio_bytes),
            content_type,
            duration_seconds,
            usage,
            element_timings: Vec::new(),
            audio_format: Some(audio_format),
        })
    }

    #[tracing::instrument(
        skip(self, request),
        fields(provider = %self.provider().as_str())
    )]
    async fn design_voice(
        &self,
        request: TtsDesignRequest<'_>,
    ) -> Result<TtsDesignResult, TtsAdapterError> {
        let api_key = Self::require_api_key(request.api_key)?;
        let base = self.resolved_base(request.api_base);
        let url = format!("{base}{VOICE_CUSTOMIZATION_PATH}");

        let target_model = request
            .persona
            .provider_model
            .as_deref()
            .unwrap_or(DEFAULT_VOICE_DESIGN_TARGET_MODEL);
        // Send only the caller-supplied design prompt. The persona's
        // `style_prompt` stays on the persona for future provider-specific
        // shaping; we deliberately do not concatenate it into `voice_prompt`
        // because DashScope's customization endpoint does not document a
        // "style" channel and merging would pollute the generated voice.
        let voice_prompt = request.design_prompt;
        let preferred_name = slugify_preferred_name(&request.persona.display_name);
        let language =
            Self::design_language_code(request.persona.provider_voice_id.as_deref().unwrap_or(""));
        let audio_format = AudioFormat::Wav;

        let payload = json!({
            "model": VOICE_DESIGN_MODEL,
            "input": {
                "action": "create",
                "target_model": target_model,
                "voice_prompt": voice_prompt,
                "preview_text": VOICE_DESIGN_PREVIEW_TEXT,
                "preferred_name": preferred_name,
                "language": language,
            },
            "parameters": {
                "sample_rate": 24000,
                "response_format": audio_format.as_str(),
            }
        });

        let response = self
            .client
            .post(&url)
            .headers(Self::auth_headers(api_key)?)
            .json(&payload)
            .send()
            .await
            .map_err(classify_transport_error)?;

        let status = response.status();
        if !status.is_success() {
            let retry_after_ms = parse_retry_after_ms(
                response
                    .headers()
                    .get(RETRY_AFTER)
                    .and_then(|v| v.to_str().ok()),
            );
            let body = response.text().await.unwrap_or_default();
            return Err(classify_status_error(status, retry_after_ms, body));
        }

        let parsed: DashScopeDesignResponse = response.json().await.map_err(|e| {
            TtsAdapterError::MalformedResponse(format!("failed to decode dashscope body: {e}"))
        })?;

        let voice_id = parsed.output.voice.ok_or_else(|| {
            TtsAdapterError::MalformedResponse(
                "dashscope voice design response did not include output.voice".into(),
            )
        })?;

        Ok(TtsDesignResult {
            provider_job_id: parsed.request_id,
            provider_voice_id: Some(voice_id),
            provider_model: Some(target_model.to_string()),
        })
    }

    fn supports_voice_design(&self) -> bool {
        true
    }
}

fn slugify_preferred_name(display_name: &str) -> String {
    let mut out = String::with_capacity(display_name.len());
    for ch in display_name.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if ch == ' ' || ch == '-' || ch == '_' {
            out.push('-');
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "persona".to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests;

mod audio;
mod dto;

use audio::{normalize_wav_header, wav_duration_seconds};
use dto::{DashScopeDesignResponse, DashScopeSynthesisResponse};
