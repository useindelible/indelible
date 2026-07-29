use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::tts_session::TtsSessionRepository;
use ind_domain::{
    AudioFormat, DocumentId, TtsChunkRecordId, TtsGenerationScope, TtsSession, TtsSessionChunk,
    TtsSessionId, TtsVoicePersonaId, UserId,
};

pub struct PgTtsSessionRepository {
    pool: PgPool,
}

impl PgTtsSessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

struct TtsSessionRow {
    id: Uuid,
    user_id: Uuid,
    document_id: Uuid,
    voice_persona_id: Option<Uuid>,
    speed: Option<f64>,
    audio_format: String,
    generation_scope: String,
    created_at: DateTime<Utc>,
}

impl TryFrom<TtsSessionRow> for TtsSession {
    type Error = AppError;

    fn try_from(row: TtsSessionRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: TtsSessionId::from_uuid(row.id),
            user_id: UserId::from_uuid(row.user_id),
            document_id: DocumentId::from_uuid(row.document_id),
            voice_persona_id: row.voice_persona_id.map(TtsVoicePersonaId::from_uuid),
            speed: row.speed.unwrap_or(1.0),
            audio_format: AudioFormat::parse(&row.audio_format).ok_or_else(|| {
                AppError::Repository(format!("unknown audio format: {}", row.audio_format).into())
            })?,
            generation_scope: TtsGenerationScope::parse(&row.generation_scope).ok_or_else(
                || {
                    AppError::Repository(
                        format!("unknown generation scope: {}", row.generation_scope).into(),
                    )
                },
            )?,
            created_at: row.created_at,
        })
    }
}

fn map_err(err: sqlx::Error) -> AppError {
    super::map_sqlx_error("tts_session", "session already exists", err)
}

#[async_trait::async_trait]
impl TtsSessionRepository for PgTtsSessionRepository {
    async fn insert(&self, session: &TtsSession) -> Result<TtsSession, AppError> {
        let row = sqlx::query_as!(
            TtsSessionRow,
            r#"
            INSERT INTO tts_sessions (
                id, user_id, document_id, voice_persona_id, speed, audio_format,
                generation_scope, created_at
            )
            VALUES (
                $1, $2, $3, $4, $5::float8::numeric(4,2), $6, $7, $8
            )
            RETURNING
                id,
                user_id,
                document_id,
                voice_persona_id,
                speed::float8 AS speed,
                audio_format,
                generation_scope,
                created_at
            "#,
            session.id.into_uuid(),
            session.user_id.into_uuid(),
            session.document_id.into_uuid(),
            session.voice_persona_id.map(|id| id.into_uuid()),
            session.speed,
            session.audio_format.as_str(),
            session.generation_scope.as_str(),
            session.created_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_err)?;

        TtsSession::try_from(row)
    }

    async fn insert_session_chunks(
        &self,
        session_id: TtsSessionId,
        chunks: &[TtsSessionChunk],
    ) -> Result<(), AppError> {
        if chunks.is_empty() {
            return Ok(());
        }

        let session_ids: Vec<Uuid> = chunks.iter().map(|_| session_id.into_uuid()).collect();
        let chunk_ids: Vec<String> = chunks.iter().map(|c| c.chunk_id.clone()).collect();
        let record_ids: Vec<Uuid> = chunks
            .iter()
            .map(|c| c.chunk_record_id.into_uuid())
            .collect();
        let positions: Vec<i32> = chunks.iter().map(|c| c.position).collect();

        sqlx::query!(
            r#"
            INSERT INTO tts_session_chunks (
                session_id, chunk_id, chunk_record_id, position
            )
            SELECT * FROM UNNEST($1::uuid[], $2::text[], $3::uuid[], $4::int4[])
            ON CONFLICT (session_id, chunk_id) DO NOTHING
            "#,
            &session_ids,
            &chunk_ids,
            &record_ids,
            &positions,
        )
        .execute(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(())
    }

    async fn resolve_chunk(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        session_id: TtsSessionId,
        chunk_id: &str,
    ) -> Result<Option<TtsSessionChunk>, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT sc.session_id, sc.chunk_id, sc.chunk_record_id, sc.position
            FROM tts_session_chunks sc
            INNER JOIN tts_sessions s ON s.id = sc.session_id
            WHERE sc.session_id = $1
              AND sc.chunk_id = $2
              AND s.user_id = $3
              AND s.document_id = $4
            "#,
            session_id.into_uuid(),
            chunk_id,
            user_id.into_uuid(),
            document_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(row.map(|r| TtsSessionChunk {
            session_id: TtsSessionId::from_uuid(r.session_id),
            chunk_id: r.chunk_id,
            chunk_record_id: TtsChunkRecordId::from_uuid(r.chunk_record_id),
            position: r.position,
        }))
    }
}
