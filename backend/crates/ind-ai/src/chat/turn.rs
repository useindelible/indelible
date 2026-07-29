use chrono::Utc;
use uuid::Uuid;

use ind_application::AppError;
use ind_domain::{
    DocumentId, DomainError, MessageRole, MilaConfig, MilaMessage, MilaMessageId, MilaSession,
    MilaSessionType, UserId,
};

use crate::ChatMessage;
use crate::content::{chat_provider_from_config, extract_text_window};
use crate::prompt::{
    build_cross_item_messages, build_highlight_messages, build_single_document_rag_messages,
    build_single_document_stuffing_messages, format_document_metadata,
};
use crate::token_estimate::{
    HIGHLIGHT_WINDOW_TOKENS, chars_for_tokens, chat_inline_token_budget, exceeds_context_threshold,
};

use super::fusion::{collect_source_chunk_ids, distinct_source_label_count};
use super::prompting::build_chat_request;
use super::retrieval::CollectionHybridSearchOutcome;
use super::{MilaChatRequest, MilaChatService, PreparedChatTurn};

use tracing::debug;

impl MilaChatService {
    pub(super) async fn prepare_turn(
        &self,
        session: &MilaSession,
        config: &MilaConfig,
        history: &[MilaMessage],
        request: &MilaChatRequest,
    ) -> Result<PreparedChatTurn, AppError> {
        let history_messages = history
            .iter()
            .map(|message| ChatMessage::new(message.role, message.content.clone()))
            .collect::<Vec<_>>();
        let provider = chat_provider_from_config(config, self.credential_cipher.as_deref())?;
        let trimmed_question = request.question.trim();
        let user_message = MilaMessage {
            id: MilaMessageId::new(),
            session_id: session.id,
            role: MessageRole::User,
            content: trimmed_question.to_string(),
            source_chunks: Vec::new(),
            created_at: Utc::now(),
        };

        let chat_system_prompt = self.resolve_chat_system_prompt(request.user_id).await?;

        let (messages, source_chunk_ids, source_label_count, retrieval_degraded) = match session
            .session_type
        {
            MilaSessionType::SingleDocument => {
                let document_id = session.document_id.ok_or_else(|| {
                    AppError::Domain(DomainError::InvariantViolation {
                        message: format!(
                            "single_document session {} is missing document_id",
                            session.id
                        ),
                    })
                })?;
                let (messages, source_chunk_ids, source_label_count) = self
                    .prepare_single_document_messages(
                        &chat_system_prompt,
                        request.user_id,
                        document_id,
                        config,
                        request,
                        &history_messages,
                    )
                    .await?;
                (messages, source_chunk_ids, source_label_count, None)
            }
            MilaSessionType::CrossItem => {
                if request.highlight_text.is_some() {
                    return Err(AppError::Domain(DomainError::Validation {
                        field: "highlight_text".into(),
                        message: "highlight chat is only supported for single-document sessions"
                            .into(),
                    }));
                }
                let (messages, source_chunk_ids, source_label_count) = self
                    .prepare_cross_item_messages(
                        &chat_system_prompt,
                        config,
                        trimmed_question,
                        &history_messages,
                    )
                    .await?;
                (messages, source_chunk_ids, source_label_count, None)
            }
            MilaSessionType::Collection => {
                if request.highlight_text.is_some() {
                    return Err(AppError::Domain(DomainError::Validation {
                        field: "highlight_text".into(),
                        message: "highlight chat is only supported for single-document sessions"
                            .into(),
                    }));
                }
                let collection_id = session.collection_id.ok_or_else(|| {
                    AppError::Domain(DomainError::InvariantViolation {
                        message: format!(
                            "collection session {} is missing collection_id",
                            session.id
                        ),
                    })
                })?;
                self.prepare_collection_messages(
                    &chat_system_prompt,
                    request.user_id,
                    collection_id,
                    config,
                    trimmed_question,
                    &history_messages,
                )
                .await?
            }
        };

        Ok(PreparedChatTurn {
            provider,
            completion_request: build_chat_request(config, request.user_id, messages),
            user_message,
            source_chunk_ids,
            source_label_count,
            retrieval_degraded,
            session_id: session.id,
            user_id: request.user_id,
        })
    }

    async fn prepare_single_document_messages(
        &self,
        system_prompt: &str,
        user_id: UserId,
        document_id: DocumentId,
        config: &MilaConfig,
        request: &MilaChatRequest,
        history: &[ChatMessage],
    ) -> Result<(Vec<ChatMessage>, Vec<Uuid>, usize), AppError> {
        let document = self.load_document(user_id, document_id).await?;
        let plain_text = self.load_document_plain_text(document_id).await?;
        let metadata = format_document_metadata(&document);
        self.prepare_single_content_messages(
            system_prompt,
            user_id,
            document_id,
            metadata,
            plain_text,
            config,
            request,
            history,
        )
        .await
    }

    /// Shared single-document chat preparation. `document_id` is the durable retrieval scope;
    /// `metadata` is the pre-formatted source context block for the prompt.
    #[allow(clippy::too_many_arguments)]
    async fn prepare_single_content_messages(
        &self,
        system_prompt: &str,
        user_id: UserId,
        document_id: DocumentId,
        metadata: String,
        plain_text: String,
        config: &MilaConfig,
        request: &MilaChatRequest,
        history: &[ChatMessage],
    ) -> Result<(Vec<ChatMessage>, Vec<Uuid>, usize), AppError> {
        let inline_budget =
            chat_inline_token_budget(config.model_context_window, config.chat_context_pct);
        let uses_rag = exceeds_context_threshold(&plain_text, inline_budget);
        debug!(
            uses_rag,
            plain_text_chars = plain_text.len(),
            inline_budget_tokens = inline_budget,
            model_context_window = config.model_context_window,
            chat_context_pct = config.chat_context_pct,
            has_highlight = request.highlight_text.is_some(),
            "single-content context strategy"
        );

        if let Some(highlight_text) = request.highlight_text.as_deref() {
            let retrieval_query = highlight_text.trim();
            let window_text = extract_text_window(
                &plain_text,
                retrieval_query,
                request.highlight_offset,
                chars_for_tokens(HIGHLIGHT_WINDOW_TOKENS),
            );

            if uses_rag {
                let hits = self
                    .search_single_document_hybrid(
                        config,
                        user_id,
                        document_id,
                        retrieval_query,
                        i64::from(config.top_k.max(1)),
                    )
                    .await?;
                let passages = self.passages_from_hits_with_parent_context(&hits).await?;
                let source_chunk_ids = collect_source_chunk_ids(&hits);
                let source_label_count = distinct_source_label_count(&hits);
                Ok((
                    build_highlight_messages(
                        system_prompt,
                        &metadata,
                        retrieval_query,
                        &window_text,
                        None,
                        &passages,
                        history,
                        request.question.trim(),
                    ),
                    source_chunk_ids,
                    source_label_count,
                ))
            } else {
                Ok((
                    build_highlight_messages(
                        system_prompt,
                        &metadata,
                        retrieval_query,
                        &window_text,
                        Some(&plain_text),
                        &[],
                        history,
                        request.question.trim(),
                    ),
                    Vec::new(),
                    0,
                ))
            }
        } else if uses_rag {
            let hits = self
                .search_single_document_hybrid(
                    config,
                    user_id,
                    document_id,
                    request.question.trim(),
                    i64::from(config.top_k.max(1)),
                )
                .await?;
            let passages = self.passages_from_hits_with_parent_context(&hits).await?;
            let source_chunk_ids = collect_source_chunk_ids(&hits);
            let source_label_count = distinct_source_label_count(&hits);
            Ok((
                build_single_document_rag_messages(
                    system_prompt,
                    &metadata,
                    &passages,
                    history,
                    request.question.trim(),
                ),
                source_chunk_ids,
                source_label_count,
            ))
        } else {
            Ok((
                build_single_document_stuffing_messages(
                    system_prompt,
                    &metadata,
                    &plain_text,
                    history,
                    request.question.trim(),
                ),
                Vec::new(),
                0,
            ))
        }
    }

    async fn prepare_cross_item_messages(
        &self,
        system_prompt: &str,
        config: &MilaConfig,
        question: &str,
        history: &[ChatMessage],
    ) -> Result<(Vec<ChatMessage>, Vec<Uuid>, usize), AppError> {
        let hits = self.search_cross_item_hybrid(config, question).await?;
        debug!(
            hit_count = hits.len(),
            top_k = config.cross_item_top_k,
            max_per_item = config.cross_item_max_per_item,
            scores = ?hits.iter().map(|h| (h.score * 1000.0).round() / 1000.0).collect::<Vec<_>>(),
            document_ids = ?hits.iter().map(|h| h.document_id).collect::<Vec<_>>(),
            "cross-item hybrid retrieval"
        );
        let passages = self.passages_from_hits_with_parent_context(&hits).await?;
        let source_chunk_ids = collect_source_chunk_ids(&hits);
        let source_label_count = distinct_source_label_count(&hits);
        Ok((
            build_cross_item_messages(system_prompt, "your library", &passages, history, question),
            source_chunk_ids,
            source_label_count,
        ))
    }

    async fn prepare_collection_messages(
        &self,
        system_prompt: &str,
        user_id: UserId,
        collection_id: ind_domain::CollectionId,
        config: &MilaConfig,
        question: &str,
        history: &[ChatMessage],
    ) -> Result<(Vec<ChatMessage>, Vec<Uuid>, usize, Option<String>), AppError> {
        let CollectionHybridSearchOutcome {
            hits,
            retrieval_degraded,
        } = self
            .search_collection_hybrid(user_id, collection_id, config, question)
            .await?;
        debug!(
            hit_count = hits.len(),
            top_k = config.cross_item_top_k,
            max_per_item = config.cross_item_max_per_item,
            scores = ?hits.iter().map(|h| (h.score * 1000.0).round() / 1000.0).collect::<Vec<_>>(),
            document_ids = ?hits.iter().map(|h| h.document_id).collect::<Vec<_>>(),
            "collection hybrid RAG retrieval"
        );
        let passages = self.passages_from_hits_with_parent_context(&hits).await?;
        let source_chunk_ids = collect_source_chunk_ids(&hits);
        let source_label_count = distinct_source_label_count(&hits);
        Ok((
            build_cross_item_messages(
                system_prompt,
                "the selected collection",
                &passages,
                history,
                question,
            ),
            source_chunk_ids,
            source_label_count,
            retrieval_degraded,
        ))
    }
}
