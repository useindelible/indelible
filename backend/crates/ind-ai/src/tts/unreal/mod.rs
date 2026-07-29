use async_trait::async_trait;
use bytes::Bytes;
use ind_domain::{
    AudioFormat, ProviderElementTiming, TtsProvider, TtsProviderUsage, TtsTimingSource,
};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue, RETRY_AFTER};
use serde_json::json;

use super::adapter::{
    TtsAdapter, TtsAdapterError, TtsDesignRequest, TtsDesignResult, TtsSynthesisRequest,
    TtsSynthesisResult,
};
use super::http::{
    TtsHttpTimeouts, build_client, classify_status_error, classify_transport_error,
    parse_retry_after_ms,
};

/// Default Unreal Speech API host. Deployments can override this through TTS
/// configuration; adapter callers pass the resolved base explicitly.
pub const UNREAL_DEFAULT_BASE: &str = "https://api.v8.unrealspeech.com";

/// Default Unreal Speech voice when a persona has no `provider_voice_id`.
/// Sierra is a supported American female voice for the v8 `/speech` endpoint.
pub(crate) const DEFAULT_VOICE_ID: &str = "Sierra";

/// Default synthesis bitrate. Unreal accepts `128k`, `192k`, `256k`, `320k`
/// for MP3 and discrete sample rates for PCM. The default matches the preset
/// the provider suggests for long-form content.
const DEFAULT_BITRATE: &str = "192k";

const SYNTHESIS_PATH: &str = "/speech";

#[derive(Debug, Clone)]
pub struct UnrealSpeechAdapter {
    client: reqwest::Client,
    transcript_supported: bool,
}

impl UnrealSpeechAdapter {
    /// Hard ceiling on the provider's `/speech` endpoint (3000 characters per
    /// call). Longer inputs are broken into sub-requests and concatenated.
    /// Exposed at `impl` scope so tests and split-mode logic reference the
    /// same value without a separate free constant going out of date.
    pub(crate) const MAX_CHARS_PER_REQUEST: usize = 3000;

    pub fn new() -> Result<Self, TtsAdapterError> {
        Ok(Self {
            client: build_client(TtsHttpTimeouts::default())?,
            transcript_supported: true,
        })
    }

    pub fn with_client(client: reqwest::Client) -> Self {
        Self {
            client,
            transcript_supported: true,
        }
    }

    pub fn with_transcript_support(mut self, transcript_supported: bool) -> Self {
        self.transcript_supported = transcript_supported;
        self
    }

    fn resolved_base(&self, api_base: Option<&str>) -> String {
        api_base
            .map(|b| b.trim().trim_end_matches('/').to_string())
            .unwrap_or_else(|| UNREAL_DEFAULT_BASE.to_string())
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
        Ok(headers)
    }

    fn require_api_key(key: Option<&str>) -> Result<&str, TtsAdapterError> {
        key.and_then(|k| if k.trim().is_empty() { None } else { Some(k) })
            .ok_or_else(|| {
                TtsAdapterError::AuthenticationFailed(
                    "api key is required for Unreal Speech".into(),
                )
            })
    }

    /// Map a domain `AudioFormat` to the provider's `/speech` output format.
    /// For v1 this adapter is intentionally MP3-only: Unreal exposes PCM/WAV
    /// controls on `/stream`, not the `/speech` path used for reader chunks.
    fn output_format_name(audio_format: AudioFormat) -> Result<&'static str, TtsAdapterError> {
        match audio_format {
            AudioFormat::Mp3 => Ok("mp3"),
            other => Err(TtsAdapterError::InvalidRequest(format!(
                "unreal speech does not support audio format {}",
                other.as_str()
            ))),
        }
    }
}

#[async_trait]
impl TtsAdapter for UnrealSpeechAdapter {
    fn provider(&self) -> TtsProvider {
        TtsProvider::UnrealSpeech
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
        let url = format!("{base}{SYNTHESIS_PATH}");

        let voice_id = request
            .provider_voice_id
            .or(request.persona.provider_voice_id.as_deref())
            .unwrap_or(DEFAULT_VOICE_ID);

        let windows = split_for_request(request.normalized_text, Self::MAX_CHARS_PER_REQUEST)?;
        if windows.is_empty() {
            return Err(TtsAdapterError::InvalidRequest(
                "normalized text is empty".into(),
            ));
        }

        // Unreal's v8 `/speech` endpoint is used as MP3-only in this adapter.
        // Multi-window MP3 concatenation strips leading ID3v2 tags from all
        // windows after the first so the persisted asset remains one MP3 file.
        let output_format = Self::output_format_name(request.audio_format)?;
        let bitrate = DEFAULT_BITRATE;

        let headers = Self::auth_headers(api_key)?;

        let mut audio = Vec::<u8>::new();
        let mut element_timings = Vec::<ProviderElementTiming>::new();
        let element_ranges = element_ranges_for_text(request.normalized_text, request.elements);
        let mut cumulative_timestamp_seconds = 0.0;
        let mut content_type = request.audio_format.content_type().to_string();
        let is_multi_window_mp3 = windows.len() > 1 && request.audio_format == AudioFormat::Mp3;

        for (index, window) in windows.iter().enumerate() {
            let mut payload = json!({
                "Text": window.text,
                "VoiceId": voice_id,
                "Bitrate": bitrate,
                "OutputFormat": output_format,
                "Pitch": request.pitch,
            });
            if self.transcript_supported {
                payload["TimestampType"] = json!("word");
            }

            let response = self
                .client
                .post(&url)
                .headers(headers.clone())
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

            let asset = self.decode_speech_response(response).await?;
            if asset.bytes.is_empty() {
                return Err(TtsAdapterError::MalformedResponse(
                    "unreal speech returned empty audio body".into(),
                ));
            }
            if let Some(ct) = asset.content_type {
                content_type = ct;
            }
            if self.transcript_supported
                && let Some(timestamps_uri) = asset.timestamps_uri.as_deref()
            {
                let rows = self.fetch_timestamp_rows(timestamps_uri).await?;
                element_timings.extend(map_timestamp_rows_to_elements(
                    &element_ranges,
                    &rows,
                    window.start_char,
                    cumulative_timestamp_seconds,
                ));
                if let Some(window_duration) = timestamp_window_duration(&rows) {
                    cumulative_timestamp_seconds += window_duration;
                }
            }
            let payload_bytes: &[u8] = if is_multi_window_mp3 && index > 0 {
                strip_id3v2_prefix(&asset.bytes)
            } else {
                &asset.bytes
            };
            audio.extend_from_slice(payload_bytes);
        }

        let characters = request.normalized_text.chars().count() as i64;
        let usage = TtsProviderUsage {
            characters: Some(characters),
            audio_seconds: None,
            cost_units: None,
        };

        Ok(TtsSynthesisResult {
            audio: Bytes::from(audio),
            content_type,
            duration_seconds: None,
            usage,
            element_timings: merge_element_timings(element_timings),
            // Single-window and multi-window paths both persist at the
            // requested format: single windows return one MP3 verbatim, and
            // multi-window MP3 concatenation strips ID3v2 tags so the joined
            // bytes are still a valid MP3 stream. `None` tells the service
            // layer to use the caller-requested format for the S3 extension.
            audio_format: None,
        })
    }

    #[tracing::instrument(
        skip(self, _request),
        fields(provider = %self.provider().as_str())
    )]
    async fn design_voice(
        &self,
        _request: TtsDesignRequest<'_>,
    ) -> Result<TtsDesignResult, TtsAdapterError> {
        // Unreal Speech does not currently expose a compatible prompt-based
        // voice design flow. Returning Unsupported keeps PersonaService's
        // capability check honest — the persona row is never marked active
        // with a missing provider voice.
        Err(TtsAdapterError::Unsupported(
            "unreal speech does not support provider-side voice design".into(),
        ))
    }

    fn supports_voice_design(&self) -> bool {
        false
    }

    fn timing_source(&self) -> TtsTimingSource {
        if self.transcript_supported {
            TtsTimingSource::ProviderTranscript
        } else {
            TtsTimingSource::Heuristic
        }
    }
}

mod response;
mod split;
mod timestamps;

use split::{split_for_request, strip_id3v2_prefix};
use timestamps::{
    element_ranges_for_text, map_timestamp_rows_to_elements, merge_element_timings,
    timestamp_window_duration,
};
