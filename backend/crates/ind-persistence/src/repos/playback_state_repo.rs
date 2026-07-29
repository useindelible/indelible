use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::playback_state::PlaybackStateRepository;
use ind_domain::{DocumentId, DomainError, PlaybackKind, PlaybackState, TtsVoicePersonaId, UserId};

pub struct PgPlaybackStateRepository {
    pool: PgPool,
}

impl PgPlaybackStateRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct PlaybackStateRow {
    user_id: Uuid,
    document_id: Uuid,
    playback_kind: String,
    position_seconds: f64,
    playback_speed: f64,
    element_index: Option<i32>,
    tts_chunk_id: Option<String>,
    tts_voice_persona_id: Option<Uuid>,
    is_playing: bool,
    updated_at: DateTime<Utc>,
}

impl TryFrom<PlaybackStateRow> for PlaybackState {
    type Error = AppError;

    fn try_from(row: PlaybackStateRow) -> Result<Self, Self::Error> {
        let kind: PlaybackKind = row.playback_kind.parse().map_err(|_| {
            AppError::Repository(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("unknown playback_kind: {}", row.playback_kind),
            )))
        })?;

        Ok(PlaybackState {
            user_id: UserId::from_uuid(row.user_id),
            document_id: DocumentId::from_uuid(row.document_id),
            playback_kind: kind,
            position_seconds: row.position_seconds,
            playback_speed: row.playback_speed,
            element_index: row.element_index,
            tts_chunk_id: row.tts_chunk_id,
            tts_voice_persona_id: row.tts_voice_persona_id.map(TtsVoicePersonaId::from_uuid),
            is_playing: row.is_playing,
            updated_at: row.updated_at,
        })
    }
}

fn map_err(err: sqlx::Error) -> AppError {
    super::map_sqlx_error("playback_state", "duplicate playback state", err)
}

#[async_trait::async_trait]
impl PlaybackStateRepository for PgPlaybackStateRepository {
    async fn upsert(&self, state: &PlaybackState) -> Result<PlaybackState, AppError> {
        let row = sqlx::query_as!(
            PlaybackStateRow,
            r#"
            INSERT INTO document_playback_states (
                user_id, document_id, playback_kind, position_seconds, playback_speed,
                element_index, tts_chunk_id, tts_voice_persona_id, is_playing
            )
            SELECT $1, $2, $3, $4, $5, $6, $7, $8, $9
            FROM documents
            WHERE id = $2 AND user_id = $1
            ON CONFLICT (user_id, document_id, playback_kind) DO UPDATE SET
                position_seconds     = EXCLUDED.position_seconds,
                playback_speed       = EXCLUDED.playback_speed,
                element_index        = EXCLUDED.element_index,
                tts_chunk_id         = EXCLUDED.tts_chunk_id,
                tts_voice_persona_id = EXCLUDED.tts_voice_persona_id,
                is_playing           = EXCLUDED.is_playing,
                updated_at           = now()
            RETURNING
                user_id, document_id, playback_kind, position_seconds, playback_speed,
                element_index, tts_chunk_id, tts_voice_persona_id, is_playing, updated_at
            "#,
            state.user_id.into_uuid(),
            state.document_id.into_uuid(),
            state.playback_kind.as_str(),
            state.position_seconds,
            state.playback_speed,
            state.element_index,
            state.tts_chunk_id.as_deref(),
            state.tts_voice_persona_id.map(|id| id.into_uuid()),
            state.is_playing,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_err)?;

        let row = row.ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "Document",
                id: state.document_id.to_string(),
            })
        })?;

        PlaybackState::try_from(row)
    }

    async fn get(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        kind: PlaybackKind,
    ) -> Result<Option<PlaybackState>, AppError> {
        let row = sqlx::query_as!(
            PlaybackStateRow,
            r#"
            SELECT user_id, document_id, playback_kind, position_seconds, playback_speed,
                   element_index, tts_chunk_id, tts_voice_persona_id, is_playing, updated_at
            FROM document_playback_states ps
            WHERE ps.user_id = $1
              AND ps.document_id = $2
              AND ps.playback_kind = $3
              AND EXISTS (
                  SELECT 1
                  FROM documents i
                  WHERE i.id = ps.document_id
                    AND i.user_id = ps.user_id
              )
            "#,
            user_id.into_uuid(),
            document_id.into_uuid(),
            kind.as_str(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_err)?;

        row.map(PlaybackState::try_from).transpose()
    }
}
