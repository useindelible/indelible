use super::helpers::*;
use super::*;

impl MilaSessionPort for MilaOperationsService {
    fn list_sessions(
        &self,
        user_id: UserId,
        limit: i64,
    ) -> BoxFuture<'_, Result<Vec<MilaSessionWithPreviewOutput>, AppError>> {
        Box::pin(async move {
            let sessions = self
                .mila_session_repo
                .list_sessions_for_user(user_id, limit)
                .await?;
            Ok(sessions
                .into_iter()
                .map(|s| MilaSessionWithPreviewOutput {
                    id: s.session.id,
                    session_type: s.session.session_type,
                    document_id: s.session.document_id,
                    collection_id: s.session.collection_id,
                    created_at: s.session.created_at,
                    last_active: s.session.last_active,
                    preview_content: s.preview_content,
                    preview_role: s.preview_role.map(|r| match r {
                        ind_domain::MessageRole::User => "user".to_string(),
                        ind_domain::MessageRole::Assistant => "assistant".to_string(),
                        ind_domain::MessageRole::System => "system".to_string(),
                    }),
                })
                .collect())
        })
    }

    fn create_session(
        &self,
        user_id: UserId,
        request: CreateMilaSessionRequest,
    ) -> BoxFuture<'_, Result<MilaSessionOutput, AppError>> {
        Box::pin(async move {
            ensure_mila_enabled(&self.service, user_id).await?;

            // Single-document chat is an engagement: it materializes/back-links the document,
            // retains it, inserts the session, and resolves content-gated AI work in ONE
            // transaction owned by the MilaSessionService -> DocumentLifecycle (TASK-234).
            if request.session_type == MilaSessionType::SingleDocument {
                let target = chat_target_from_request(&request)?;
                let outcome = self
                    .mila_session_service
                    .start_single_document_chat(user_id, target)
                    .await?;
                let provenance = self
                    .mila_session_service
                    .load_provenance(user_id, outcome.document.id)
                    .await?;
                let mut output = mila_session_output(outcome.session);
                output.provenance = provenance;
                return Ok(output);
            }

            match request.session_type {
                MilaSessionType::SingleDocument => {
                    return Err(AppError::Domain(
                        ind_domain::DomainError::InvariantViolation {
                            message: "single-document Mila session reached multi-document dispatch"
                                .into(),
                        },
                    ));
                }
                MilaSessionType::CrossItem => {}
                MilaSessionType::Collection => {
                    let collection_id = request.collection_id.ok_or_else(|| {
                        AppError::Domain(ind_domain::DomainError::Validation {
                            field: "collection_id".into(),
                            message: "collection sessions require collection_id".into(),
                        })
                    })?;
                    ensure_collection_owned(self.collection_repo.as_ref(), user_id, collection_id)
                        .await?;
                }
            }

            let now = chrono::Utc::now();
            let session = self
                .mila_session_repo
                .create_session(&MilaSession {
                    id: MilaSessionId::new(),
                    user_id,
                    document_id: request.document_id,
                    collection_id: request.collection_id,
                    session_type: request.session_type,
                    created_at: now,
                    last_active: now,
                })
                .await?;

            Ok(mila_session_output(session))
        })
    }

    fn get_session_messages(
        &self,
        user_id: UserId,
        session_id: MilaSessionId,
    ) -> BoxFuture<'_, Result<MilaConversationOutput, AppError>> {
        Box::pin(async move {
            let session = self
                .mila_session_repo
                .find_session_for_user(session_id, user_id)
                .await?
                .ok_or_else(|| {
                    AppError::Domain(ind_domain::DomainError::NotFound {
                        entity: "mila_session",
                        id: session_id.to_string(),
                    })
                })?;
            let messages = self
                .mila_session_repo
                .list_messages(session_id, user_id)
                .await?;

            let all_chunk_ids: Vec<uuid::Uuid> = messages
                .iter()
                .flat_map(|m| m.source_chunks.iter().copied())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            let ref_map: std::collections::HashMap<uuid::Uuid, (DocumentId, String)> =
                resolve_source_refs(self.content_vector_repo.as_ref(), &all_chunk_ids).await?;

            let mut session_output = mila_session_output(session);
            if let Some(document_id) = session_output.document_id {
                session_output.provenance = self
                    .mila_session_service
                    .load_provenance(user_id, document_id)
                    .await?;
            }

            Ok(MilaConversationOutput {
                session: session_output,
                messages: messages
                    .into_iter()
                    .map(|m| mila_message_output(m, &ref_map))
                    .collect(),
            })
        })
    }

    fn delete_session(
        &self,
        user_id: UserId,
        session_id: MilaSessionId,
    ) -> BoxFuture<'_, Result<(), AppError>> {
        Box::pin(async move {
            self.mila_session_repo
                .delete_session(session_id, user_id)
                .await
        })
    }
}
