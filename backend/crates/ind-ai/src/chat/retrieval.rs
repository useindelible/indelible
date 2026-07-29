use ind_application::AppError;
use ind_application::repos::content_vector::{
    CollectionDocumentFtsQuery, CollectionDocumentVectorQuery, CrossDocumentFtsQuery,
    CrossDocumentVectorQuery, SingleDocumentFtsQuery, SingleDocumentVectorQuery,
};
use ind_domain::{Document, DocumentId, DomainError, MilaConfig, SearchHit, UserId};
use tracing::{debug, warn};

use crate::EmbeddingRequest;
use crate::content::{embedding_provider_from_config, map_ai_error};

use super::MilaChatService;
use super::fusion::{MAX_PER_SECTION, apply_diversity_cap, reciprocal_rank_fusion};

pub(super) struct CollectionHybridSearchOutcome {
    pub(super) hits: Vec<SearchHit>,
    pub(super) retrieval_degraded: Option<String>,
}

impl MilaChatService {
    pub(super) async fn search_single_document_hybrid(
        &self,
        config: &MilaConfig,
        user_id: UserId,
        document_id: DocumentId,
        query_text: &str,
        limit: i64,
    ) -> Result<Vec<SearchHit>, AppError> {
        let fetch_limit = limit.saturating_mul(2).max(limit);

        let fts_query = SingleDocumentFtsQuery {
            user_id,
            document_id,
            text_query: query_text.to_string(),
            limit: fetch_limit,
        };
        let fts_future = self.content_vector_repo.fts_single_document(&fts_query);
        let embed_future = self.embed_query(config, user_id, query_text);

        let (fts_result, embed_result) = futures::join!(fts_future, embed_future);

        let fts_hits = match fts_result {
            Ok(hits) => hits,
            Err(err) => {
                warn!(error = %err, "FTS single-document retrieval failed, proceeding with vector-only");
                Vec::new()
            }
        };

        let vector_hits = match embed_result {
            Ok(embedding) => {
                match self
                    .content_vector_repo
                    .search_single_document(&SingleDocumentVectorQuery {
                        user_id,
                        document_id,
                        query_embedding: embedding,
                        embedding_model: config.embedding_model.clone(),
                        embedding_dim: config.embedding_dim,
                        section_kind: None,
                        limit: fetch_limit,
                    })
                    .await
                {
                    Ok(hits) => hits,
                    Err(err) => {
                        warn!(error = %err, "vector single-document search failed, proceeding with FTS-only");
                        Vec::new()
                    }
                }
            }
            Err(err) => {
                warn!(error = %err, "embedding failed, proceeding with FTS-only");
                Vec::new()
            }
        };

        debug!(
            vector_count = vector_hits.len(),
            fts_count = fts_hits.len(),
            "single-document hybrid retrieval sources"
        );

        let fused = reciprocal_rank_fusion(vector_hits, fts_hits, MAX_PER_SECTION);
        Ok(fused.into_iter().take(limit as usize).collect())
    }

    pub(super) async fn search_cross_item_hybrid(
        &self,
        config: &MilaConfig,
        question: &str,
    ) -> Result<Vec<SearchHit>, AppError> {
        let vector_fetch_limit = i64::from(config.cross_item_top_k.max(1))
            .saturating_mul(i64::from(config.cross_item_max_per_item.max(1)))
            .saturating_mul(4);
        let fts_fetch_limit = vector_fetch_limit;

        let fts_query = CrossDocumentFtsQuery {
            user_id: config.user_id,
            text_query: question.to_string(),
            limit: fts_fetch_limit,
        };
        let fts_future = self.content_vector_repo.fts_cross_document(&fts_query);
        let embed_future = self.embed_query(config, config.user_id, question);

        let (fts_result, embed_result) = futures::join!(fts_future, embed_future);

        let fts_hits = match fts_result {
            Ok(hits) => hits,
            Err(err) => {
                warn!(error = %err, "FTS cross-document retrieval failed, proceeding with vector-only");
                Vec::new()
            }
        };

        let vector_hits = match embed_result {
            Ok(embedding) => {
                match self
                    .content_vector_repo
                    .search_cross_document(&CrossDocumentVectorQuery {
                        user_id: config.user_id,
                        query_embedding: embedding,
                        embedding_model: config.embedding_model.clone(),
                        embedding_dim: config.embedding_dim,
                        section_kind: None,
                        limit: vector_fetch_limit.max(i64::from(config.cross_item_top_k.max(1))),
                    })
                    .await
                {
                    Ok(hits) => hits,
                    Err(err) => {
                        warn!(error = %err, "vector cross-item search failed, proceeding with FTS-only");
                        Vec::new()
                    }
                }
            }
            Err(err) => {
                warn!(error = %err, "embedding failed, proceeding with FTS-only");
                Vec::new()
            }
        };

        debug!(
            vector_count = vector_hits.len(),
            fts_count = fts_hits.len(),
            "cross-item hybrid retrieval sources"
        );

        let fused = reciprocal_rank_fusion(vector_hits, fts_hits, MAX_PER_SECTION);
        Ok(apply_diversity_cap(
            fused,
            config.cross_item_max_per_item.max(1) as usize,
            config.cross_item_top_k.max(1) as usize,
        ))
    }

    pub(super) async fn search_collection_hybrid(
        &self,
        user_id: UserId,
        collection_id: ind_domain::CollectionId,
        config: &MilaConfig,
        question: &str,
    ) -> Result<CollectionHybridSearchOutcome, AppError> {
        let vector_fetch_limit = i64::from(config.cross_item_top_k.max(1))
            .saturating_mul(i64::from(config.cross_item_max_per_item.max(1)))
            .saturating_mul(4);
        let fts_fetch_limit = vector_fetch_limit;

        let fts_query = CollectionDocumentFtsQuery {
            user_id,
            collection_id,
            include_descendants: true,
            text_query: question.to_string(),
            limit: fts_fetch_limit,
        };
        let fts_future = self.content_vector_repo.fts_collection_document(&fts_query);
        let embed_future = self.embed_query(config, user_id, question);

        let (fts_result, embed_result) = futures::join!(fts_future, embed_future);

        let mut fts_failed = false;
        let fts_hits = match fts_result {
            Ok(hits) => hits,
            Err(err) => {
                fts_failed = true;
                warn!(error = %err, "FTS collection retrieval failed, proceeding with vector-only");
                Vec::new()
            }
        };

        let mut vector_failed = false;
        let mut embedding_failed = false;
        let vector_hits = match embed_result {
            Ok(embedding) => {
                match self
                    .content_vector_repo
                    .search_collection_document(&CollectionDocumentVectorQuery {
                        user_id,
                        collection_id,
                        include_descendants: true,
                        query_embedding: embedding,
                        embedding_model: config.embedding_model.clone(),
                        embedding_dim: config.embedding_dim,
                        section_kind: None,
                        limit: vector_fetch_limit.max(i64::from(config.cross_item_top_k.max(1))),
                    })
                    .await
                {
                    Ok(hits) => hits,
                    Err(err) => {
                        vector_failed = true;
                        warn!(error = %err, "vector collection search failed, proceeding with FTS-only");
                        Vec::new()
                    }
                }
            }
            Err(err) => {
                embedding_failed = true;
                warn!(error = %err, "embedding failed, proceeding with FTS-only");
                Vec::new()
            }
        };

        debug!(
            vector_count = vector_hits.len(),
            fts_count = fts_hits.len(),
            "collection hybrid retrieval sources"
        );

        let fused = reciprocal_rank_fusion(vector_hits, fts_hits, MAX_PER_SECTION);
        Ok(CollectionHybridSearchOutcome {
            hits: apply_diversity_cap(
                fused,
                config.cross_item_max_per_item.max(1) as usize,
                config.cross_item_top_k.max(1) as usize,
            ),
            retrieval_degraded: collection_retrieval_degraded_reason(
                fts_failed,
                vector_failed,
                embedding_failed,
            ),
        })
    }

    async fn embed_query(
        &self,
        config: &MilaConfig,
        user_id: UserId,
        query_text: &str,
    ) -> Result<Vec<f32>, AppError> {
        let provider = embedding_provider_from_config(config, self.credential_cipher.as_deref())?;
        let response = self
            .ai_client
            .embedding(
                &provider,
                EmbeddingRequest {
                    model: config.embedding_model.clone(),
                    input: query_text.to_string(),
                    user: Some(user_id.to_string()),
                    dimensions: Some(config.embedding_dim),
                },
            )
            .await
            .map_err(map_ai_error)?;

        let actual_dim = i32::try_from(response.embedding.len()).map_err(|_| {
            AppError::Domain(DomainError::InvariantViolation {
                message: "embedding dimension overflow".into(),
            })
        })?;
        if actual_dim != config.embedding_dim {
            return Err(AppError::Domain(DomainError::InvariantViolation {
                message: format!(
                    "embedding dimension mismatch for user {}: expected {}, got {}",
                    user_id, config.embedding_dim, actual_dim
                ),
            }));
        }

        Ok(response.embedding)
    }

    pub(super) async fn load_document(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<Document, AppError> {
        self.document_repo
            .find_by_id(user_id, document_id)
            .await?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "document",
                    id: document_id.to_string(),
                })
            })
    }

    /// Plain readable text for single-document chat (TASK-234). Resolves prepared content via the
    /// id-reuse bridge (`load_for_document`), falling back to the document-addressable readable text
    /// for net-new feed-prepared documents that have no legacy item.
    pub(super) async fn load_document_plain_text(
        &self,
        document_id: DocumentId,
    ) -> Result<String, AppError> {
        let text = match self.content_provider.load_for_document(document_id).await? {
            Some(prepared) if !prepared.root_text.trim().is_empty() => prepared.root_text,
            _ => self
                .content_provider
                .load_readable_text_for_document(document_id)
                .await?
                .filter(|text| !text.trim().is_empty())
                .ok_or_else(|| {
                    AppError::Domain(DomainError::NotFound {
                        entity: "prepared_content",
                        id: document_id.to_string(),
                    })
                })?,
        };

        Ok(text)
    }
}

fn collection_retrieval_degraded_reason(
    fts_failed: bool,
    vector_failed: bool,
    embedding_failed: bool,
) -> Option<String> {
    let mut reasons = Vec::new();
    if fts_failed {
        reasons.push("fts_failed");
    }
    if embedding_failed {
        reasons.push("embedding_failed");
    } else if vector_failed {
        reasons.push("vector_failed");
    }

    if reasons.is_empty() {
        None
    } else {
        Some(reasons.join(","))
    }
}
