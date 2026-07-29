use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use ind_application::AppError;
use ind_application::repos::entity::EntityRepository;
use ind_auth::CredentialCipher;
use ind_domain::{Entity, EntityId, ExtractedEntity, MilaConfig, UserId};

use crate::actions::first_choice_content;
use crate::content::{chat_provider_from_config, map_ai_error};
use crate::{AiProviderClient, ChatCompletionRequest, ChatMessage};

use super::{
    AdjudicationItem, BatchVerdict, ENTITY_RESOLUTION_SYSTEM_PROMPT,
    batch_resolution_response_format, build_batch_resolution_prompt, parse_batch_resolution,
};

const CANDIDATE_LIMIT: i64 = 5;

/// Minimum model confidence to merge an extracted entity into an existing one.
const MATCH_MIN_CONFIDENCE: f32 = 0.85;

/// Resolves each extracted entity to a real entity before it is persisted: exact entity/alias hits
/// resolve for free; lexically-similar candidates (plus those matching a model-emitted alias) are
/// adjudicated in a single batched model call per document. A duplicate is never written. If the
/// adjudication call fails, ambiguous entities are created as new (fail-open) rather than losing the
/// document's entities.
pub(crate) struct EntityResolver {
    entity_repo: Arc<dyn EntityResolutionStore>,
    ai_client: Arc<dyn AiProviderClient>,
    credential_cipher: Option<Arc<CredentialCipher>>,
}

#[async_trait]
pub(crate) trait EntityResolutionStore: Send + Sync {
    async fn find_for_resolution(
        &self,
        user_id: UserId,
        name: &str,
        entity_type: ind_domain::EntityType,
    ) -> Result<Option<Entity>, AppError>;
    async fn block_candidates(
        &self,
        user_id: UserId,
        name: &str,
        entity_type: ind_domain::EntityType,
        limit: i64,
    ) -> Result<Vec<Entity>, AppError>;
    async fn insert_canonical(
        &self,
        user_id: UserId,
        name: &str,
        entity_type: ind_domain::EntityType,
        description: Option<&str>,
    ) -> Result<Entity, AppError>;
    async fn insert_alias(
        &self,
        user_id: UserId,
        name: &str,
        entity_type: ind_domain::EntityType,
        entity_id: EntityId,
    ) -> Result<(), AppError>;
    async fn register_alias_if_absent(
        &self,
        user_id: UserId,
        name: &str,
        entity_type: ind_domain::EntityType,
        entity_id: EntityId,
    ) -> Result<(), AppError>;
    async fn set_document_mentions(
        &self,
        user_id: UserId,
        document_id: ind_domain::DocumentId,
        mentions: &[(EntityId, i32)],
    ) -> Result<(), AppError>;
}

pub(crate) struct EntityRepositoryAdapter(pub(crate) Arc<dyn EntityRepository>);

#[async_trait]
impl EntityResolutionStore for EntityRepositoryAdapter {
    async fn find_for_resolution(
        &self,
        user_id: UserId,
        name: &str,
        entity_type: ind_domain::EntityType,
    ) -> Result<Option<Entity>, AppError> {
        self.0.find_for_resolution(user_id, name, entity_type).await
    }
    async fn block_candidates(
        &self,
        user_id: UserId,
        name: &str,
        entity_type: ind_domain::EntityType,
        limit: i64,
    ) -> Result<Vec<Entity>, AppError> {
        self.0
            .block_candidates(user_id, name, entity_type, limit)
            .await
    }
    async fn insert_canonical(
        &self,
        user_id: UserId,
        name: &str,
        entity_type: ind_domain::EntityType,
        description: Option<&str>,
    ) -> Result<Entity, AppError> {
        self.0
            .insert_canonical(user_id, name, entity_type, description)
            .await
    }
    async fn insert_alias(
        &self,
        user_id: UserId,
        name: &str,
        entity_type: ind_domain::EntityType,
        entity_id: EntityId,
    ) -> Result<(), AppError> {
        self.0
            .insert_alias(user_id, name, entity_type, entity_id)
            .await
    }
    async fn register_alias_if_absent(
        &self,
        user_id: UserId,
        name: &str,
        entity_type: ind_domain::EntityType,
        entity_id: EntityId,
    ) -> Result<(), AppError> {
        self.0
            .register_alias_if_absent(user_id, name, entity_type, entity_id)
            .await
    }
    async fn set_document_mentions(
        &self,
        user_id: UserId,
        document_id: ind_domain::DocumentId,
        mentions: &[(EntityId, i32)],
    ) -> Result<(), AppError> {
        self.0
            .set_document_mentions(user_id, document_id, mentions)
            .await
    }
}

/// Per-entity classification before any model call.
enum Decision {
    /// Exact entity or known alias hit.
    Hit(EntityId),
    /// No candidates — create a new entity.
    New,
    /// Lexically-similar candidates surfaced — needs adjudication.
    Pending(Vec<Entity>),
}

impl EntityResolver {
    pub(crate) fn with_store(
        entity_repo: Arc<dyn EntityResolutionStore>,
        ai_client: Arc<dyn AiProviderClient>,
    ) -> Self {
        Self {
            entity_repo,
            ai_client,
            credential_cipher: None,
        }
    }

    pub(crate) fn with_credential_cipher(mut self, cipher: Option<Arc<CredentialCipher>>) -> Self {
        self.credential_cipher = cipher;
        self
    }

    pub(crate) async fn resolve_document_entities(
        &self,
        user_id: UserId,
        config: &MilaConfig,
        doc_title: &str,
        extracted: &[ExtractedEntity],
    ) -> Result<Vec<(EntityId, i32)>, AppError> {
        let decisions = self.classify(user_id, extracted).await?;

        let ambiguous: Vec<usize> = decisions
            .iter()
            .enumerate()
            .filter_map(|(index, decision)| {
                matches!(decision, Decision::Pending(_)).then_some(index)
            })
            .collect();

        let matches = self
            .adjudicate_ambiguous(config, doc_title, extracted, &decisions, &ambiguous)
            .await;

        let mut resolved = Vec::with_capacity(extracted.len());
        for (position, entity) in extracted.iter().enumerate() {
            let count = entity.mention_count.max(1);
            let entity_id = match &decisions[position] {
                Decision::Hit(id) => *id,
                Decision::New => self.insert_canonical(user_id, entity).await?,
                Decision::Pending(_) => match matches.get(&position) {
                    Some(target) => {
                        self.entity_repo
                            .insert_alias(user_id, &entity.name, entity.entity_type, *target)
                            .await?;
                        *target
                    }
                    None => self.insert_canonical(user_id, entity).await?,
                },
            };
            self.record_model_aliases(user_id, entity_id, entity)
                .await?;
            resolved.push((entity_id, count));
        }
        Ok(resolved)
    }

    async fn classify(
        &self,
        user_id: UserId,
        extracted: &[ExtractedEntity],
    ) -> Result<Vec<Decision>, AppError> {
        let mut decisions = Vec::with_capacity(extracted.len());
        for entity in extracted {
            if let Some(found) = self
                .entity_repo
                .find_for_resolution(user_id, &entity.name, entity.entity_type)
                .await?
            {
                decisions.push(Decision::Hit(found.id));
                continue;
            }

            let mut candidates = self
                .entity_repo
                .block_candidates(user_id, &entity.name, entity.entity_type, CANDIDATE_LIMIT)
                .await?;
            for alias in &entity.aliases {
                if let Some(hit) = self
                    .entity_repo
                    .find_for_resolution(user_id, alias, entity.entity_type)
                    .await?
                    && !candidates.iter().any(|candidate| candidate.id == hit.id)
                {
                    candidates.push(hit);
                }
            }

            decisions.push(if candidates.is_empty() {
                Decision::New
            } else {
                Decision::Pending(candidates)
            });
        }
        Ok(decisions)
    }

    /// One batched adjudication call for all ambiguous entities. Returns confirmed merges as a map
    /// from the entity's position in `extracted` to its target entity id. On any model/parse error
    /// the map is empty (fail-open: ambiguous entities are created as new).
    async fn adjudicate_ambiguous(
        &self,
        config: &MilaConfig,
        doc_title: &str,
        extracted: &[ExtractedEntity],
        decisions: &[Decision],
        ambiguous: &[usize],
    ) -> HashMap<usize, EntityId> {
        let mut matches = HashMap::new();
        if ambiguous.is_empty() {
            return matches;
        }

        let items: Vec<AdjudicationItem> = ambiguous
            .iter()
            .map(|&position| AdjudicationItem {
                entity: &extracted[position],
                candidates: candidates_of(&decisions[position]),
            })
            .collect();

        let verdicts = match self.adjudicate_batch(config, doc_title, &items).await {
            Ok(verdicts) => verdicts,
            Err(err) => {
                tracing::warn!(
                    error = %err,
                    "entity resolution: batch adjudication failed; ambiguous entities created as new (fail-open)"
                );
                return matches;
            }
        };

        for verdict in verdicts {
            let Some(&position) = verdict
                .entity_index
                .checked_sub(1)
                .and_then(|index| ambiguous.get(index))
            else {
                continue;
            };
            let candidates = candidates_of(&decisions[position]);
            if let Some(candidate_number) = verdict.match_index
                && verdict.confidence >= MATCH_MIN_CONFIDENCE
                && (1..=candidates.len()).contains(&candidate_number)
            {
                matches.insert(position, candidates[candidate_number - 1].id);
            }
        }
        matches
    }

    async fn adjudicate_batch(
        &self,
        config: &MilaConfig,
        doc_title: &str,
        items: &[AdjudicationItem<'_>],
    ) -> Result<Vec<BatchVerdict>, AppError> {
        let provider = chat_provider_from_config(config, self.credential_cipher.as_deref())?;
        let mut system = ENTITY_RESOLUTION_SYSTEM_PROMPT.to_string();
        if !config.supports_structured_output {
            system.push_str("\n\nOutput only the JSON object described.");
        }
        let messages = vec![
            ChatMessage::system(system),
            ChatMessage::user(build_batch_resolution_prompt(doc_title, items)),
        ];
        let request = adjudication_request(config, messages, items.len());

        let response = self
            .ai_client
            .chat_completion(&provider, request)
            .await
            .map_err(map_ai_error)?;
        parse_batch_resolution(&first_choice_content(&response)?)
    }

    async fn insert_canonical(
        &self,
        user_id: UserId,
        entity: &ExtractedEntity,
    ) -> Result<EntityId, AppError> {
        Ok(self
            .entity_repo
            .insert_canonical(
                user_id,
                &entity.name,
                entity.entity_type,
                entity.description.as_deref(),
            )
            .await?
            .id)
    }

    /// Record the model's emitted aliases for `entity_id`, never hijacking an existing name.
    async fn record_model_aliases(
        &self,
        user_id: UserId,
        entity_id: EntityId,
        entity: &ExtractedEntity,
    ) -> Result<(), AppError> {
        for alias in &entity.aliases {
            let alias = alias.trim();
            if alias.is_empty() || alias == entity.name {
                continue;
            }
            self.entity_repo
                .register_alias_if_absent(user_id, alias, entity.entity_type, entity_id)
                .await?;
        }
        Ok(())
    }
}

fn candidates_of(decision: &Decision) -> &[Entity] {
    match decision {
        Decision::Pending(candidates) => candidates,
        _ => &[],
    }
}

fn adjudication_request(
    config: &MilaConfig,
    messages: Vec<ChatMessage>,
    item_count: usize,
) -> ChatCompletionRequest {
    let mut request = ChatCompletionRequest::new(config.chat_model.clone(), messages);
    if !config.supports_reasoning_effort {
        request.temperature = Some(0.0);
    }
    request.max_completion_tokens = Some(64 + 24 * item_count as u32);
    if config.supports_structured_output {
        request.response_format = Some(batch_resolution_response_format());
    }
    request
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    #[test]
    fn reasoning_capable_entity_adjudication_omits_sampling_parameters() {
        let request = adjudication_request(&reasoning_config(), Vec::new(), 1);
        let json = serde_json::to_value(request).unwrap();

        assert!(json.get("temperature").is_none());
        assert!(json.get("top_p").is_none());
        assert!(json.get("reasoning_effort").is_none());
    }

    #[test]
    fn sampling_entity_adjudication_keeps_temperature() {
        let mut config = reasoning_config();
        config.supports_reasoning_effort = false;
        let request = adjudication_request(&config, Vec::new(), 1);
        let json = serde_json::to_value(request).unwrap();

        assert_eq!(json.get("temperature"), Some(&serde_json::json!(0.0)));
        assert!(json.get("reasoning_effort").is_none());
    }

    fn reasoning_config() -> MilaConfig {
        MilaConfig {
            user_id: UserId::new(),
            chat_api_base: "https://api.openai.com/v1".into(),
            chat_api_key_enc: None,
            chat_model: "reasoning-model".into(),
            embedding_api_base: "https://api.openai.com/v1".into(),
            embedding_api_key_enc: None,
            embedding_model: "embedding-model".into(),
            embedding_dim: 768,
            byo_enabled: true,
            model_context_window: 16_000,
            chat_context_pct: 70,
            chunk_size: 800,
            chunk_overlap: 100,
            top_k: 6,
            cross_item_top_k: 20,
            cross_item_max_per_item: 3,
            enabled: true,
            supports_structured_output: true,
            supports_reasoning_effort: true,
            chat_cipher_version: 0,
            embedding_cipher_version: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
