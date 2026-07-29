use std::sync::Arc;

use chrono::Utc;
use ind_domain::{
    DomainError, TtsAudioAsset, TtsAudioAssetId, TtsChunk, TtsChunkRecordId, TtsChunkStatus,
    TtsElementTiming, TtsProvider, TtsSpokenElement, TtsTimingSource,
};
use uuid::Uuid;

use super::hash::normalized_hash;
use super::timings::build_element_timings;
use super::types::{
    SynthesizeChunkInput, SynthesizeChunkOutcome, TtsAdapterResolver, TtsManagedLimits,
};
use crate::AppError;
use crate::ports::TtsSynthesisRequest;
use crate::repos::billing_usage_event::BillingUsageEventRepository;
use crate::repos::tts_audio_asset::TtsAudioAssetRepository;
use crate::repos::tts_chunk::TtsChunkRepository;
use crate::repos::tts_element_timing::TtsElementTimingRepository;
use crate::repos::usage_counter::UsageCounterRepository;
use crate::services::tts::adapter_error;
use crate::services::tts::cache_key::TtsCacheKeyInput;
use crate::services::tts::entitlements::TtsEntitlements;
use crate::storage::{ByteRange, ObjectStorage, RangedObjectData};

pub struct SynthesisService {
    pub(super) chunks: Arc<dyn TtsChunkRepository>,
    pub(super) audio_assets: Arc<dyn TtsAudioAssetRepository>,
    pub(super) element_timings: Arc<dyn TtsElementTimingRepository>,
    pub(super) billing_usage_events: Arc<dyn BillingUsageEventRepository>,
    pub(super) usage_counters: Arc<dyn UsageCounterRepository>,
    pub(super) storage: Arc<dyn ObjectStorage>,
    pub(super) adapters: TtsAdapterResolver,
    pub(super) entitlements: Arc<TtsEntitlements>,
    pub(super) managed_limits: TtsManagedLimits,
}

impl SynthesisService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        chunks: Arc<dyn TtsChunkRepository>,
        audio_assets: Arc<dyn TtsAudioAssetRepository>,
        element_timings: Arc<dyn TtsElementTimingRepository>,
        billing_usage_events: Arc<dyn BillingUsageEventRepository>,
        usage_counters: Arc<dyn UsageCounterRepository>,
        storage: Arc<dyn ObjectStorage>,
        adapters: TtsAdapterResolver,
        entitlements: Arc<TtsEntitlements>,
        managed_limits: TtsManagedLimits,
    ) -> Self {
        Self {
            chunks,
            audio_assets,
            element_timings,
            billing_usage_events,
            usage_counters,
            storage,
            adapters,
            entitlements,
            managed_limits,
        }
    }

    pub fn provider_timing_source(&self, provider: TtsProvider) -> TtsTimingSource {
        (self.adapters)(provider)
            .map(|adapter| adapter.timing_source())
            .unwrap_or(TtsTimingSource::Heuristic)
    }

    pub async fn get_chunk(
        &self,
        user_id: ind_domain::UserId,
        chunk_record_id: TtsChunkRecordId,
    ) -> Result<Option<TtsChunk>, AppError> {
        self.chunks.get(user_id, chunk_record_id).await
    }

    pub async fn get_timing(
        &self,
        chunk_record_id: TtsChunkRecordId,
        element_index: i32,
    ) -> Result<Option<TtsElementTiming>, AppError> {
        self.element_timings
            .get_by_element(chunk_record_id, element_index)
            .await
    }

    pub async fn get_audio(
        &self,
        user_id: ind_domain::UserId,
        chunk_record_id: TtsChunkRecordId,
        range: Option<ByteRange>,
    ) -> Result<RangedObjectData, AppError> {
        let asset = self
            .audio_assets
            .get_by_chunk_record(user_id, chunk_record_id)
            .await?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "TtsAudioAsset",
                    id: chunk_record_id.to_string(),
                })
            })?;

        self.storage.get_object_range(&asset.s3_key, range).await
    }

    pub async fn synthesize_chunk(
        &self,
        input: SynthesizeChunkInput<'_>,
    ) -> Result<SynthesizeChunkOutcome, AppError> {
        self.entitlements
            .authorize_synthesis(input.persona.provider)?;
        let timing_source = self.provider_timing_source(input.persona.provider);

        let cache_key = TtsCacheKeyInput {
            normalized_text: input.normalized_text,
            provider: input.persona.provider,
            provider_model: input.provider_model,
            voice_persona_id: Some(input.persona.id),
            provider_voice_id: input.provider_voice_id,
            prompt_hash: &input.persona.prompt_hash,
            pitch: input.pitch,
            audio_format: input.audio_format,
            sample_rate: input.sample_rate,
            pronunciation_version: input.pronunciation_version,
            chunking_version: input.chunking_version,
            timing_source,
        }
        .hash();

        let mut existing_chunk = self
            .chunks
            .get_by_cache_key(input.user_id, &cache_key)
            .await?;
        if let Some(chunk) = existing_chunk.as_ref()
            && chunk.status == TtsChunkStatus::Ready
            && let Some(asset) = self
                .audio_assets
                .get_by_chunk_record(input.user_id, chunk.id)
                .await?
        {
            if asset.size_bytes <= 0 {
                let _ = self
                    .audio_assets
                    .delete_by_chunk_record(input.user_id, chunk.id)
                    .await;
                let _ = self.storage.delete(&asset.s3_key).await;
                let _ = self.chunks.delete(input.user_id, chunk.id).await;
                existing_chunk = None;
            } else {
                let cached_timings = self.cached_timings(chunk.id, input.elements).await?;
                let provider_timings = cached_timings
                    .iter()
                    .map(|timing| ind_domain::ProviderElementTiming {
                        element_index: timing.element_index,
                        start_timestamp: timing.start_timestamp,
                        end_timestamp: timing.end_timestamp,
                    })
                    .collect::<Vec<_>>();
                let element_timings = build_element_timings(
                    chunk.id,
                    input.elements,
                    chunk.duration_seconds.unwrap_or_else(|| {
                        input.normalized_text.chars().count().max(1) as f64 / 15.0
                    }),
                    &provider_timings,
                    timing_source,
                )
                .map_err(|error| AppError::ExternalService {
                    service: "tts".into(),
                    message: error.to_string(),
                })?;
                let audio_format = chunk.audio_format;
                return Ok(SynthesizeChunkOutcome {
                    chunk: chunk.clone(),
                    audio_asset: asset,
                    element_timings,
                    timing_source,
                    cache_hit: true,
                    usage: None,
                    audio_format,
                });
            }
        }

        let mut quota_reservation = Some(
            self.reserve_managed_quota(
                input.user_id,
                input.normalized_text,
                input.managed_character_limit,
            )
            .await?,
        );

        let adapter = (self.adapters)(input.persona.provider).ok_or_else(|| {
            AppError::Domain(DomainError::Validation {
                field: "provider".into(),
                message: format!(
                    "no adapter registered for provider {}",
                    input.persona.provider.as_str()
                ),
            })
        })?;

        let synthesis = adapter
            .synthesize(TtsSynthesisRequest {
                persona: input.persona,
                provider_model: input.provider_model,
                provider_voice_id: input.provider_voice_id,
                text: input.text,
                normalized_text: input.normalized_text,
                elements: input.elements,
                pitch: input.pitch,
                audio_format: input.audio_format,
                sample_rate: input.sample_rate,
                api_key: input.api_key,
                api_base: input.api_base,
            })
            .await
            .map_err(adapter_error);

        let synthesis = match synthesis {
            Ok(synthesis) => synthesis,
            Err(err) => {
                if let Some(reservation) = quota_reservation.as_ref() {
                    self.release_managed_quota(input.user_id, reservation)
                        .await?;
                }
                return Err(err);
            }
        };

        if synthesis.audio.is_empty() {
            if let Some(reservation) = quota_reservation.as_ref() {
                self.release_managed_quota(input.user_id, reservation)
                    .await?;
            }
            return Err(AppError::ExternalService {
                service: "tts".into(),
                message: "provider returned empty audio body".into(),
            });
        }

        let chunk_id = existing_chunk
            .as_ref()
            .map(|chunk| chunk.id)
            .unwrap_or_else(|| TtsChunkRecordId::from_uuid(Uuid::now_v7()));
        let timings = match build_element_timings(
            chunk_id,
            input.elements,
            synthesis
                .duration_seconds
                .unwrap_or_else(|| input.normalized_text.chars().count().max(1) as f64 / 15.0),
            &synthesis.element_timings,
            timing_source,
        ) {
            Ok(timings) => timings,
            Err(error) => {
                if let Some(reservation) = quota_reservation.as_ref() {
                    self.release_managed_quota(input.user_id, reservation)
                        .await?;
                }
                return Err(AppError::ExternalService {
                    service: "tts".into(),
                    message: error.to_string(),
                });
            }
        };

        if let Some(reservation) = quota_reservation.take() {
            match self
                .ensure_actual_usage_reserved(input.user_id, reservation, &synthesis.usage)
                .await
            {
                Ok(updated) => quota_reservation = Some(updated),
                Err(err) => return Err(err),
            }
        }

        let now = Utc::now();
        let effective_format = synthesis.audio_format.unwrap_or(input.audio_format);
        let s3_key = format!(
            "tts/{}/{}.{}",
            input.user_id,
            chunk_id,
            effective_format.as_str()
        );
        let upload = self
            .storage
            .upload(&s3_key, &synthesis.content_type, synthesis.audio.clone())
            .await;
        let upload = match upload {
            Ok(upload) => upload,
            Err(err) => {
                if let Some(reservation) = quota_reservation.as_ref() {
                    self.release_managed_quota(input.user_id, reservation)
                        .await?;
                }
                return Err(err);
            }
        };

        let new_chunk = TtsChunk {
            id: chunk_id,
            user_id: input.user_id,
            document_id: input.document_id,
            chunk_id: input.chunk_id.to_string(),
            cache_key,
            voice_persona_id: Some(input.persona.id),
            provider: input.persona.provider,
            provider_model: input.provider_model.map(str::to_string),
            provider_voice_id: input.provider_voice_id.map(str::to_string),
            pitch: input.pitch,
            audio_format: effective_format,
            sample_rate: input.sample_rate,
            pronunciation_version: input.pronunciation_version,
            chunking_version: input.chunking_version,
            normalized_text_hash: normalized_hash(input.normalized_text),
            start_element_index: input.start_element_index,
            end_element_index: input.end_element_index,
            duration_seconds: synthesis.duration_seconds,
            status: TtsChunkStatus::Pending,
            created_at: now,
            updated_at: now,
        };
        let chunk = match existing_chunk {
            Some(_) => self
                .chunks
                .get(input.user_id, chunk_id)
                .await?
                .ok_or_else(|| {
                    AppError::Domain(DomainError::NotFound {
                        entity: "TtsChunkRecord",
                        id: chunk_id.to_string(),
                    })
                }),
            None => self.chunks.insert(&new_chunk).await,
        };
        let chunk = match chunk {
            Ok(chunk) => chunk,
            Err(err) => {
                let _ = self.storage.delete(&s3_key).await;
                if let Some(reservation) = quota_reservation.as_ref() {
                    self.release_managed_quota(input.user_id, reservation)
                        .await?;
                }
                return Err(err);
            }
        };

        let asset = TtsAudioAsset {
            id: TtsAudioAssetId::from_uuid(Uuid::now_v7()),
            user_id: input.user_id,
            chunk_record_id: chunk.id,
            s3_key: upload.key,
            content_type: synthesis.content_type.clone(),
            size_bytes: upload.size_bytes,
            created_at: now,
        };
        let asset = match self.audio_assets.insert(&asset).await {
            Ok(asset) => asset,
            Err(err) => {
                let _ = self.storage.delete(&s3_key).await;
                if let Some(reservation) = quota_reservation.as_ref() {
                    self.release_managed_quota(input.user_id, reservation)
                        .await?;
                }
                return Err(err);
            }
        };

        if !timings.is_empty()
            && let Err(err) = self.element_timings.insert_batch(&timings).await
        {
            let _ = self
                .audio_assets
                .delete_by_chunk_record(input.user_id, chunk.id)
                .await;
            let _ = self.storage.delete(&s3_key).await;
            if let Some(reservation) = quota_reservation.as_ref() {
                self.release_managed_quota(input.user_id, reservation)
                    .await?;
            }
            return Err(err);
        }

        // Over-reservation must be released before the chunk is marked ready
        // so the usage counters agree with the about-to-be-emitted billing
        // event. If this fails we still roll back asset + storage + quota so
        // no orphaned counter remains. Pass `&mut` so the reservation tracks
        // the actual usage, keeping a later `release_managed_quota` honest on
        // the mark_ready failure path.
        if let Some(reservation) = quota_reservation.as_mut()
            && let Err(err) = self
                .release_over_reserved_quota(input.user_id, reservation, &synthesis.usage)
                .await
        {
            let _ = self
                .audio_assets
                .delete_by_chunk_record(input.user_id, chunk.id)
                .await;
            let _ = self.chunks.delete(input.user_id, chunk.id).await;
            let _ = self.storage.delete(&s3_key).await;
            if let Some(reservation) = quota_reservation.as_ref() {
                self.release_managed_quota(input.user_id, reservation)
                    .await?;
            }
            return Err(err);
        }

        // Mark the chunk ready BEFORE committing the billing event. If
        // `mark_ready` fails, the chunk row, asset, and quota are rolled back
        // and no billing event exists; on retry a brand-new chunk_id is
        // generated and the idempotency key naturally produces a single
        // charge. The previous ordering (`record_usage` first) could leave a
        // billing event alive while the chunk was torn down and then
        // re-generated — a second managed-billing event on retry would
        // double-charge the user for the same logical synthesis.
        let chunk = match self
            .chunks
            .mark_ready(
                input.user_id,
                chunk.id,
                synthesis.duration_seconds,
                Utc::now(),
            )
            .await
        {
            Ok(chunk) => chunk,
            Err(err) => {
                let _ = self
                    .audio_assets
                    .delete_by_chunk_record(input.user_id, chunk.id)
                    .await;
                let _ = self.chunks.delete(input.user_id, chunk.id).await;
                let _ = self.storage.delete(&s3_key).await;
                if let Some(reservation) = quota_reservation.as_ref() {
                    self.release_managed_quota(input.user_id, reservation)
                        .await?;
                }
                return Err(err);
            }
        };

        // With the chunk already visible, record usage. A failure here must
        // roll back both the chunk and its asset (the bookkeeping and the
        // data must stay consistent) and release the quota reservation so a
        // retry sees a clean slate.
        let usage = match self
            .record_usage(
                input.user_id,
                input.persona.provider,
                chunk.id,
                &synthesis.usage,
            )
            .await
        {
            Ok(usage) => usage,
            Err(err) => {
                let _ = self.chunks.delete(input.user_id, chunk.id).await;
                let _ = self
                    .audio_assets
                    .delete_by_chunk_record(input.user_id, chunk.id)
                    .await;
                let _ = self.storage.delete(&s3_key).await;
                if let Some(reservation) = quota_reservation.as_ref() {
                    self.release_managed_quota(input.user_id, reservation)
                        .await?;
                }
                return Err(err);
            }
        };

        Ok(SynthesizeChunkOutcome {
            chunk,
            audio_asset: asset,
            element_timings: timings,
            timing_source,
            cache_hit: false,
            usage,
            audio_format: effective_format,
        })
    }

    async fn cached_timings(
        &self,
        chunk_record_id: TtsChunkRecordId,
        elements: &[TtsSpokenElement],
    ) -> Result<Vec<TtsElementTiming>, AppError> {
        let mut timings = Vec::with_capacity(elements.len());
        for element in elements {
            if let Some(timing) = self
                .element_timings
                .get_by_element(chunk_record_id, element.element_index)
                .await?
            {
                timings.push(timing);
            }
        }
        Ok(timings)
    }
}
