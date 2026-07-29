use std::sync::Arc;

use crate::config::ServerConfig;
use crate::services::repositories::Repositories;
use ind_application::ports::TtsOperations;
use ind_application::storage::ObjectStorage;
use ind_persistence::repos::{
    PgBillingUsageEventRepository, PgDocumentAssetRepository, PgDocumentRepository,
    PgPlaybackStateRepository, PgTtsAudioAssetRepository, PgTtsChunkRepository,
    PgTtsElementTimingRepository, PgTtsSessionRepository, PgTtsVoicePersonaRepository,
};
use secrecy::ExposeSecret;

pub(super) fn build_tts_ops(
    config: &ServerConfig,
    pool: &sqlx::PgPool,
    storage: Option<&Arc<dyn ObjectStorage>>,
    repos: &Repositories,
) -> Option<Arc<dyn TtsOperations>> {
    if !config.tts.enabled {
        tracing::info!("TTS services disabled by configuration");
        return None;
    }
    let Some(storage) = storage.cloned() else {
        tracing::warn!("TTS services disabled because object storage is not configured");
        return None;
    };

    let tts_voice_persona_repo = Arc::new(PgTtsVoicePersonaRepository::new(pool.clone()));
    let mut tts_registry = ind_ai::tts::TtsAdapterRegistry::new();
    if config.tts.use_mock_adapter {
        tts_registry = tts_registry.with(Arc::new(ind_ai::tts::MockTtsAdapter));
    }
    #[expect(
        clippy::expect_used,
        reason = "DashScope adapter builds from static config; failure is a fatal boot misconfiguration"
    )]
    let dashscope_adapter = ind_ai::tts::DashScopeAdapter::new()
        .expect("build DashScope adapter")
        .with_transcript_support(config.tts.dashscope.transcript_supported);
    tts_registry = tts_registry.with(Arc::new(dashscope_adapter));
    #[expect(
        clippy::expect_used,
        reason = "Unreal Speech adapter builds from static config; failure is a fatal boot misconfiguration"
    )]
    let unreal_adapter = ind_ai::tts::UnrealSpeechAdapter::new()
        .expect("build Unreal Speech adapter")
        .with_transcript_support(config.tts.unreal_speech.transcript_supported);
    tts_registry = tts_registry.with(Arc::new(unreal_adapter));
    let registry_for_resolver = tts_registry.clone();
    let persona_adapter_resolver: ind_application::services::tts::PersonaAdapterResolver =
        Arc::new(move |p| registry_for_resolver.get(p));
    let registry_for_synthesis = tts_registry.clone();
    let synthesis_adapter_resolver: ind_application::services::tts::synthesis::TtsAdapterResolver =
        Arc::new(move |p| registry_for_synthesis.get(p));

    let deployment = config.tts.deployment.into();
    let tts_entitlements = Arc::new(
        ind_application::services::tts::TtsEntitlements::new(deployment)
            .with_hosted_managed_custom_persona(config.tts.hosted_managed_custom_persona),
    );

    let tts_credential_resolver = build_credential_resolver(config);
    let persona_repo_for_session = tts_voice_persona_repo.clone()
        as Arc<dyn ind_application::repos::tts_voice_persona::TtsVoicePersonaRepository>;

    let tts_persona_service = Arc::new(
        ind_application::services::tts::PersonaService::new(
            tts_voice_persona_repo
                as Arc<dyn ind_application::repos::tts_voice_persona::TtsVoicePersonaRepository>,
        )
        .with_entitlements(tts_entitlements.clone())
        .with_adapters(persona_adapter_resolver)
        .with_credentials(tts_credential_resolver.clone()),
    );
    let tts_chunk_repo = Arc::new(PgTtsChunkRepository::new(pool.clone()))
        as Arc<dyn ind_application::repos::tts_chunk::TtsChunkRepository>;
    let tts_audio_asset_repo = Arc::new(PgTtsAudioAssetRepository::new(pool.clone()))
        as Arc<dyn ind_application::repos::tts_audio_asset::TtsAudioAssetRepository>;
    let tts_element_timing_repo = Arc::new(PgTtsElementTimingRepository::new(pool.clone()))
        as Arc<dyn ind_application::repos::tts_element_timing::TtsElementTimingRepository>;
    let tts_session_repo = Arc::new(PgTtsSessionRepository::new(pool.clone()))
        as Arc<dyn ind_application::repos::tts_session::TtsSessionRepository>;
    let playback_repo = Arc::new(PgPlaybackStateRepository::new(pool.clone()))
        as Arc<dyn ind_application::repos::playback_state::PlaybackStateRepository>;

    let synthesis_service = Arc::new(
        ind_application::services::tts::synthesis::SynthesisService::new(
            tts_chunk_repo,
            tts_audio_asset_repo,
            tts_element_timing_repo,
            Arc::new(PgBillingUsageEventRepository::new(pool.clone()))
                as Arc<
                    dyn ind_application::repos::billing_usage_event::BillingUsageEventRepository,
                >,
            repos.usage_counter.clone(),
            storage.clone(),
            synthesis_adapter_resolver,
            tts_entitlements.clone(),
            ind_application::services::tts::synthesis::TtsManagedLimits::default(),
        ),
    );
    let element_source = Arc::new(
        ind_application::services::tts::ReadableTtsElementSource::new(
            Arc::new(PgDocumentRepository::new(pool.clone()))
                as Arc<dyn ind_application::repos::document::DocumentRepository>,
            Arc::new(PgDocumentAssetRepository::new(pool.clone()))
                as Arc<dyn ind_application::repos::document_asset::DocumentAssetRepository>,
            storage,
            Arc::new(ind_ingest::ScraperHtmlExtractor),
        ),
    ) as Arc<dyn ind_domain::TtsElementSource>;
    let session_service = Arc::new(ind_application::services::tts::TtsSessionService::new(
        persona_repo_for_session,
        tts_session_repo,
        element_source,
        synthesis_service,
        tts_credential_resolver,
        tts_entitlements,
    ));

    let tts = ind_application::services::tts::TtsOperationsService::new(
        ind_application::services::tts::TtsOperationsDeps {
            persona_service: tts_persona_service,
            session_service,
            playback_repo,
        },
    );
    tracing::info!("TTS services initialized");
    Some(Arc::new(tts) as Arc<dyn TtsOperations>)
}

fn build_credential_resolver(
    config: &ServerConfig,
) -> Arc<dyn ind_application::services::tts::TtsProviderCredentialResolver> {
    let mut resolver = ind_application::services::tts::DefaultTtsCredentialResolver::new();
    if let Some(api_key) = config.tts.dashscope.api_key.as_ref() {
        resolver = resolver.with_credential(
            ind_domain::TtsProvider::DashScope,
            ind_application::services::tts::DeploymentTtsCredential {
                api_key: api_key.expose_secret().to_string(),
                api_base: config.tts.dashscope.api_base.clone(),
            },
        );
    }
    if let Some(api_key) = config.tts.unreal_speech.api_key.as_ref() {
        resolver = resolver.with_credential(
            ind_domain::TtsProvider::UnrealSpeech,
            ind_application::services::tts::DeploymentTtsCredential {
                api_key: api_key.expose_secret().to_string(),
                api_base: config.tts.unreal_speech.api_base.clone(),
            },
        );
    }
    Arc::new(resolver)
}
