#[async_trait]
impl ChatDocuments for ChatHarness {
    async fn find_by_id(&self, user: UserId, id: DocumentId) -> Result<Option<Document>, AppError> {
        Ok((user == self.user_id && id == self.document.id).then(|| self.document.clone()))
    }
}

#[async_trait]
impl ChatContent for ChatHarness {
    async fn load_for_document(
        &self,
        _: DocumentId,
    ) -> Result<Option<PreparedItemContent>, AppError> {
        Ok(Some(self.prepared.lock().unwrap().clone()))
    }
    async fn load_readable_text_for_document(
        &self,
        _: DocumentId,
    ) -> Result<Option<String>, AppError> {
        Ok(None)
    }
}

#[async_trait]
impl ChatConfig for ChatHarness {
    async fn get_by_user(&self, _: UserId) -> Result<Option<MilaConfig>, AppError> {
        Ok(Some(self.config.lock().unwrap().clone()))
    }
}

#[async_trait]
impl ChatPresets for ChatHarness {
    async fn find_default_for_action(
        &self,
        _: UserId,
        _: AiPromptAction,
    ) -> Result<Option<AiPromptPreset>, AppError> {
        Ok(None)
    }
    async fn find_system_preset_for_action(
        &self,
        action: AiPromptAction,
    ) -> Result<Option<AiPromptPreset>, AppError> {
        Ok(Some(AiPromptPreset {
            id: ind_domain::AiPromptPresetId::new(),
            user_id: None,
            name: "System".into(),
            action,
            system_prompt: "Use sources".into(),
            is_default: false,
            is_system: true,
            created_at: Utc::now(),
        }))
    }
}

fn retrieval_failure() -> AppError {
    AppError::Domain(DomainError::InvariantViolation {
        message: "scripted retrieval failure".into(),
    })
}

#[async_trait]
impl ChatRetrieval for ChatHarness {
    async fn search_single_document(
        &self,
        _: &SingleDocumentVectorQuery,
    ) -> Result<Vec<SearchHit>, AppError> {
        if *self.fail_vector.lock().unwrap() {
            Err(retrieval_failure())
        } else {
            Ok(self.vector_hits.lock().unwrap().clone())
        }
    }
    async fn search_cross_document(
        &self,
        _: &CrossDocumentVectorQuery,
    ) -> Result<Vec<SearchHit>, AppError> {
        if *self.fail_vector.lock().unwrap() {
            Err(retrieval_failure())
        } else {
            Ok(self.vector_hits.lock().unwrap().clone())
        }
    }
    async fn search_collection_document(
        &self,
        query: &CollectionDocumentVectorQuery,
    ) -> Result<Vec<SearchHit>, AppError> {
        self.collection_vectors.lock().unwrap().push(query.clone());
        if *self.fail_vector.lock().unwrap() {
            Err(retrieval_failure())
        } else {
            Ok(self.vector_hits.lock().unwrap().clone())
        }
    }
    async fn fts_single_document(
        &self,
        _: &SingleDocumentFtsQuery,
    ) -> Result<Vec<SearchHit>, AppError> {
        if *self.fail_fts.lock().unwrap() {
            Err(retrieval_failure())
        } else {
            Ok(self.fts_hits.lock().unwrap().clone())
        }
    }
    async fn fts_cross_document(
        &self,
        _: &CrossDocumentFtsQuery,
    ) -> Result<Vec<SearchHit>, AppError> {
        if *self.fail_fts.lock().unwrap() {
            Err(retrieval_failure())
        } else {
            Ok(self.fts_hits.lock().unwrap().clone())
        }
    }
    async fn fts_collection_document(
        &self,
        query: &CollectionDocumentFtsQuery,
    ) -> Result<Vec<SearchHit>, AppError> {
        self.collection_fts.lock().unwrap().push(query.clone());
        if *self.fail_fts.lock().unwrap() {
            Err(retrieval_failure())
        } else {
            Ok(self.fts_hits.lock().unwrap().clone())
        }
    }
}

#[async_trait]
impl ChatSessions for ChatHarness {
    async fn find_session_for_user(
        &self,
        id: MilaSessionId,
        user: UserId,
    ) -> Result<Option<MilaSession>, AppError> {
        let session = self.session.lock().unwrap().clone();
        Ok((session.id == id && session.user_id == user).then_some(session))
    }
    async fn insert_message(
        &self,
        _: UserId,
        message: &MilaMessage,
    ) -> Result<MilaMessage, AppError> {
        self.messages.lock().unwrap().push(message.clone());
        Ok(message.clone())
    }
    async fn list_messages(
        &self,
        _: MilaSessionId,
        _: UserId,
    ) -> Result<Vec<MilaMessage>, AppError> {
        Ok(Vec::new())
    }
    async fn touch_session(
        &self,
        _: MilaSessionId,
        _: UserId,
        _: chrono::DateTime<Utc>,
    ) -> Result<(), AppError> {
        Ok(())
    }
}
