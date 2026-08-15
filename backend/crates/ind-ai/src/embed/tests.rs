use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::Utc;
use ind_application::AppError;
use ind_domain::{
    ContentVector, Document, DocumentId, DocumentType, MilaConfig, PreparedContentKind,
    PreparedContentLeaf, PreparedContentParent, PreparedItemContent, PreparedSectionKind,
    SearchSectionKind, UserId,
};

use super::*;
use crate::test_support::ScriptedAiProvider;

struct EmbedHarness {
    prepared: Mutex<Option<PreparedItemContent>>,
    readable: Mutex<Option<String>>,
    document: Mutex<Document>,
    config: Mutex<MilaConfig>,
    replacements: Mutex<Vec<Vec<ContentVector>>>,
    provider: Arc<ScriptedAiProvider>,
}

impl EmbedHarness {
    fn new(document_id: DocumentId) -> Arc<Self> {
        let user_id = UserId::new();
        Arc::new(Self {
            prepared: Mutex::new(Some(prepared(document_id, user_id))),
            readable: Mutex::new(None),
            document: Mutex::new(document(document_id, user_id)),
            config: Mutex::new(config(user_id)),
            replacements: Mutex::new(Vec::new()),
            provider: Arc::new(ScriptedAiProvider::default()),
        })
    }
    fn indexer(self: &Arc<Self>) -> EmbeddingIndexer {
        EmbeddingIndexer::with_ports(
            self.clone(),
            self.clone(),
            self.clone(),
            self.clone(),
            self.provider.clone(),
            platform_defaults(),
        )
    }
}

fn document(document_id: DocumentId, user_id: UserId) -> Document {
    Document {
        id: document_id,
        user_id,
        document_type: DocumentType::Book,
        canonical_url: None,
        original_url: None,
        content_hash: None,
        title: "Book".into(),
        author: None,
        excerpt: None,
        published_at: None,
        language: None,
        domain: None,
        lead_image_url: None,
        thumbnail_url: None,
        word_count: None,
        reading_time_minutes: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

fn config(user_id: UserId) -> MilaConfig {
    MilaConfig {
        user_id,
        chat_api_base: "https://example.com/v1".into(),
        chat_api_key_enc: None,
        chat_model: "chat".into(),
        embedding_api_base: "https://example.com/v1".into(),
        embedding_api_key_enc: None,
        embedding_model: "embed".into(),
        embedding_dim: 4,
        byo_enabled: true,
        model_context_window: 16_000,
        chat_context_pct: 70,
        chunk_size: 8,
        chunk_overlap: 2,
        top_k: 5,
        cross_item_top_k: 10,
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

fn platform_defaults() -> MilaPlatformDefaults {
    MilaPlatformDefaults {
        chat_api_base: "https://example.com/v1".into(),
        chat_model: "chat".into(),
        embedding_api_base: "https://example.com/v1".into(),
        embedding_model: "embed".into(),
        embedding_dim: 4,
        model_context_window: 16_000,
        summary_max_output_tokens: 1024,
        tags_max_output_tokens: 1024,
        entities_max_output_tokens: 2000,
        chat_max_output_tokens: 1024,
        chat_context_pct: 70,
        chunk_size: 8,
        chunk_overlap: 2,
        top_k: 5,
        cross_item_top_k: 10,
        cross_item_max_per_item: 3,
        enabled: true,
        supports_structured_output: true,
        supports_reasoning_effort: true,
    }
}

fn prepared(document_id: DocumentId, user_id: UserId) -> PreparedItemContent {
    let leaves = ["first passage", "second passage"]
        .into_iter()
        .enumerate()
        .map(|(ordinal, text)| PreparedContentLeaf {
            parent_key: "chapter-1".into(),
            kind: PreparedSectionKind::Chapter,
            key: format!("chapter-1:{ordinal}"),
            ordinal: ordinal as i32,
            text: text.into(),
            locator: None,
        })
        .collect();
    PreparedItemContent {
        document_id,
        user_id,
        source_kind: PreparedContentKind::Epub,
        title: "Book".into(),
        root_text: "first passage second passage".into(),
        parents: vec![PreparedContentParent {
            kind: PreparedSectionKind::Chapter,
            key: "chapter-1".into(),
            title: Some("Chapter 1".into()),
            ordinal: 0,
            text: "first passage second passage".into(),
            locator: None,
        }],
        leaves,
    }
}

#[async_trait]
impl EmbeddingContent for EmbedHarness {
    async fn prepared(&self, _: DocumentId) -> Result<Option<PreparedItemContent>, AppError> {
        Ok(self.prepared.lock().unwrap().clone())
    }
    async fn readable_text(&self, _: DocumentId) -> Result<Option<String>, AppError> {
        Ok(self.readable.lock().unwrap().clone())
    }
}
#[async_trait]
impl EmbeddingConfig for EmbedHarness {
    async fn get(&self, _: UserId) -> Result<Option<MilaConfig>, AppError> {
        Ok(Some(self.config.lock().unwrap().clone()))
    }
}
#[async_trait]
impl EmbeddingVectors for EmbedHarness {
    async fn replace(
        &self,
        _: DocumentId,
        _: UserId,
        vectors: &[ContentVector],
        _: &EffectiveEmbeddingTarget,
        _: &MilaPlatformDefaults,
    ) -> Result<VectorReplacementOutcome, AppError> {
        self.replacements.lock().unwrap().push(vectors.to_vec());
        Ok(VectorReplacementOutcome::Committed)
    }
}
#[async_trait]
impl EmbeddingDocument for EmbedHarness {
    async fn get(&self, _: DocumentId) -> Result<Option<Document>, AppError> {
        Ok(Some(self.document.lock().unwrap().clone()))
    }

    async fn set_language_if_missing(
        &self,
        _: UserId,
        _: DocumentId,
        language: &str,
    ) -> Result<bool, AppError> {
        let mut document = self.document.lock().unwrap();
        if document.language.is_some() {
            return Ok(false);
        }
        document.language = Some(language.to_string());
        Ok(true)
    }
}
#[tokio::test]
async fn structured_content_replaces_once_after_all_calls_and_maps_chapters() {
    let document_id = DocumentId::new();
    let harness = EmbedHarness::new(document_id);
    harness.indexer().embed_document(document_id).await.unwrap();
    let replacements = harness.replacements.lock().unwrap();
    assert_eq!((replacements.len(), replacements[0].len()), (1, 2));
    assert!(
        replacements[0]
            .iter()
            .all(|v| v.section_kind == SearchSectionKind::EpubChapter)
    );
    assert!(
        replacements[0]
            .iter()
            .all(|vector| vector.search_config == "english")
    );
    assert_eq!(
        *harness.provider.embedding_inputs.lock().unwrap(),
        vec![
            "Title: Book\nSection: Chapter 1\n\nfirst passage",
            "Title: Book\nSection: Chapter 1\n\nsecond passage"
        ]
    );
}

#[tokio::test]
async fn declared_non_english_language_configures_every_chunk_as_simple() {
    let document_id = DocumentId::new();
    let harness = EmbedHarness::new(document_id);
    harness.document.lock().unwrap().language = Some("de-DE".into());

    harness.indexer().embed_document(document_id).await.unwrap();

    let replacements = harness.replacements.lock().unwrap();
    assert!(
        replacements[0]
            .iter()
            .all(|vector| vector.search_config == "simple")
    );
    assert_eq!(
        harness.document.lock().unwrap().language.as_deref(),
        Some("de-DE")
    );
}

#[tokio::test]
async fn later_provider_failure_leaves_vectors_untouched_and_retry_restarts_deterministically() {
    let document_id = DocumentId::new();
    let harness = EmbedHarness::new(document_id);
    *harness.provider.fail_embedding_call.lock().unwrap() = Some(2);
    assert!(harness.indexer().embed_document(document_id).await.is_err());
    assert!(harness.replacements.lock().unwrap().is_empty());
    *harness.provider.fail_embedding_call.lock().unwrap() = None;
    harness.indexer().embed_document(document_id).await.unwrap();
    let inputs = harness.provider.embedding_inputs.lock().unwrap();
    assert_eq!(inputs[0], inputs[2]);
    assert_eq!(harness.replacements.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn readable_fallback_chunks_and_recursively_splits_context_failures() {
    let document_id = DocumentId::new();
    let harness = EmbedHarness::new(document_id);
    *harness.prepared.lock().unwrap() = None;
    *harness.readable.lock().unwrap() = Some("one two three four five six seven eight".into());
    *harness.provider.embedding_context_limit.lock().unwrap() = Some(2);
    harness.indexer().embed_document(document_id).await.unwrap();
    assert!(harness.provider.embedding_inputs.lock().unwrap().len() > 2);
    let replacements = harness.replacements.lock().unwrap();
    assert_eq!(replacements.len(), 1);
    assert!(!replacements[0].is_empty());
}

#[tokio::test]
async fn missing_content_replaces_with_empty_but_disabled_config_does_nothing() {
    let document_id = DocumentId::new();
    let harness = EmbedHarness::new(document_id);
    *harness.prepared.lock().unwrap() = None;
    harness.indexer().embed_document(document_id).await.unwrap();
    assert!(harness.replacements.lock().unwrap()[0].is_empty());
    harness.replacements.lock().unwrap().clear();
    harness.config.lock().unwrap().enabled = false;
    harness.indexer().embed_document(document_id).await.unwrap();
    assert!(harness.replacements.lock().unwrap().is_empty());
}
