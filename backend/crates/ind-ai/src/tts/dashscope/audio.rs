use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use reqwest::header::CONTENT_TYPE;
use std::io::Cursor;

use crate::tts::adapter::TtsAdapterError;
use crate::tts::http::classify_transport_error;

use super::DashScopeAdapter;
use super::dto::DashScopeAudioPayload;

pub(super) fn wav_duration_seconds(bytes: &[u8]) -> Option<f64> {
    let reader = hound::WavReader::new(Cursor::new(bytes)).ok()?;
    let spec = reader.spec();
    if spec.sample_rate == 0 {
        return None;
    }
    let sample_count = reader.duration() as f64;
    let duration = sample_count / spec.sample_rate as f64;
    duration.is_finite().then_some(duration)
}

pub(super) fn normalize_wav_header(mut bytes: Vec<u8>) -> Vec<u8> {
    if bytes.len() < 44 || bytes.len() > (u32::MAX as usize) {
        return bytes;
    }
    if &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return bytes;
    }

    let riff_size = (bytes.len() as u32).saturating_sub(8);
    bytes[4..8].copy_from_slice(&riff_size.to_le_bytes());

    let mut offset = 12usize;
    while offset + 8 <= bytes.len() {
        let chunk_size = u32::from_le_bytes([
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]) as usize;
        let payload_start = offset + 8;
        if &bytes[offset..offset + 4] == b"data" {
            let data_size = bytes.len().saturating_sub(payload_start) as u32;
            bytes[offset + 4..offset + 8].copy_from_slice(&data_size.to_le_bytes());
            break;
        }
        let padded_size = chunk_size + (chunk_size % 2);
        let Some(next_offset) = payload_start.checked_add(padded_size) else {
            break;
        };
        if next_offset > bytes.len() {
            break;
        }
        offset = next_offset;
    }

    bytes
}

impl DashScopeAdapter {
    /// Turn a DashScope `output.audio` block into raw bytes plus an optional
    /// content-type. DashScope returns the audio as inline base64 (`data`) or
    /// as a signed download URL (`url`); the URL form is common for
    /// longer-form synthesis. The signed URLs do not accept the DashScope
    /// bearer token on the follow-up GET — the signature IS the
    /// authentication — so the download request is issued without the
    /// `Authorization` or `X-DashScope-Async` headers.
    pub(super) async fn extract_audio(
        &self,
        audio: &DashScopeAudioPayload,
    ) -> Result<(Vec<u8>, Option<String>), TtsAdapterError> {
        if let Some(data) = audio.data.as_deref() {
            // DashScope may return an empty inline `data` field alongside a
            // signed download URL. Treat only non-empty inline audio as
            // authoritative and fall through to `url` otherwise.
            if !data.trim().is_empty() {
                let decoded = BASE64_STANDARD.decode(data).map_err(|e| {
                    TtsAdapterError::MalformedResponse(format!(
                        "failed to decode dashscope audio data: {e}"
                    ))
                })?;
                if decoded.is_empty() {
                    return Err(TtsAdapterError::MalformedResponse(
                        "dashscope audio payload decoded to an empty body".into(),
                    ));
                }
                return Ok((decoded, None));
            }
        }
        let url = audio.url.as_deref().ok_or_else(|| {
            TtsAdapterError::MalformedResponse(
                "dashscope audio payload contained neither data nor url".into(),
            )
        })?;
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(classify_transport_error)?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(TtsAdapterError::ProviderError {
                status_code: status.as_u16(),
                message: format!("audio download failed: {body}"),
            });
        }
        let response_content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        let bytes = response.bytes().await.map_err(|e| {
            TtsAdapterError::MalformedResponse(format!(
                "failed to download dashscope audio bytes: {e}"
            ))
        })?;
        if bytes.is_empty() {
            return Err(TtsAdapterError::MalformedResponse(
                "dashscope audio download returned empty body".into(),
            ));
        }
        Ok((bytes.to_vec(), response_content_type))
    }
}
