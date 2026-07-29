use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use chrono::Utc;

use ind_application::repos::content_vector::ContentVectorRepository;
use ind_application::repos::document::DocumentRepository;
use ind_application::repos::mila_config::MilaConfigRepository;
use ind_application::repos::prepared_content::PreparedContentProvider;
use ind_application::{AppError, classify_search_language};
use ind_auth::CredentialCipher;
use ind_domain::{
    ContentVector, ContentVectorId, Document, DocumentId, DomainError, MilaConfig,
    PreparedItemContent, PreparedSectionKind, SearchSectionKind, UserId,
};

use crate::chunker::{ChunkingConfig, approximate_token_count, chunk_text};
use crate::content::{embedding_provider_from_config, map_ai_error};
use crate::{AiError, AiProviderClient, AiProviderConfig, EmbeddingRequest};

fn map_section_kind(kind: PreparedSectionKind) -> SearchSectionKind {
    match kind {
        PreparedSectionKind::Item => SearchSectionKind::Item,
        PreparedSectionKind::Chapter => SearchSectionKind::EpubChapter,
    }
}

struct EmbeddingUnit {
    section_kind: SearchSectionKind,
    section_key: String,
    text: String,
}

enum EmbedTextError {
    Provider(AiError),
    App(AppError),
}

pub struct EmbeddingIndexer {
    content_provider: Arc<dyn EmbeddingContent>,
    mila_config_repo: Arc<dyn EmbeddingConfig>,
    content_vector_repo: Arc<dyn EmbeddingVectors>,
    document_repo: Arc<dyn EmbeddingDocument>,
    ai_client: Arc<dyn AiProviderClient>,
    credential_cipher: Option<Arc<CredentialCipher>>,
}

#[async_trait::async_trait]
trait EmbeddingContent: Send + Sync {
    async fn prepared(
        &self,
        document_id: DocumentId,
    ) -> Result<Option<PreparedItemContent>, AppError>;
    async fn readable_text(&self, document_id: DocumentId) -> Result<Option<String>, AppError>;
}

#[async_trait::async_trait]
trait EmbeddingConfig: Send + Sync {
    async fn get(&self, user_id: UserId) -> Result<Option<MilaConfig>, AppError>;
}

#[async_trait::async_trait]
trait EmbeddingVectors: Send + Sync {
    async fn replace(
        &self,
        document_id: DocumentId,
        vectors: &[ContentVector],
    ) -> Result<(), AppError>;
}

#[async_trait::async_trait]
trait EmbeddingDocument: Send + Sync {
    async fn get(&self, document_id: DocumentId) -> Result<Option<Document>, AppError>;
    async fn set_language_if_missing(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        language: &str,
    ) -> Result<bool, AppError>;
}

struct ContentAdapter(Arc<dyn PreparedContentProvider>);
struct ConfigAdapter(Arc<dyn MilaConfigRepository>);
struct VectorAdapter(Arc<dyn ContentVectorRepository>);
struct DocumentAdapter(Arc<dyn DocumentRepository>);

#[async_trait::async_trait]
impl EmbeddingContent for ContentAdapter {
    async fn prepared(&self, id: DocumentId) -> Result<Option<PreparedItemContent>, AppError> {
        self.0.load_for_document(id).await
    }

    async fn readable_text(&self, id: DocumentId) -> Result<Option<String>, AppError> {
        self.0.load_readable_text_for_document(id).await
    }
}

#[async_trait::async_trait]
impl EmbeddingConfig for ConfigAdapter {
    async fn get(&self, user_id: UserId) -> Result<Option<MilaConfig>, AppError> {
        self.0.get_by_user(user_id).await
    }
}

#[async_trait::async_trait]
impl EmbeddingVectors for VectorAdapter {
    async fn replace(&self, id: DocumentId, vectors: &[ContentVector]) -> Result<(), AppError> {
        self.0.replace_for_document(id, vectors).await
    }
}

#[async_trait::async_trait]
impl EmbeddingDocument for DocumentAdapter {
    async fn get(&self, id: DocumentId) -> Result<Option<Document>, AppError> {
        self.0.find_by_id_global(id).await
    }

    async fn set_language_if_missing(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        language: &str,
    ) -> Result<bool, AppError> {
        self.0
            .set_language_if_missing(user_id, document_id, language)
            .await
    }
}

impl EmbeddingIndexer {
    pub fn new(
        content_provider: Arc<dyn PreparedContentProvider>,
        mila_config_repo: Arc<dyn MilaConfigRepository>,
        content_vector_repo: Arc<dyn ContentVectorRepository>,
        document_repo: Arc<dyn DocumentRepository>,
        ai_client: Arc<dyn AiProviderClient>,
    ) -> Self {
        Self {
            content_provider: Arc::new(ContentAdapter(content_provider)),
            mila_config_repo: Arc::new(ConfigAdapter(mila_config_repo)),
            content_vector_repo: Arc::new(VectorAdapter(content_vector_repo)),
            document_repo: Arc::new(DocumentAdapter(document_repo)),
            ai_client,
            credential_cipher: None,
        }
    }

    #[cfg(test)]
    fn with_ports(
        content_provider: Arc<dyn EmbeddingContent>,
        mila_config_repo: Arc<dyn EmbeddingConfig>,
        content_vector_repo: Arc<dyn EmbeddingVectors>,
        document_repo: Arc<dyn EmbeddingDocument>,
        ai_client: Arc<dyn AiProviderClient>,
    ) -> Self {
        Self {
            content_provider,
            mila_config_repo,
            content_vector_repo,
            document_repo,
            ai_client,
            credential_cipher: None,
        }
    }

    pub fn with_credential_cipher(mut self, cipher: Option<Arc<CredentialCipher>>) -> Self {
        self.credential_cipher = cipher;
        self
    }

    pub async fn embed_document(&self, document_id: DocumentId) -> Result<(), AppError> {
        let document = self.document(document_id).await?;
        let user_id = document.user_id;
        let Some(config) = self.mila_config_repo.get(user_id).await? else {
            return Ok(());
        };
        if !config.enabled {
            return Ok(());
        }

        let prepared = self.content_provider.prepared(document_id).await?;
        let readable = match &prepared {
            Some(prepared) if !prepared.leaves.is_empty() => None,
            _ => self.content_provider.readable_text(document_id).await?,
        };
        let detected_from = prepared
            .as_ref()
            .map(|content| content.root_text.as_str())
            .or(readable.as_deref())
            .unwrap_or_default();
        let language_decision = classify_search_language(
            document.language.as_deref(),
            &[
                document.title.as_str(),
                document.excerpt.as_deref().unwrap_or_default(),
                detected_from,
            ],
        );
        if document.language.is_none()
            && let Some(language) = language_decision.language.as_deref()
        {
            self.document_repo
                .set_language_if_missing(user_id, document_id, language)
                .await?;
        }
        let search_config = language_decision.search_config.as_regconfig().to_string();

        // Build the text units to embed. Structured leaves come from prepared content; otherwise
        // the readable-text fallback is chunked with the user's configured Mila chunk settings.
        let units: Vec<EmbeddingUnit> = match &prepared {
            Some(prepared) if !prepared.leaves.is_empty() => prepared
                .leaves
                .iter()
                .map(|leaf| EmbeddingUnit {
                    section_kind: map_section_kind(leaf.kind),
                    section_key: leaf.parent_key.clone(),
                    text: prepared.enriched_leaf_text(leaf),
                })
                .collect(),
            _ => match readable {
                Some(text) if !text.trim().is_empty() => chunk_text(
                    text.as_str(),
                    ChunkingConfig {
                        chunk_size: config.chunk_size.max(1) as usize,
                        chunk_overlap: config.chunk_overlap.max(0) as usize,
                    },
                )
                .into_iter()
                .map(|chunk| EmbeddingUnit {
                    section_kind: SearchSectionKind::Item,
                    section_key: String::new(),
                    text: chunk.content,
                })
                .collect(),
                _ => Vec::new(),
            },
        };

        if units.is_empty() {
            return self.content_vector_repo.replace(document_id, &[]).await;
        }

        let provider = embedding_provider_from_config(&config, self.credential_cipher.as_deref())?;
        let mut vectors = Vec::new();
        let mut next_chunk_indexes = HashMap::<String, i32>::new();
        for unit in units {
            let embedded = self
                .embed_text_with_context_retry(&provider, &config, user_id, document_id, unit.text)
                .await?;
            for (text, embedding) in embedded {
                let next_chunk_index = next_chunk_indexes
                    .entry(unit.section_key.clone())
                    .or_default();
                let chunk_index = *next_chunk_index;
                *next_chunk_index = next_chunk_index.checked_add(1).ok_or_else(|| {
                    AppError::Domain(DomainError::InvariantViolation {
                        message: "embedding chunk index overflow".into(),
                    })
                })?;
                let token_count = approximate_token_count(&text).max(1) as i32;
                vectors.push(ContentVector {
                    id: ContentVectorId::new(),
                    document_id,
                    user_id,
                    section_kind: unit.section_kind,
                    section_key: unit.section_key.clone(),
                    chunk_index,
                    content: text,
                    token_count,
                    search_config: search_config.clone(),
                    embedding,
                    embedding_model: config.embedding_model.clone(),
                    embedding_dim: config.embedding_dim,
                    created_at: Utc::now(),
                });
            }
        }

        self.content_vector_repo
            .replace(document_id, &vectors)
            .await
    }

    async fn embed_text_with_context_retry(
        &self,
        provider: &AiProviderConfig,
        config: &MilaConfig,
        user_id: UserId,
        document_id: DocumentId,
        text: String,
    ) -> Result<Vec<(String, Vec<f32>)>, AppError> {
        let mut pending = VecDeque::from([text]);
        let mut embedded = Vec::new();

        while let Some(text) = pending.pop_front() {
            match self
                .embed_text(provider, config, user_id, document_id, &text)
                .await
            {
                Ok(embedding) => embedded.push((text, embedding)),
                Err(EmbedTextError::Provider(error)) if is_context_length_error(&error) => {
                    let parts = split_text_for_context_retry(&text);
                    if parts.len() <= 1 {
                        return Err(map_ai_error(error));
                    }
                    for part in parts.into_iter().rev() {
                        pending.push_front(part);
                    }
                }
                Err(EmbedTextError::Provider(error)) => return Err(map_ai_error(error)),
                Err(EmbedTextError::App(error)) => return Err(error),
            }
        }

        Ok(embedded)
    }

    async fn embed_text(
        &self,
        provider: &AiProviderConfig,
        config: &MilaConfig,
        user_id: UserId,
        document_id: DocumentId,
        text: &str,
    ) -> Result<Vec<f32>, EmbedTextError> {
        let response = self
            .ai_client
            .embedding(
                provider,
                EmbeddingRequest {
                    model: config.embedding_model.clone(),
                    input: text.to_string(),
                    user: Some(user_id.to_string()),
                    dimensions: Some(config.embedding_dim),
                },
            )
            .await
            .map_err(EmbedTextError::Provider)?;

        let actual_dim = i32::try_from(response.embedding.len()).map_err(|_| {
            EmbedTextError::App(AppError::Domain(DomainError::InvariantViolation {
                message: "embedding dimension overflow".into(),
            }))
        })?;
        if actual_dim != config.embedding_dim {
            return Err(EmbedTextError::App(AppError::Domain(
                DomainError::InvariantViolation {
                    message: format!(
                        "embedding dimension mismatch for document {document_id}: expected {}, got {}",
                        config.embedding_dim, actual_dim
                    ),
                },
            )));
        }

        Ok(response.embedding)
    }

    async fn document(&self, document_id: DocumentId) -> Result<Document, AppError> {
        self.document_repo.get(document_id).await?.ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "document",
                id: document_id.to_string(),
            })
        })
    }
}

fn is_context_length_error(error: &AiError) -> bool {
    let AiError::ProviderError { message, .. } = error else {
        return false;
    };
    let lower = message.to_ascii_lowercase();
    lower.contains("context length")
        || lower.contains("too long")
        || lower.contains("too many tokens")
}

fn split_text_for_context_retry(text: &str) -> Vec<String> {
    let chars = text.chars().collect::<Vec<_>>();
    if chars.len() <= 1 {
        return Vec::new();
    }

    let midpoint = chars.len() / 2;
    [chars[..midpoint].iter(), chars[midpoint..].iter()]
        .into_iter()
        .map(|part| part.copied().collect::<String>())
        .filter(|part| !part.is_empty())
        .collect()
}

#[cfg(test)]
mod tests;
