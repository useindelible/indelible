#![allow(clippy::unwrap_used)]
use async_trait::async_trait;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use ind_ai::tts::MockTtsAdapter;
use ind_application::ports::{
    TtsAdapter, TtsAdapterError, TtsDesignRequest, TtsDesignResult, TtsSynthesisRequest,
    TtsSynthesisResult,
};
use ind_application::repos::tts_chunk::TtsChunkRepository;
use ind_application::repos::tts_voice_persona::TtsVoicePersonaRepository;
use ind_application::services::tts::entitlements::{Deployment, TtsEntitlements};
use ind_application::services::tts::synthesis::{
    SynthesisService, SynthesizeChunkInput, SynthesizeChunkOutcome, TTS_MANAGED_CHARS_QUOTA,
    TTS_MANAGED_COST_UNITS_QUOTA, TTS_MANAGED_SECONDS_QUOTA, TtsAdapterResolver, TtsManagedLimits,
};
use ind_application::{AppError, storage::ObjectStorage};
use ind_domain::{
    AudioFormat, Document, TtsChunk, TtsChunkRecordId, TtsElementKind, TtsPersonaStatus,
    TtsProvider, TtsSpokenElement, TtsVoicePersona, TtsVoicePersonaId, User, UserId,
};
use ind_persistence::repos::{
    PgBillingUsageEventRepository, PgTtsAudioAssetRepository, PgTtsChunkRepository,
    PgTtsElementTimingRepository, PgTtsVoicePersonaRepository, PgUsageCounterRepository,
};
use ind_test_support::{DocumentFactory, TestDb, UserFactory};
use std::sync::Arc;
struct FailingReadyChunks(PgTtsChunkRepository);
#[async_trait]
impl TtsChunkRepository for FailingReadyChunks {
    async fn get_by_cache_key(
        &self,
        user: UserId,
        key: &str,
    ) -> Result<Option<TtsChunk>, AppError> {
        self.0.get_by_cache_key(user, key).await
    }
    async fn get(&self, user: UserId, id: TtsChunkRecordId) -> Result<Option<TtsChunk>, AppError> {
        self.0.get(user, id).await
    }
    async fn insert(&self, chunk: &TtsChunk) -> Result<TtsChunk, AppError> {
        self.0.insert(chunk).await
    }
    async fn mark_ready(
        &self,
        _user: UserId,
        _id: TtsChunkRecordId,
        _duration: Option<f64>,
        _updated_at: DateTime<Utc>,
    ) -> Result<TtsChunk, AppError> {
        Err(AppError::Repository("forced mark-ready failure".into()))
    }
    async fn delete(&self, user: UserId, id: TtsChunkRecordId) -> Result<(), AppError> {
        self.0.delete(user, id).await
    }
}
struct EmptyAudioAdapter;
#[async_trait]
impl TtsAdapter for EmptyAudioAdapter {
    fn provider(&self) -> TtsProvider {
        TtsProvider::Mock
    }
    async fn synthesize(
        &self,
        request: TtsSynthesisRequest<'_>,
    ) -> Result<TtsSynthesisResult, TtsAdapterError> {
        let mut result = MockTtsAdapter::new().synthesize(request).await?;
        result.audio = Bytes::new();
        Ok(result)
    }
    async fn design_voice(
        &self,
        request: TtsDesignRequest<'_>,
    ) -> Result<TtsDesignResult, TtsAdapterError> {
        MockTtsAdapter::new().design_voice(request).await
    }
}
async fn subject(db: &TestDb) -> (User, Document, TtsVoicePersona) {
    let user = UserFactory::new().insert(db.pool()).await;
    let document = DocumentFactory::new(user.id).insert(db.pool()).await;
    let now = Utc::now();
    let persona = PgTtsVoicePersonaRepository::new(db.pool().clone())
        .insert(&TtsVoicePersona {
            id: TtsVoicePersonaId::new(),
            user_id: Some(user.id),
            display_name: "Journey narrator".into(),
            description: None,
            provider: TtsProvider::Mock,
            provider_voice_id: Some("mock-voice".into()),
            provider_model: None,
            design_prompt: None,
            style_prompt: None,
            pace: None,
            energy: None,
            warmth: None,
            formality: None,
            pronunciation_prefs: serde_json::json!({}),
            status: TtsPersonaStatus::Active,
            is_builtin: false,
            prompt_hash: "journey".into(),
            created_at: now,
            updated_at: now,
        })
        .await
        .unwrap();
    (user, document, persona)
}
async fn service(
    db: &TestDb,
    chunks: Arc<dyn TtsChunkRepository>,
    adapter: Arc<dyn TtsAdapter>,
    limits: TtsManagedLimits,
) -> (SynthesisService, Arc<dyn ObjectStorage>) {
    let pool = db.pool().clone();
    let storage = db.storage().await;
    let resolver: TtsAdapterResolver =
        Arc::new(move |provider| (provider == TtsProvider::Mock).then(|| adapter.clone()));
    (
        SynthesisService::new(
            chunks,
            Arc::new(PgTtsAudioAssetRepository::new(pool.clone())),
            Arc::new(PgTtsElementTimingRepository::new(pool.clone())),
            Arc::new(PgBillingUsageEventRepository::new(pool.clone())),
            Arc::new(PgUsageCounterRepository::new(pool)),
            storage.clone(),
            resolver,
            Arc::new(TtsEntitlements::new(Deployment::SelfHosted)),
            limits,
        ),
        storage,
    )
}
fn input<'a>(
    user: UserId,
    document: &Document,
    persona: &'a TtsVoicePersona,
    elements: &'a [TtsSpokenElement],
    text: &'a str,
) -> SynthesizeChunkInput<'a> {
    SynthesizeChunkInput {
        user_id: user,
        document_id: document.id,
        chunk_id: "chunk-1",
        text,
        normalized_text: text,
        elements,
        start_element_index: 0,
        end_element_index: 0,
        persona,
        provider_model: None,
        provider_voice_id: Some("mock-voice"),
        pitch: 1.0,
        audio_format: AudioFormat::Mp3,
        sample_rate: 24_000,
        pronunciation_version: 1,
        chunking_version: 1,
        managed_character_limit: None,
        api_key: None,
        api_base: None,
    }
}
fn elements(text: &str) -> Vec<TtsSpokenElement> {
    vec![TtsSpokenElement {
        element_index: 0,
        kind: TtsElementKind::Paragraph,
        text: text.into(),
        char_start: 0,
        char_end: text.chars().count() as i32,
        chunk_id: "chunk-1".into(),
    }]
}
async fn synthesize(
    service: &SynthesisService,
    user: UserId,
    document: &Document,
    persona: &TtsVoicePersona,
    text: &str,
) -> Result<SynthesizeChunkOutcome, AppError> {
    let spoken = elements(text);
    service
        .synthesize_chunk(input(user, document, persona, &spoken, text))
        .await
}
async fn durable_counts(db: &TestDb, user: UserId) -> (i64, i64, i64, i64) {
    sqlx::query_as(
        "SELECT \
         (SELECT count(*) FROM tts_chunks WHERE user_id = $1), \
         (SELECT count(*) FROM tts_audio_assets WHERE user_id = $1), \
         (SELECT count(*) FROM billing_usage_events WHERE user_id = $1), \
         COALESCE((SELECT sum(current_value)::bigint FROM usage_counters WHERE user_id = $1), 0)",
    )
    .bind(user.into_uuid())
    .fetch_one(db.pool())
    .await
    .unwrap()
}
#[tokio::test]
async fn real_synthesis_persists_timings_scopes_cache_and_repairs_zero_byte_assets() {
    let db = TestDb::new().await;
    let (user, document, persona) = subject(&db).await;
    let (svc, storage) = service(
        &db,
        Arc::new(PgTtsChunkRepository::new(db.pool().clone())),
        Arc::new(MockTtsAdapter::new()),
        TtsManagedLimits::default(),
    )
    .await;
    let first = synthesize(&svc, user.id, &document, &persona, "real synthesis")
        .await
        .unwrap();
    assert!(!first.cache_hit);
    assert_eq!(
        svc.get_timing(first.chunk.id, 0)
            .await
            .unwrap()
            .unwrap()
            .element_index,
        0
    );
    assert!(
        synthesize(&svc, user.id, &document, &persona, "real synthesis")
            .await
            .unwrap()
            .cache_hit
    );
    let (other, other_document, _) = subject(&db).await;
    assert!(
        !synthesize(&svc, other.id, &other_document, &persona, "real synthesis")
            .await
            .unwrap()
            .cache_hit
    );
    sqlx::query("UPDATE tts_audio_assets SET size_bytes = 0 WHERE id = $1")
        .bind(first.audio_asset.id.into_uuid())
        .execute(db.pool())
        .await
        .unwrap();
    storage
        .upload(&first.audio_asset.s3_key, "audio/mpeg", Bytes::new())
        .await
        .unwrap();
    let repaired = synthesize(&svc, user.id, &document, &persona, "real synthesis")
        .await
        .unwrap();
    assert!(!repaired.cache_hit);
    assert_ne!(repaired.chunk.id, first.chunk.id);
    assert!(repaired.audio_asset.size_bytes > 0);
}
#[tokio::test]
async fn every_real_quota_axis_and_empty_audio_roll_back_all_durable_state() {
    for (quota, limits) in [
        (
            TTS_MANAGED_CHARS_QUOTA,
            TtsManagedLimits {
                monthly_characters: 0,
                ..Default::default()
            },
        ),
        (
            TTS_MANAGED_SECONDS_QUOTA,
            TtsManagedLimits {
                monthly_seconds: 0,
                ..Default::default()
            },
        ),
        (
            TTS_MANAGED_COST_UNITS_QUOTA,
            TtsManagedLimits {
                monthly_cost_units: 0,
                ..Default::default()
            },
        ),
    ] {
        let db = TestDb::new().await;
        let (user, document, persona) = subject(&db).await;
        let (svc, _) = service(
            &db,
            Arc::new(PgTtsChunkRepository::new(db.pool().clone())),
            Arc::new(MockTtsAdapter::new()),
            limits,
        )
        .await;
        assert!(matches!(
            synthesize(&svc, user.id, &document, &persona, "quota boundary").await,
            Err(AppError::QuotaExceeded { quota: actual }) if actual == quota
        ));
        assert_eq!(durable_counts(&db, user.id).await, (0, 0, 0, 0));
    }
    let db = TestDb::new().await;
    let (user, document, persona) = subject(&db).await;
    let (svc, storage) = service(
        &db,
        Arc::new(PgTtsChunkRepository::new(db.pool().clone())),
        Arc::new(EmptyAudioAdapter),
        TtsManagedLimits::default(),
    )
    .await;
    assert!(matches!(
        synthesize(&svc, user.id, &document, &persona, "empty provider").await,
        Err(AppError::ExternalService { service, .. }) if service == "tts"
    ));
    assert_eq!(durable_counts(&db, user.id).await, (0, 0, 0, 0));
    assert!(
        storage
            .list_keys(&format!("tts/{}/", user.id))
            .await
            .unwrap()
            .is_empty()
    );
}
#[tokio::test]
async fn mark_ready_failure_rolls_back_real_rows_quota_storage_and_billing() {
    let db = TestDb::new().await;
    let (user, document, persona) = subject(&db).await;
    let (svc, storage) = service(
        &db,
        Arc::new(FailingReadyChunks(PgTtsChunkRepository::new(
            db.pool().clone(),
        ))),
        Arc::new(MockTtsAdapter::new()),
        TtsManagedLimits::default(),
    )
    .await;
    assert!(matches!(
        synthesize(&svc, user.id, &document, &persona, "mark ready failure").await,
        Err(AppError::Repository(_))
    ));
    assert_eq!(durable_counts(&db, user.id).await, (0, 0, 0, 0));
    assert!(
        storage
            .list_keys(&format!("tts/{}/", user.id))
            .await
            .unwrap()
            .is_empty()
    );
}
