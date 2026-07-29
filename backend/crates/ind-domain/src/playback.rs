use crate::{DocumentId, TtsVoicePersonaId, UserId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackKind {
    Tts,
    Audio,
    Video,
}

impl PlaybackKind {
    pub fn as_str(self) -> &'static str {
        match self {
            PlaybackKind::Tts => "tts",
            PlaybackKind::Audio => "audio",
            PlaybackKind::Video => "video",
        }
    }
}

impl std::str::FromStr for PlaybackKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tts" => Ok(PlaybackKind::Tts),
            "audio" => Ok(PlaybackKind::Audio),
            "video" => Ok(PlaybackKind::Video),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlaybackState {
    pub user_id: UserId,
    pub document_id: DocumentId,
    pub playback_kind: PlaybackKind,
    pub position_seconds: f64,
    pub playback_speed: f64,
    pub element_index: Option<i32>,
    pub tts_chunk_id: Option<String>,
    pub tts_voice_persona_id: Option<TtsVoicePersonaId>,
    pub is_playing: bool,
    pub updated_at: DateTime<Utc>,
}
