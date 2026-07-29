use sha2::{Digest, Sha256};

use ind_domain::{AudioFormat, TtsProvider, TtsTimingSource, TtsVoicePersonaId};

/// Inputs that must all match for a cached TTS chunk to be reusable.
///
/// The cache key is the SHA-256 hash of these fields joined with newlines.
/// The order is fixed so that minor changes to the struct layout do not
/// invalidate the cache across deploys. New dimensions are appended so every
/// previously generated key becomes unreachable under the new contract.
pub struct TtsCacheKeyInput<'a> {
    pub normalized_text: &'a str,
    pub provider: TtsProvider,
    pub provider_model: Option<&'a str>,
    pub voice_persona_id: Option<TtsVoicePersonaId>,
    pub provider_voice_id: Option<&'a str>,
    pub prompt_hash: &'a str,
    pub pitch: f64,
    pub audio_format: AudioFormat,
    pub sample_rate: i32,
    pub pronunciation_version: i32,
    pub chunking_version: i32,
    pub timing_source: TtsTimingSource,
}

impl<'a> TtsCacheKeyInput<'a> {
    pub fn hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.normalized_text.as_bytes());
        hasher.update(b"\n");
        hasher.update(self.provider.as_str().as_bytes());
        hasher.update(b"\n");
        hasher.update(self.provider_model.unwrap_or("").as_bytes());
        hasher.update(b"\n");
        hasher.update(
            self.voice_persona_id
                .map(|id| id.to_string())
                .unwrap_or_default()
                .as_bytes(),
        );
        hasher.update(b"\n");
        hasher.update(self.provider_voice_id.unwrap_or("").as_bytes());
        hasher.update(b"\n");
        hasher.update(self.prompt_hash.as_bytes());
        hasher.update(b"\n");
        hasher.update(format_rate(self.pitch).as_bytes());
        hasher.update(b"\n");
        hasher.update(self.audio_format.as_str().as_bytes());
        hasher.update(b"\n");
        hasher.update(self.sample_rate.to_string().as_bytes());
        hasher.update(b"\n");
        hasher.update(self.pronunciation_version.to_string().as_bytes());
        hasher.update(b"\n");
        hasher.update(self.chunking_version.to_string().as_bytes());
        hasher.update(b"\n");
        hasher.update(self.timing_source.as_str().as_bytes());

        hex_encode(&hasher.finalize()[..])
    }
}

// NUMERIC(4,2) rounding is used in persistence, so hashing at two decimal
// places keeps the cache key consistent with what is stored after a DB
// round-trip; 1.005 and 1.01 must not collide.
fn format_rate(value: f64) -> String {
    format!("{:.2}", (value * 100.0).round() / 100.0)
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cache_key(timing_source: TtsTimingSource) -> String {
        TtsCacheKeyInput {
            normalized_text: "Paragraph",
            provider: TtsProvider::UnrealSpeech,
            provider_model: None,
            voice_persona_id: None,
            provider_voice_id: Some("Sierra"),
            prompt_hash: "prompt",
            pitch: 1.0,
            audio_format: AudioFormat::Mp3,
            sample_rate: 24_000,
            pronunciation_version: 1,
            chunking_version: 2,
            timing_source,
        }
        .hash()
    }

    #[test]
    fn timing_contract_is_part_of_cache_identity() {
        assert_ne!(
            cache_key(TtsTimingSource::ProviderTranscript),
            cache_key(TtsTimingSource::Heuristic)
        );
    }
}
