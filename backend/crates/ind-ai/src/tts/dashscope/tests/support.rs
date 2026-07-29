use super::super::*;
use chrono::Utc;
use ind_domain::{AudioFormat, TtsPersonaStatus, TtsVoicePersona, TtsVoicePersonaId};
use serde_json::json;
use uuid::Uuid;

pub(super) fn sample_persona() -> TtsVoicePersona {
    TtsVoicePersona {
        id: TtsVoicePersonaId::from_uuid(Uuid::now_v7()),
        user_id: None,
        display_name: "Test Persona".into(),
        description: None,
        provider: TtsProvider::DashScope,
        provider_voice_id: Some("Cherry".into()),
        provider_model: None,
        design_prompt: None,
        style_prompt: None,
        pace: None,
        energy: None,
        warmth: None,
        formality: None,
        pronunciation_prefs: json!({}),
        status: TtsPersonaStatus::Active,
        is_builtin: false,
        prompt_hash: "h".into(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

pub(super) fn synth_request<'a>(
    persona: &'a TtsVoicePersona,
    text: &'a str,
) -> TtsSynthesisRequest<'a> {
    TtsSynthesisRequest {
        persona,
        provider_model: None,
        provider_voice_id: None,
        text,
        normalized_text: text,
        elements: &[],
        pitch: 1.0,
        audio_format: AudioFormat::Mp3,
        sample_rate: 24000,
        api_key: Some("test-key"),
        api_base: None,
    }
}
