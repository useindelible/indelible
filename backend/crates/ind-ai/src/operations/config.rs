use super::helpers::*;
use super::*;

const IMMEDIATE_REPAIR_BATCH_SIZE: i64 = 10;

fn embedding_target(
    config: &MilaConfig,
) -> ind_application::repos::embedding_backfill::EffectiveEmbeddingTarget {
    ind_application::repos::embedding_backfill::EffectiveEmbeddingTarget {
        embedding_model: config.embedding_model.clone(),
        embedding_dim: config.embedding_dim,
    }
}

impl MilaConfigPort for MilaOperationsService {
    fn get_config(&self, user_id: UserId) -> BoxFuture<'_, Result<MilaConfigOutput, AppError>> {
        Box::pin(async move {
            let config = self.service.get_config(user_id).await?.ok_or_else(|| {
                AppError::Domain(ind_domain::DomainError::InvariantViolation {
                    message: format!("missing effective Mila config for user {user_id}"),
                })
            })?;
            Ok(mila_config_output(config))
        })
    }

    fn get_status(&self, user_id: UserId) -> BoxFuture<'_, Result<MilaStatusOutput, AppError>> {
        Box::pin(async move {
            // Staleness is measured against the provider Mila actually uses, which is
            // the managed default when BYO is off — otherwise toggling off would leave
            // reindex_required false while queries embed with a different model.
            let config = self
                .service
                .get_config(user_id)
                .await?
                .map(|config| config.resolve_effective(self.service.platform_defaults()))
                .ok_or_else(|| {
                    AppError::Domain(ind_domain::DomainError::InvariantViolation {
                        message: format!("missing effective Mila config for user {user_id}"),
                    })
                })?;

            let eligible_items = self
                .embedding_backfill_repo
                .count_eligible_items(user_id)
                .await?;
            let indexed_items = self
                .embedding_backfill_repo
                .count_indexed_items(user_id, &config.embedding_model, config.embedding_dim)
                .await?;
            let stale_items = self
                .embedding_backfill_repo
                .count_stale_items(user_id, &config.embedding_model, config.embedding_dim)
                .await?;
            let has_pending_jobs = if config.enabled {
                self.embedding_backfill_repo
                    .has_active_embedding_work(user_id)
                    .await?
            } else {
                false
            };

            Ok(mila_status_view(
                config.enabled,
                eligible_items,
                indexed_items,
                stale_items,
                has_pending_jobs,
            ))
        })
    }

    fn upsert_config(
        &self,
        user_id: UserId,
        request: UpdateMilaConfigRequest,
    ) -> BoxFuture<'_, Result<MilaConfigOutput, AppError>> {
        Box::pin(async move {
            let current = self.service.get_config(user_id).await?;
            let previous_target = current.as_ref().map(|config| {
                embedding_target(&config.resolve_effective(self.service.platform_defaults()))
            });
            let config = self
                .service
                .upsert_config(
                    user_id,
                    ind_application::UpsertMilaConfigInput {
                        chat_api_base: Some(request.chat_api_base),
                        chat_api_key: api_key_update(
                            request.chat_api_key,
                            request.clear_chat_api_key,
                            self.credential_cipher.as_deref(),
                        )?,
                        chat_model: Some(request.chat_model),
                        embedding_api_base: Some(request.embedding_api_base),
                        embedding_api_key: api_key_update(
                            request.embedding_api_key,
                            request.clear_embedding_api_key,
                            self.credential_cipher.as_deref(),
                        )?,
                        embedding_model: Some(request.embedding_model),
                        embedding_dim: Some(request.embedding_dim),
                        model_context_window: Some(request.model_context_window),
                        chat_context_pct: Some(request.chat_context_pct),
                        chunk_size: None,
                        chunk_overlap: None,
                        top_k: Some(request.top_k),
                        cross_item_top_k: Some(request.cross_item_top_k),
                        cross_item_max_per_item: Some(request.cross_item_max_per_item),
                        enabled: Some(request.enabled),
                        byo_enabled: request.byo_enabled,
                        supports_structured_output: request.supports_structured_output,
                        supports_reasoning_effort: request.supports_reasoning_effort,
                    },
                )
                .await?;

            let effective = config.resolve_effective(self.service.platform_defaults());

            if mila_enable_requires_backfill(current.as_ref(), config.enabled) {
                enqueue_embedding_backfill(
                    self.outbox_repo.as_ref(),
                    self.embedding_backfill_repo.as_ref(),
                    user_id,
                    &effective.embedding_model,
                    effective.embedding_dim,
                )
                .await?;
            } else if effective.enabled
                && previous_target.as_ref() != Some(&embedding_target(&effective))
            {
                self.embedding_backfill_repo
                    .enqueue_user_vector_repairs(
                        user_id,
                        &embedding_target(&effective),
                        IMMEDIATE_REPAIR_BATCH_SIZE,
                    )
                    .await?;
            }

            Ok(mila_config_output(config))
        })
    }

    fn reindex_config(
        &self,
        user_id: UserId,
        request: UpdateMilaConfigRequest,
    ) -> BoxFuture<'_, Result<MilaConfigOutput, AppError>> {
        Box::pin(async move {
            let config = self
                .service
                .upsert_config(
                    user_id,
                    ind_application::UpsertMilaConfigInput {
                        chat_api_base: Some(request.chat_api_base),
                        chat_api_key: api_key_update(
                            request.chat_api_key,
                            request.clear_chat_api_key,
                            self.credential_cipher.as_deref(),
                        )?,
                        chat_model: Some(request.chat_model),
                        embedding_api_base: Some(request.embedding_api_base),
                        embedding_api_key: api_key_update(
                            request.embedding_api_key,
                            request.clear_embedding_api_key,
                            self.credential_cipher.as_deref(),
                        )?,
                        embedding_model: Some(request.embedding_model),
                        embedding_dim: Some(request.embedding_dim),
                        model_context_window: Some(request.model_context_window),
                        chat_context_pct: Some(request.chat_context_pct),
                        chunk_size: None,
                        chunk_overlap: None,
                        top_k: Some(request.top_k),
                        cross_item_top_k: Some(request.cross_item_top_k),
                        cross_item_max_per_item: Some(request.cross_item_max_per_item),
                        enabled: Some(request.enabled),
                        byo_enabled: request.byo_enabled,
                        supports_structured_output: request.supports_structured_output,
                        supports_reasoning_effort: request.supports_reasoning_effort,
                    },
                )
                .await?;

            let effective = config.resolve_effective(self.service.platform_defaults());
            self.embedding_backfill_repo
                .retry_user_vector_repairs(
                    user_id,
                    &embedding_target(&effective),
                    IMMEDIATE_REPAIR_BATCH_SIZE,
                )
                .await?;

            Ok(mila_config_output(config))
        })
    }

    fn test_config(
        &self,
        user_id: UserId,
        request: TestMilaConfigRequest,
    ) -> BoxFuture<'_, Result<MilaProviderTestResult, AppError>> {
        Box::pin(async move {
            crate::MilaProviderTestService::new(
                self.ai_client.clone(),
                self.credential_cipher.clone(),
            )
            .test_config(user_id, request, |user_id| self.service.get_config(user_id))
            .await
        })
    }
}
