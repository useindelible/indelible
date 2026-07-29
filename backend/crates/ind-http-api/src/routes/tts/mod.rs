pub mod audio;
pub mod dto;

use axum::Router;
use axum::extract::{Path, Query, State};
use axum::routing::{get, patch, post};
use http::StatusCode;
use ind_application::ports::{TtsOperations, UpsertPlaybackStateInput};
use ind_application::services::tts::{CreatePersonaInput, StartSessionInput};
use ind_domain::{
    AudioFormat, DocumentId, PlaybackKind, TtsGenerationScope, TtsProvider, TtsSessionId,
};

use crate::error::{ApiError, FieldError};
use crate::extract::{Json, ValidatedJson};
use crate::middleware::AccountAccess;
use crate::response::{ApiResponse, EmptyResponse};
use crate::state::AppState;

pub use audio::stream_session_chunk_audio;
pub use dto::{
    CreateVoicePersonaBody, ElementTimestampResponse, GetPlaybackStateParams, PlannedChunkResponse,
    PlannedChunkTimingResponse, PlaybackKindDto, PlaybackStateResponse, ResolveChunkResponse,
    SessionManifestResponse, SessionResponse, SessionStartResponse, StartSessionBody,
    TtsTimingSourceDto, UpsertPlaybackStateBody, VoicePersonaListResponse, VoicePersonaResponse,
};

fn require_tts_ops(state: &AppState) -> Result<&dyn TtsOperations, ApiError> {
    state
        .tts_ops
        .as_deref()
        .ok_or(ApiError::ServiceUnavailable {
            message: "tts service not configured".into(),
        })
}

/// List the user's voice personas (including built-ins).
#[utoipa::path(
    get,
    path = "/api/v1/tts/voice-personas",
    responses(
        (status = 200, description = "Voice personas available to the user", body = VoicePersonaListResponse),
        (status = 401, description = "Authentication required"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "TTS",
)]
pub async fn list_personas(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
) -> Result<ApiResponse<VoicePersonaListResponse>, ApiError> {
    let ops = require_tts_ops(&state)?;
    let personas = ops
        .list_personas(auth_user.user_id)
        .await
        .map_err(ApiError::from)?;
    Ok(ApiResponse::new(VoicePersonaListResponse {
        personas: personas
            .into_iter()
            .map(VoicePersonaResponse::from_domain)
            .collect(),
    }))
}

/// Create a new voice persona.
#[utoipa::path(
    post,
    path = "/api/v1/tts/voice-personas",
    request_body = CreateVoicePersonaBody,
    responses(
        (status = 201, description = "Persona created", body = VoicePersonaResponse),
        (status = 422, description = "Validation error"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "TTS",
)]
pub async fn create_persona(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<CreateVoicePersonaBody>,
) -> Result<(StatusCode, Json<VoicePersonaResponse>), ApiError> {
    let ops = require_tts_ops(&state)?;

    let provider = TtsProvider::parse(&body.provider).ok_or_else(|| ApiError::BadRequest {
        message: format!("unknown provider: {}", body.provider),
    })?;
    let input = CreatePersonaInput {
        display_name: body.display_name,
        description: body.description,
        provider,
        provider_voice_id: body.provider_voice_id,
        provider_model: body.provider_model,
        design_prompt: body.design_prompt,
        style_prompt: body.style_prompt,
        pace: body.pace,
        energy: body.energy,
        warmth: body.warmth,
        formality: body.formality,
        pronunciation_prefs: body
            .pronunciation_prefs
            .unwrap_or_else(|| serde_json::json!({})),
    };

    let persona = ops
        .create_persona(auth_user.user_id, input)
        .await
        .map_err(ApiError::from)?;
    Ok((
        StatusCode::CREATED,
        Json(VoicePersonaResponse::from_domain(persona)),
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/documents/{document_id}/tts/sessions",
    request_body = StartSessionBody,
    responses(
        (status = 200, description = "TTS session manifest", body = SessionManifestResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Document not found"),
        (status = 422, description = "Validation error"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "TTS",
)]
pub async fn start_document_tts_session(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(document_id): Path<String>,
    ValidatedJson(body): ValidatedJson<StartSessionBody>,
) -> Result<ApiResponse<SessionManifestResponse>, ApiError> {
    let ops = require_tts_ops(&state)?;
    let document_id = parse_document_id(&document_id)?;
    let voice_persona_id = body
        .voice_persona_id
        .as_deref()
        .map(dto::parse_persona_id)
        .transpose()?;
    let audio_format =
        AudioFormat::parse(&body.audio_format).ok_or_else(|| ApiError::ValidationError {
            errors: vec![FieldError {
                field: "audio_format".into(),
                message: "unknown audio format".into(),
            }],
        })?;
    let generation_scope = TtsGenerationScope::parse(&body.generation_scope).ok_or_else(|| {
        ApiError::ValidationError {
            errors: vec![FieldError {
                field: "generation_scope".into(),
                message: "unknown generation scope".into(),
            }],
        }
    })?;

    let manifest = ops
        .start_session(
            auth_user.user_id,
            StartSessionInput {
                document_id,
                voice_persona_id,
                speed: body.speed,
                audio_format,
                sample_rate: body.sample_rate,
                generation_scope,
                pronunciation_version: body.pronunciation_version,
                chunking_version: body.chunking_version,
                start_element_index: body.start_element_index,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(ApiResponse::new(SessionManifestResponse::from_manifest(
        manifest,
    )))
}

#[utoipa::path(
    get,
    path = "/api/v1/documents/{document_id}/tts/chunks/{chunk_id}",
    params(
        ("document_id" = String, Path, description = "Document id"),
        ("chunk_id" = String, Path, description = "Stable chunk id"),
        ("session_id" = String, Query, description = "TTS session id"),
    ),
    responses(
        (status = 200, description = "Chunk metadata", body = ResolveChunkResponse),
        (status = 404, description = "Chunk not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "TTS",
)]
pub async fn resolve_document_tts_chunk(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path((document_id, chunk_id)): Path<(String, String)>,
    Query(params): Query<SessionQuery>,
) -> Result<ApiResponse<ResolveChunkResponse>, ApiError> {
    let ops = require_tts_ops(&state)?;
    let document_id = parse_document_id(&document_id)?;
    let session_id = parse_session_id(&params.session_id)?;
    let resolved = ops
        .resolve_session_chunk(auth_user.user_id, document_id, session_id, chunk_id.clone())
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound {
            entity: "TtsSessionChunk",
            id: chunk_id,
        })?;

    Ok(ApiResponse::new(ResolveChunkResponse::from_resolved(
        resolved,
        document_id,
        session_id,
    )))
}

#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
pub struct SessionQuery {
    pub session_id: String,
}

#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
pub struct TimestampQuery {
    pub session_id: String,
    pub chunk_id: String,
    pub element_index: i32,
}

#[utoipa::path(
    get,
    path = "/api/v1/documents/{document_id}/tts/timestamp",
    params(
        ("document_id" = String, Path, description = "Document id"),
        TimestampQuery,
    ),
    responses(
        (status = 200, description = "Element timestamp", body = ElementTimestampResponse),
        (status = 404, description = "Timestamp not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "TTS",
)]
pub async fn resolve_document_tts_timestamp(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(document_id): Path<String>,
    Query(params): Query<TimestampQuery>,
) -> Result<ApiResponse<ElementTimestampResponse>, ApiError> {
    let ops = require_tts_ops(&state)?;
    let document_id = parse_document_id(&document_id)?;
    let session_id = parse_session_id(&params.session_id)?;
    let timing = ops
        .resolve_element_timestamp(
            auth_user.user_id,
            document_id,
            session_id,
            params.chunk_id.clone(),
            params.element_index,
        )
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound {
            entity: "TtsElementTiming",
            id: format!("{}:{}", params.chunk_id, params.element_index),
        })?;

    Ok(ApiResponse::new(ElementTimestampResponse::from_domain(
        timing,
    )))
}

#[utoipa::path(
    patch,
    path = "/api/v1/documents/{document_id}/playback",
    request_body = UpsertPlaybackStateBody,
    responses(
        (status = 204, description = "Playback state persisted"),
        (status = 404, description = "Document not found"),
        (status = 422, description = "Validation error"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "TTS",
)]
pub async fn upsert_document_playback_state(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(document_id): Path<String>,
    ValidatedJson(body): ValidatedJson<UpsertPlaybackStateBody>,
) -> Result<EmptyResponse, ApiError> {
    let ops = require_tts_ops(&state)?;
    let document_id = parse_document_id(&document_id)?;
    let playback_kind = parse_playback_kind("playback_kind", &body.playback_kind)?;
    let tts_voice_persona_id = parse_optional_persona_id(body.tts_voice_persona_id.as_deref())?;
    if let Some(persona_id) = tts_voice_persona_id {
        ops.get_persona(auth_user.user_id, persona_id)
            .await
            .map_err(ApiError::from)?;
    }

    ops.upsert_playback_state(
        auth_user.user_id,
        document_id,
        UpsertPlaybackStateInput {
            playback_kind,
            position_seconds: body.position_seconds,
            playback_speed: body.playback_speed,
            element_index: body.element_index,
            tts_chunk_id: body.tts_chunk_id,
            tts_voice_persona_id,
            is_playing: body.is_playing,
        },
    )
    .await
    .map_err(ApiError::from)?;
    Ok(EmptyResponse)
}

#[utoipa::path(
    get,
    path = "/api/v1/documents/{document_id}/playback",
    params(
        ("document_id" = String, Path, description = "Document id"),
        GetPlaybackStateParams,
    ),
    responses(
        (status = 200, description = "Playback state", body = PlaybackStateResponse),
        (status = 404, description = "Document or playback state not found"),
    ),
    security(("bearer" = []), ("api_token" = [])),
    tag = "TTS",
)]
pub async fn get_document_playback_state(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(document_id): Path<String>,
    Query(params): Query<GetPlaybackStateParams>,
) -> Result<Json<PlaybackStateResponse>, ApiError> {
    let ops = require_tts_ops(&state)?;
    let document_id = parse_document_id(&document_id)?;
    let playback_kind = parse_playback_kind("kind", &params.kind)?;
    let state = ops
        .get_playback_state(auth_user.user_id, document_id, playback_kind)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound {
            entity: "PlaybackState",
            id: document_id.to_string(),
        })?;

    Ok(Json(PlaybackStateResponse::from_domain(state)))
}

fn parse_document_id(raw: &str) -> Result<DocumentId, ApiError> {
    raw.parse().map_err(|_| ApiError::NotFound {
        entity: "Document",
        id: raw.to_string(),
    })
}

fn parse_session_id(raw: &str) -> Result<TtsSessionId, ApiError> {
    raw.parse().map_err(|_| ApiError::NotFound {
        entity: "TtsSession",
        id: raw.to_string(),
    })
}

fn parse_playback_kind(field: &'static str, raw: &str) -> Result<PlaybackKind, ApiError> {
    raw.parse::<PlaybackKind>()
        .map_err(|_| ApiError::ValidationError {
            errors: vec![FieldError {
                field: field.to_string(),
                message: "must be one of: tts, audio, video".to_string(),
            }],
        })
}

fn parse_optional_persona_id(
    raw: Option<&str>,
) -> Result<Option<ind_domain::TtsVoicePersonaId>, ApiError> {
    raw.map(|value| {
        value
            .parse::<ind_domain::TtsVoicePersonaId>()
            .map_err(|_| ApiError::ValidationError {
                errors: vec![FieldError {
                    field: "tts_voice_persona_id".to_string(),
                    message: "must be a vper_ prefixed voice persona ID".to_string(),
                }],
            })
    })
    .transpose()
}

pub fn tts_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/tts/voice-personas",
            get(list_personas).post(create_persona),
        )
        .route(
            "/api/v1/assets/documents/{document_id}/tts/{session_id}/{chunk_file}",
            get(stream_session_chunk_audio),
        )
        .route(
            "/api/v1/documents/{document_id}/tts/sessions",
            post(start_document_tts_session),
        )
        .route(
            "/api/v1/documents/{document_id}/tts/chunks/{chunk_id}",
            get(resolve_document_tts_chunk),
        )
        .route(
            "/api/v1/documents/{document_id}/tts/timestamp",
            get(resolve_document_tts_timestamp),
        )
        .route(
            "/api/v1/documents/{document_id}/playback",
            patch(upsert_document_playback_state).get(get_document_playback_state),
        )
}
