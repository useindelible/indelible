use std::sync::Arc;

use futures::future::BoxFuture;
use ind_domain::UserId;

use super::{
    CreatePersonaInput, PersonaService, StartSessionInput, TtsSessionManifest, TtsSessionService,
};
use crate::AppError;
use crate::ports::{TtsOperations, TtsResolvedChunk, UpsertPlaybackStateInput};
use crate::repos::playback_state::PlaybackStateRepository;
use crate::storage::{ByteRange, RangedObjectData};

// -- TtsOperations --

pub struct TtsOperationsService {
    persona_service: Arc<PersonaService>,
    session_service: Arc<TtsSessionService>,
    playback_repo: Arc<dyn PlaybackStateRepository>,
}

pub struct TtsOperationsDeps {
    pub persona_service: Arc<PersonaService>,
    pub session_service: Arc<TtsSessionService>,
    pub playback_repo: Arc<dyn PlaybackStateRepository>,
}

impl TtsOperationsService {
    pub fn new(deps: TtsOperationsDeps) -> Self {
        let TtsOperationsDeps {
            persona_service,
            session_service,
            playback_repo,
        } = deps;
        Self {
            persona_service,
            session_service,
            playback_repo,
        }
    }
}

impl TtsOperations for TtsOperationsService {
    fn list_personas(
        &self,
        user_id: UserId,
    ) -> BoxFuture<'_, Result<Vec<ind_domain::TtsVoicePersona>, AppError>> {
        Box::pin(self.persona_service.list(user_id))
    }

    fn create_persona(
        &self,
        user_id: UserId,
        input: CreatePersonaInput,
    ) -> BoxFuture<'_, Result<ind_domain::TtsVoicePersona, AppError>> {
        Box::pin(self.persona_service.create(user_id, input))
    }

    fn get_persona(
        &self,
        user_id: UserId,
        persona_id: ind_domain::TtsVoicePersonaId,
    ) -> BoxFuture<'_, Result<ind_domain::TtsVoicePersona, AppError>> {
        Box::pin(self.persona_service.get(user_id, persona_id))
    }

    fn start_session(
        &self,
        user_id: UserId,
        input: StartSessionInput,
    ) -> BoxFuture<'_, Result<TtsSessionManifest, AppError>> {
        Box::pin(self.session_service.start_session(user_id, input))
    }

    fn resolve_session_chunk(
        &self,
        user_id: UserId,
        document_id: ind_domain::DocumentId,
        session_id: ind_domain::TtsSessionId,
        chunk_id: String,
    ) -> BoxFuture<'_, Result<Option<TtsResolvedChunk>, AppError>> {
        Box::pin(async move {
            self.session_service
                .resolve_chunk(user_id, document_id, session_id, &chunk_id)
                .await
        })
    }

    fn resolve_element_timestamp(
        &self,
        user_id: UserId,
        document_id: ind_domain::DocumentId,
        session_id: ind_domain::TtsSessionId,
        chunk_id: String,
        element_index: i32,
    ) -> BoxFuture<'_, Result<Option<ind_domain::TtsElementTiming>, AppError>> {
        Box::pin(async move {
            self.session_service
                .resolve_element_timestamp(
                    user_id,
                    document_id,
                    session_id,
                    &chunk_id,
                    element_index,
                )
                .await
        })
    }

    fn get_session_chunk_audio(
        &self,
        user_id: UserId,
        document_id: ind_domain::DocumentId,
        session_id: ind_domain::TtsSessionId,
        chunk_id: String,
        range: Option<ByteRange>,
    ) -> BoxFuture<'_, Result<RangedObjectData, AppError>> {
        Box::pin(async move {
            self.session_service
                .get_session_chunk_audio(user_id, document_id, session_id, &chunk_id, range)
                .await
        })
    }

    fn upsert_playback_state(
        &self,
        user_id: UserId,
        document_id: ind_domain::DocumentId,
        input: UpsertPlaybackStateInput,
    ) -> BoxFuture<'_, Result<ind_domain::PlaybackState, AppError>> {
        let state = ind_domain::PlaybackState {
            user_id,
            document_id,
            playback_kind: input.playback_kind,
            position_seconds: input.position_seconds,
            playback_speed: input.playback_speed,
            element_index: input.element_index,
            tts_chunk_id: input.tts_chunk_id,
            tts_voice_persona_id: input.tts_voice_persona_id,
            is_playing: input.is_playing,
            updated_at: chrono::Utc::now(),
        };
        Box::pin(async move { self.playback_repo.upsert(&state).await })
    }

    fn get_playback_state(
        &self,
        user_id: UserId,
        document_id: ind_domain::DocumentId,
        kind: ind_domain::PlaybackKind,
    ) -> BoxFuture<'_, Result<Option<ind_domain::PlaybackState>, AppError>> {
        Box::pin(self.playback_repo.get(user_id, document_id, kind))
    }
}
