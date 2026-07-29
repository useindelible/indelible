use async_trait::async_trait;
use bytes::Bytes;

use ind_domain::{ProviderElementTiming, TtsProvider, TtsProviderUsage, TtsTimingSource};

use super::adapter::{
    TtsAdapter, TtsAdapterError, TtsDesignRequest, TtsDesignResult, TtsSynthesisRequest,
    TtsSynthesisResult,
};

/// Deterministic in-memory adapter used by integration tests.
///
/// Produces `normalized_text.len()` bytes of synthetic audio (character `b'A'`
/// repeated) and reports a fixed synthesis rate of 15 characters per second.
/// This keeps test fixtures byte-reproducible while exercising the full
/// application pipeline (cache, storage, session chunk resolution, usage
/// events).
#[derive(Debug, Default, Clone)]
pub struct MockTtsAdapter;

impl MockTtsAdapter {
    pub const CHARS_PER_SECOND: f64 = 15.0;

    pub fn new() -> Self {
        Self
    }

    fn synthetic_audio(len: usize) -> Bytes {
        Bytes::from(vec![b'A'; len.max(1)])
    }
}

#[async_trait]
impl TtsAdapter for MockTtsAdapter {
    fn provider(&self) -> TtsProvider {
        TtsProvider::Mock
    }

    async fn synthesize(
        &self,
        request: TtsSynthesisRequest<'_>,
    ) -> Result<TtsSynthesisResult, TtsAdapterError> {
        if request.normalized_text.is_empty() {
            return Err(TtsAdapterError::InvalidRequest(
                "normalized_text must be non-empty".into(),
            ));
        }

        let chars = request.normalized_text.chars().count();
        let duration = chars as f64 / Self::CHARS_PER_SECOND;
        let audio = Self::synthetic_audio(chars);
        let content_type = request.audio_format.content_type().to_string();

        let usage = TtsProviderUsage {
            characters: Some(chars as i64),
            audio_seconds: Some(duration),
            cost_units: Some(chars as i64),
        };

        let element_timings = mock_element_timings(request.elements, duration);

        Ok(TtsSynthesisResult {
            audio,
            content_type,
            duration_seconds: Some(duration),
            usage,
            element_timings,
            audio_format: None,
        })
    }

    async fn design_voice(
        &self,
        _request: TtsDesignRequest<'_>,
    ) -> Result<TtsDesignResult, TtsAdapterError> {
        Ok(TtsDesignResult {
            provider_job_id: Some("mock-job".into()),
            provider_voice_id: Some("mock-voice".into()),
            provider_model: None,
        })
    }

    fn timing_source(&self) -> TtsTimingSource {
        TtsTimingSource::ProviderTranscript
    }
}

fn mock_element_timings(
    elements: &[ind_domain::TtsSpokenElement],
    duration: f64,
) -> Vec<ProviderElementTiming> {
    if elements.is_empty() {
        return vec![ProviderElementTiming {
            element_index: 0,
            start_timestamp: 0.0,
            end_timestamp: Some(duration),
        }];
    }

    let total_chars = elements
        .iter()
        .map(|element| element.text.chars().count())
        .sum::<usize>()
        .max(1) as f64;
    let mut prefix_chars = 0usize;
    elements
        .iter()
        .map(|element| {
            let start = duration * (prefix_chars as f64 / total_chars);
            prefix_chars += element.text.chars().count();
            let end = duration * (prefix_chars as f64 / total_chars);
            ProviderElementTiming {
                element_index: element.element_index,
                start_timestamp: start,
                end_timestamp: Some(end),
            }
        })
        .collect()
}
