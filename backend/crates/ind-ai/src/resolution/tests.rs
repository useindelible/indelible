use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::Utc;
use ind_application::AppError;
use ind_domain::{
    DocumentId, Entity, EntityId, EntityType, ExtractedEntity, MessageRole, MilaConfig, UserId,
};

use super::{
    AdjudicationItem, ENTITY_RESOLUTION_SYSTEM_PROMPT, EntityResolutionStore, EntityResolver,
    build_batch_resolution_prompt,
};
use crate::{
    AiError, AiProviderClient, AiProviderConfig, ChatCompletionChoice, ChatCompletionRequest,
    ChatCompletionResponse, ChatCompletionStream, ChatMessage, EmbeddingRequest, EmbeddingResponse,
    UsageStats,
};

/// In-memory store mirroring the Postgres semantics the resolver relies on: exact (type, name)
/// hits and type-scoped aliases resolve directly; blocking is type-agnostic and excludes only the
/// exact (type, name) row. Trigram nuances stay in the persistence tests.
#[derive(Default)]
struct FakeStore {
    canonical: Mutex<Vec<Entity>>,
    aliases: Mutex<Vec<(EntityType, String, EntityId)>>,
}

impl FakeStore {
    fn canonical_count(&self) -> usize {
        self.canonical.lock().unwrap().len()
    }

    fn alias_target(&self, entity_type: EntityType, name: &str) -> Option<EntityId> {
        self.aliases
            .lock()
            .unwrap()
            .iter()
            .find(|(ty, alias, _)| *ty == entity_type && alias == name)
            .map(|(_, _, id)| *id)
    }
}

#[async_trait]
impl EntityResolutionStore for FakeStore {
    async fn find_for_resolution(
        &self,
        user_id: UserId,
        name: &str,
        entity_type: EntityType,
    ) -> Result<Option<Entity>, AppError> {
        let canonical = self.canonical.lock().unwrap();
        if let Some(entity) = canonical.iter().find(|entity| {
            entity.user_id == user_id && entity.entity_type == entity_type && entity.name == name
        }) {
            return Ok(Some(entity.clone()));
        }
        let target = self.alias_target(entity_type, name);
        Ok(target.and_then(|id| canonical.iter().find(|entity| entity.id == id).cloned()))
    }

    async fn block_candidates(
        &self,
        user_id: UserId,
        name: &str,
        entity_type: EntityType,
        limit: i64,
    ) -> Result<Vec<Entity>, AppError> {
        let query = name.to_lowercase();
        let query_words: Vec<&str> = query.split_whitespace().collect();
        Ok(self
            .canonical
            .lock()
            .unwrap()
            .iter()
            .filter(|entity| {
                entity.user_id == user_id
                    && !(entity.entity_type == entity_type && entity.name == name)
            })
            .filter(|entity| {
                let candidate = entity.name.to_lowercase();
                let candidate_words: Vec<&str> = candidate.split_whitespace().collect();
                candidate == query
                    || query_words
                        .iter()
                        .all(|word| candidate_words.contains(word))
                    || candidate_words
                        .iter()
                        .all(|word| query_words.contains(word))
            })
            .take(limit.max(0) as usize)
            .cloned()
            .collect())
    }

    async fn insert_canonical(
        &self,
        user_id: UserId,
        name: &str,
        entity_type: EntityType,
        description: Option<&str>,
    ) -> Result<Entity, AppError> {
        let mut canonical = self.canonical.lock().unwrap();
        if let Some(existing) = canonical.iter().find(|entity| {
            entity.user_id == user_id && entity.entity_type == entity_type && entity.name == name
        }) {
            return Ok(existing.clone());
        }
        let entity = Entity {
            id: EntityId::new(),
            user_id,
            name: name.to_string(),
            entity_type,
            description: description.map(str::to_string),
            created_at: Utc::now(),
        };
        canonical.push(entity.clone());
        Ok(entity)
    }

    async fn insert_alias(
        &self,
        _user_id: UserId,
        name: &str,
        entity_type: EntityType,
        entity_id: EntityId,
    ) -> Result<(), AppError> {
        let mut aliases = self.aliases.lock().unwrap();
        aliases.retain(|(ty, alias, _)| !(*ty == entity_type && alias == name));
        aliases.push((entity_type, name.to_string(), entity_id));
        Ok(())
    }

    async fn register_alias_if_absent(
        &self,
        user_id: UserId,
        name: &str,
        entity_type: EntityType,
        entity_id: EntityId,
    ) -> Result<(), AppError> {
        let is_entity = self.canonical.lock().unwrap().iter().any(|entity| {
            entity.user_id == user_id && entity.entity_type == entity_type && entity.name == name
        });
        if is_entity || self.alias_target(entity_type, name).is_some() {
            return Ok(());
        }
        self.aliases
            .lock()
            .unwrap()
            .push((entity_type, name.to_string(), entity_id));
        Ok(())
    }

    async fn set_document_mentions(
        &self,
        _user_id: UserId,
        _document_id: DocumentId,
        _mentions: &[(EntityId, i32)],
    ) -> Result<(), AppError> {
        Ok(())
    }
}

/// Returns queued responses in order and counts calls; an empty queue yields `{}` so the resolver
/// fails open exactly as it would on a malformed model reply.
#[derive(Default)]
struct QueuedAiClient {
    responses: Mutex<VecDeque<String>>,
    requests: Mutex<Vec<ChatCompletionRequest>>,
}

impl QueuedAiClient {
    fn push(&self, content: &str) {
        self.responses
            .lock()
            .unwrap()
            .push_back(content.to_string());
    }

    fn calls(&self) -> usize {
        self.requests.lock().unwrap().len()
    }

    fn last_user_prompt(&self) -> String {
        self.requests
            .lock()
            .unwrap()
            .last()
            .and_then(|request| {
                request
                    .messages
                    .last()
                    .map(|message| message.content.clone())
            })
            .unwrap_or_default()
    }
}

#[async_trait]
impl AiProviderClient for QueuedAiClient {
    async fn chat_completion(
        &self,
        _provider: &AiProviderConfig,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AiError> {
        self.requests.lock().unwrap().push(request);
        let content = self
            .responses
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| "{}".to_string());
        Ok(ChatCompletionResponse {
            id: "resp_test".into(),
            model: "test-model".into(),
            choices: vec![ChatCompletionChoice {
                index: 0,
                message: ChatMessage::new(MessageRole::Assistant, content),
                finish_reason: Some("stop".into()),
            }],
            usage: Some(UsageStats {
                prompt_tokens: 10,
                completion_tokens: Some(5),
                total_tokens: 15,
            }),
        })
    }

    async fn chat_completion_stream(
        &self,
        _provider: &AiProviderConfig,
        _request: ChatCompletionRequest,
    ) -> Result<ChatCompletionStream, AiError> {
        unreachable!()
    }

    async fn embedding(
        &self,
        _provider: &AiProviderConfig,
        _request: EmbeddingRequest,
    ) -> Result<EmbeddingResponse, AiError> {
        unreachable!()
    }
}

pub(super) fn sample_config(user_id: UserId) -> MilaConfig {
    MilaConfig {
        user_id,
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

fn extracted(name: &str, entity_type: EntityType, description: &str) -> ExtractedEntity {
    ExtractedEntity {
        name: name.into(),
        entity_type,
        description: Some(description.into()),
        mention_count: 1,
        aliases: Vec::new(),
    }
}

struct Harness {
    store: Arc<FakeStore>,
    ai: Arc<QueuedAiClient>,
    resolver: EntityResolver,
    user: UserId,
    config: MilaConfig,
}

fn harness() -> Harness {
    let store = Arc::new(FakeStore::default());
    let ai = Arc::new(QueuedAiClient::default());
    let resolver = EntityResolver::with_store(store.clone(), ai.clone());
    let user = UserId::new();
    let config = sample_config(user);
    Harness {
        store,
        ai,
        resolver,
        user,
        config,
    }
}

impl Harness {
    async fn resolve(&self, entity: ExtractedEntity) -> EntityId {
        self.resolver
            .resolve_document_entities(self.user, &self.config, "Doc", &[entity])
            .await
            .unwrap()[0]
            .0
    }
}

#[tokio::test]
async fn same_type_exact_hit_resolves_without_a_model_call() {
    let h = harness();
    let first = h
        .resolve(extracted(
            "DeepSeek",
            EntityType::Organization,
            "Chinese AI company",
        ))
        .await;
    let second = h
        .resolve(extracted("DeepSeek", EntityType::Organization, "AI lab"))
        .await;

    assert_eq!(first, second);
    assert_eq!(h.ai.calls(), 0);
    assert_eq!(h.store.canonical_count(), 1);
}

#[tokio::test]
async fn same_name_under_another_type_is_adjudicated_and_merged() {
    // Unambiguous type drift: the same regulation labelled event, then topic.
    let h = harness();
    let event = h
        .resolve(extracted(
            "Digital Markets Act",
            EntityType::Event,
            "EU regulation on large online platforms",
        ))
        .await;

    h.ai.push(r#"{"results":[{"entity":1,"match":1,"confidence":0.95}]}"#);
    let merged = h
        .resolve(extracted(
            "Digital Markets Act",
            EntityType::Work,
            "EU regulation on large online platforms",
        ))
        .await;

    assert_eq!(merged, event);
    assert_eq!(h.ai.calls(), 1);
    assert_eq!(h.store.canonical_count(), 1);
    assert_eq!(
        h.store
            .alias_target(EntityType::Work, "Digital Markets Act"),
        Some(event)
    );

    let again = h
        .resolve(extracted(
            "Digital Markets Act",
            EntityType::Work,
            "EU regulation on large online platforms",
        ))
        .await;
    assert_eq!(again, event);
    assert_eq!(
        h.ai.calls(),
        1,
        "the alias under the extracted type resolves for free"
    );
}

#[tokio::test]
async fn same_name_under_another_type_stays_separate_when_model_rejects() {
    let h = harness();
    let company = h
        .resolve(extracted(
            "Amazon",
            EntityType::Organization,
            "E-commerce company",
        ))
        .await;

    h.ai.push(r#"{"results":[{"entity":1,"match":null,"confidence":0.9}]}"#);
    let river = h
        .resolve(extracted(
            "Amazon",
            EntityType::Location,
            "River in South America",
        ))
        .await;

    assert_ne!(river, company);
    assert_eq!(h.ai.calls(), 1);
    assert_eq!(h.store.canonical_count(), 2);
    assert_eq!(h.store.alias_target(EntityType::Location, "Amazon"), None);
}

#[tokio::test]
async fn adjudication_prompt_shows_each_candidate_type() {
    let h = harness();
    h.resolve(extracted(
        "DeepSeek",
        EntityType::Organization,
        "Chinese AI company",
    ))
    .await;
    h.ai.push(r#"{"results":[{"entity":1,"match":null,"confidence":0.5}]}"#);
    h.resolve(extracted(
        "DeepSeek",
        EntityType::Work,
        "Chinese AI model family",
    ))
    .await;

    let prompt = h.ai.last_user_prompt();
    assert!(prompt.contains("name: DeepSeek\n  type: work"), "{prompt}");
    assert!(
        prompt.contains("1. name: DeepSeek   type: organization   description: Chinese AI company"),
        "{prompt}"
    );
}

#[test]
fn system_prompt_treats_type_labels_as_weak_evidence() {
    assert!(ENTITY_RESOLUTION_SYSTEM_PROMPT.contains("weak evidence"));
    assert!(ENTITY_RESOLUTION_SYSTEM_PROMPT.contains("Digital Markets Act"));
    assert!(ENTITY_RESOLUTION_SYSTEM_PROMPT.contains("Amazon/location the river => null"));
}

#[test]
fn batch_prompt_renders_candidate_type_inline() {
    let entity = extracted("DeepSeek", EntityType::Work, "model family");
    let candidate = Entity {
        id: EntityId::new(),
        user_id: UserId::new(),
        name: "DeepSeek".into(),
        entity_type: EntityType::Organization,
        description: Some("AI company".into()),
        created_at: Utc::now(),
    };
    let prompt = build_batch_resolution_prompt(
        "Doc",
        &[AdjudicationItem {
            entity: &entity,
            candidates: std::slice::from_ref(&candidate),
        }],
    );
    assert!(prompt.contains("1. name: DeepSeek   type: organization   description: AI company"));
}
