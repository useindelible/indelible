use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::Utc;
use futures::StreamExt;

use ind_application::AppError;
use ind_application::repos::content_vector::{
    CollectionDocumentFtsQuery, CollectionDocumentVectorQuery, CrossDocumentFtsQuery,
    CrossDocumentVectorQuery, SingleDocumentFtsQuery, SingleDocumentVectorQuery,
};
use ind_domain::{
    AiPromptAction, AiPromptPreset, CollectionId, ContentVectorId, Document, DocumentId,
    DocumentType, DomainError, ItemType, MessageRole, MilaConfig, MilaMessage, MilaSession,
    MilaSessionId, MilaSessionType, PreparedContentKind, PreparedContentParent,
    PreparedItemContent, PreparedSectionKind, SearchHit, SearchResultKind, SearchSectionKind,
    SearchSectionRef, UserId,
};

use super::*;
use crate::test_support::ScriptedAiProvider;

struct ChatHarness {
    user_id: UserId,
    document: Document,
    prepared: Mutex<PreparedItemContent>,
    config: Mutex<MilaConfig>,
    session: Mutex<MilaSession>,
    vector_hits: Mutex<Vec<SearchHit>>,
    fts_hits: Mutex<Vec<SearchHit>>,
    fail_vector: Mutex<bool>,
    fail_fts: Mutex<bool>,
    messages: Mutex<Vec<MilaMessage>>,
    collection_vectors: Mutex<Vec<CollectionDocumentVectorQuery>>,
    collection_fts: Mutex<Vec<CollectionDocumentFtsQuery>>,
    provider: Arc<ScriptedAiProvider>,
}

impl ChatHarness {
    fn new(session_type: MilaSessionType) -> Arc<Self> {
        let user_id = UserId::new();
        let document = document(user_id, "Document");
        let collection_id = (session_type == MilaSessionType::Collection).then(CollectionId::new);
        Arc::new(Self {
            prepared: Mutex::new(prepared(&document, "small readable body")),
            config: Mutex::new(config(user_id)),
            session: Mutex::new(MilaSession {
                id: MilaSessionId::new(),
                user_id,
                document_id: (session_type == MilaSessionType::SingleDocument)
                    .then_some(document.id),
                collection_id,
                session_type,
                created_at: Utc::now(),
                last_active: Utc::now(),
            }),
            user_id,
            document,
            vector_hits: Mutex::new(Vec::new()),
            fts_hits: Mutex::new(Vec::new()),
            fail_vector: Mutex::new(false),
            fail_fts: Mutex::new(false),
            messages: Mutex::new(Vec::new()),
            collection_vectors: Mutex::new(Vec::new()),
            collection_fts: Mutex::new(Vec::new()),
            provider: Arc::new(ScriptedAiProvider::default()),
        })
    }

    fn service(self: &Arc<Self>) -> MilaChatService {
        MilaChatService::with_ports(
            self.clone(),
            self.clone(),
            self.clone(),
            self.clone(),
            self.clone(),
            self.clone(),
            self.provider.clone(),
        )
    }

    async fn run(self: &Arc<Self>) -> Vec<MilaChatDelta> {
        let session_id = self.session.lock().unwrap().id;
        let stream = self
            .service()
            .stream_chat(MilaChatRequest {
                user_id: self.user_id,
                session_id,
                question: "question".into(),
                highlight_text: None,
                highlight_offset: None,
            })
            .await
            .unwrap();
        stream.map(|delta| delta.unwrap()).collect().await
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
        model_context_window: 4096,
        chat_context_pct: 1,
        chunk_size: 8,
        chunk_overlap: 2,
        top_k: 3,
        cross_item_top_k: 3,
        cross_item_max_per_item: 1,
        enabled: true,
        supports_structured_output: true,
        supports_reasoning_effort: true,
        chat_cipher_version: 0,
        embedding_cipher_version: 0,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

fn document(user_id: UserId, title: &str) -> Document {
    Document {
        id: DocumentId::new(),
        user_id,
        document_type: DocumentType::Book,
        canonical_url: Some("https://example.com/book".into()),
        original_url: Some("https://example.com/book".into()),
        content_hash: None,
        title: title.into(),
        author: None,
        excerpt: None,
        published_at: None,
        language: Some("en".into()),
        domain: Some("example.com".into()),
        lead_image_url: None,
        thumbnail_url: None,
        word_count: None,
        reading_time_minutes: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

fn prepared(document: &Document, text: &str) -> PreparedItemContent {
    PreparedItemContent {
        document_id: document.id,
        user_id: document.user_id,
        source_kind: PreparedContentKind::Epub,
        title: document.title.clone(),
        root_text: text.into(),
        parents: vec![PreparedContentParent {
            kind: PreparedSectionKind::Chapter,
            key: "chapter-1".into(),
            title: Some("Wrong parent".into()),
            ordinal: 0,
            text: "wrong parent body".into(),
            locator: None,
        }],
        leaves: Vec::new(),
    }
}

fn hit(document_id: DocumentId, section: &str, snippet: &str) -> SearchHit {
    SearchHit {
        source_chunk_id: Some(ContentVectorId::new()),
        result_kind: SearchResultKind::Document,
        document_id: Some(document_id),
        delivery_id: None,
        source_entry_id: None,
        title: "Hit".into(),
        snippet: snippet.into(),
        score: 1.0,
        content_type: ItemType::Article,
        url: None,
        saved_at: Utc::now(),
        updated_at: Utc::now(),
        section: Some(SearchSectionRef {
            kind: SearchSectionKind::EpubChapter,
            key: section.into(),
            title: None,
        }),
        entity_chips: Vec::new(),
        sender_id: None,
    }
}
