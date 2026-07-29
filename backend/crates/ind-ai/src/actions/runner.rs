use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;

use ind_application::AppError;
use ind_application::event_intents::ai_output_completed;
use ind_application::repos::ai_output::AiOutputRepository;
use ind_application::repos::ai_preset::AiPromptPresetRepository;
use ind_application::repos::ai_run::AiRunRepository;
use ind_application::repos::document::DocumentRepository;
use ind_application::repos::entity::EntityRepository;
use ind_application::repos::event::MutationSideEffects;
use ind_application::repos::mila_config::MilaConfigRepository;
use ind_application::repos::prepared_content::PreparedContentProvider;
use ind_auth::CredentialCipher;
use ind_domain::{
    AiOutput, AiOutputId, AiOutputType, AiPromptAction, AiPromptPreset, AiRun, AiRunId, Document,
    DocumentId, DomainError, MilaConfig, PreparedItemContent, UserId,
};

use crate::AiProviderClient;
use crate::resolution::{EntityRepositoryAdapter, EntityResolutionStore, EntityResolver};

use super::parse::{parse_entities_output, parse_summary_output, parse_tags_output};
use super::prompt::build_document_user_prompt;

pub struct AiActionRunner {
    pub(super) store: Arc<dyn ActionStore>,
    entity_store: Arc<dyn EntityResolutionStore>,
    pub(super) ai_client: Arc<dyn AiProviderClient>,
    pub(super) credential_cipher: Option<Arc<CredentialCipher>>,
}

#[async_trait]
pub(super) trait ActionStore: Send + Sync {
    async fn find_document(&self, document_id: DocumentId) -> Result<Option<Document>, AppError>;
    async fn load_content(
        &self,
        document_id: DocumentId,
    ) -> Result<Option<PreparedItemContent>, AppError>;
    async fn load_readable(&self, document_id: DocumentId) -> Result<Option<String>, AppError>;
    async fn mila_config(&self, user_id: UserId) -> Result<Option<MilaConfig>, AppError>;
    async fn default_preset(
        &self,
        user_id: UserId,
        action: AiPromptAction,
    ) -> Result<Option<AiPromptPreset>, AppError>;
    async fn system_preset(
        &self,
        action: AiPromptAction,
    ) -> Result<Option<AiPromptPreset>, AppError>;
    async fn upsert_output(
        &self,
        output: &AiOutput,
        effects: MutationSideEffects,
    ) -> Result<AiOutput, AppError>;
    async fn create_run(&self, run: &AiRun) -> Result<AiRun, AppError>;
    async fn complete_run(
        &self,
        run_id: AiRunId,
        input_tokens: Option<i32>,
        output_tokens: Option<i32>,
    ) -> Result<(), AppError>;
    async fn fail_run(
        &self,
        run_id: AiRunId,
        message: String,
        effects: MutationSideEffects,
    ) -> Result<(), AppError>;
}

struct ApplicationActionStore {
    documents: Arc<dyn DocumentRepository>,
    content: Arc<dyn PreparedContentProvider>,
    configs: Arc<dyn MilaConfigRepository>,
    presets: Arc<dyn AiPromptPresetRepository>,
    outputs: Arc<dyn AiOutputRepository>,
    runs: Arc<dyn AiRunRepository>,
}

#[async_trait]
impl ActionStore for ApplicationActionStore {
    async fn find_document(&self, id: DocumentId) -> Result<Option<Document>, AppError> {
        self.documents.find_by_id_global(id).await
    }
    async fn load_content(&self, id: DocumentId) -> Result<Option<PreparedItemContent>, AppError> {
        self.content.load_for_document(id).await
    }
    async fn load_readable(&self, id: DocumentId) -> Result<Option<String>, AppError> {
        self.content.load_readable_text_for_document(id).await
    }
    async fn mila_config(&self, user: UserId) -> Result<Option<MilaConfig>, AppError> {
        self.configs.get_by_user(user).await
    }
    async fn default_preset(
        &self,
        user: UserId,
        action: AiPromptAction,
    ) -> Result<Option<AiPromptPreset>, AppError> {
        self.presets.find_default_for_action(user, action).await
    }
    async fn system_preset(
        &self,
        action: AiPromptAction,
    ) -> Result<Option<AiPromptPreset>, AppError> {
        self.presets.find_system_preset_for_action(action).await
    }
    async fn upsert_output(
        &self,
        output: &AiOutput,
        effects: MutationSideEffects,
    ) -> Result<AiOutput, AppError> {
        self.outputs.upsert(output, effects).await
    }
    async fn create_run(&self, run: &AiRun) -> Result<AiRun, AppError> {
        self.runs.create(run).await
    }
    async fn complete_run(
        &self,
        id: AiRunId,
        input: Option<i32>,
        output: Option<i32>,
    ) -> Result<(), AppError> {
        self.runs
            .mark_completed(id, input, output, Utc::now())
            .await
    }
    async fn fail_run(
        &self,
        id: AiRunId,
        message: String,
        effects: MutationSideEffects,
    ) -> Result<(), AppError> {
        self.runs
            .mark_failed(id, message, effects, Utc::now())
            .await
    }
}

pub(super) struct PreparedAction {
    pub(super) user_id: UserId,
    pub(super) config: MilaConfig,
    pub(super) system_prompt: String,
    pub(super) user_prompt: String,
    pub(super) document_title: String,
}

#[derive(Clone, Copy)]
pub(super) struct ActionTarget(DocumentId);

impl ActionTarget {
    pub(super) fn run_document_id(self) -> DocumentId {
        self.0
    }

    fn output_document_id(self) -> Option<DocumentId> {
        Some(self.0)
    }
}

impl AiActionRunner {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        document_repo: Arc<dyn DocumentRepository>,
        content_provider: Arc<dyn PreparedContentProvider>,
        mila_config_repo: Arc<dyn MilaConfigRepository>,
        ai_preset_repo: Arc<dyn AiPromptPresetRepository>,
        ai_output_repo: Arc<dyn AiOutputRepository>,
        ai_run_repo: Arc<dyn AiRunRepository>,
        entity_repo: Arc<dyn EntityRepository>,
        ai_client: Arc<dyn AiProviderClient>,
    ) -> Self {
        let store = Arc::new(ApplicationActionStore {
            documents: document_repo,
            content: content_provider,
            configs: mila_config_repo,
            presets: ai_preset_repo,
            outputs: ai_output_repo,
            runs: ai_run_repo,
        });
        Self {
            store,
            entity_store: Arc::new(EntityRepositoryAdapter(entity_repo)),
            ai_client,
            credential_cipher: None,
        }
    }

    pub fn with_credential_cipher(mut self, cipher: Option<Arc<CredentialCipher>>) -> Self {
        self.credential_cipher = cipher;
        self
    }

    pub async fn can_process_document(&self, document_id: DocumentId) -> Result<bool, AppError> {
        let prepared = self.store.load_content(document_id).await?;
        let user_id = match &prepared {
            Some(prepared) if !prepared.root_text.trim().is_empty() => prepared.user_id,
            Some(_) => return Ok(false),
            None => match self.store.load_readable(document_id).await? {
                Some(text) if !text.trim().is_empty() => self.document_user(document_id).await?,
                _ => return Ok(false),
            },
        };

        let Some(config) = self.store.mila_config(user_id).await? else {
            return Ok(false);
        };

        Ok(config.enabled)
    }

    pub async fn summarize_document(&self, document_id: DocumentId) -> Result<(), AppError> {
        let Some(prepared) = self
            .prepare_document_action(document_id, AiPromptAction::Summary)
            .await?
        else {
            return Ok(());
        };
        self.run_summary(&prepared, ActionTarget(document_id)).await
    }

    pub async fn suggest_tags_for_document(&self, document_id: DocumentId) -> Result<(), AppError> {
        let Some(prepared) = self
            .prepare_document_action(document_id, AiPromptAction::Tags)
            .await?
        else {
            return Ok(());
        };
        self.run_tags(&prepared, ActionTarget(document_id)).await
    }

    pub async fn extract_entities_for_document(
        &self,
        document_id: DocumentId,
    ) -> Result<(), AppError> {
        let Some(prepared) = self
            .prepare_document_action(document_id, AiPromptAction::Entities)
            .await?
        else {
            return Ok(());
        };
        self.run_entities(&prepared, ActionTarget(document_id))
            .await
    }

    async fn run_summary(
        &self,
        prepared: &PreparedAction,
        target: ActionTarget,
    ) -> Result<(), AppError> {
        let run = self
            .start_run(prepared, target, AiPromptAction::Summary)
            .await?;
        let model = match self
            .run_model_completion(prepared, AiPromptAction::Summary)
            .await
        {
            Ok(model) => model,
            Err(err) => {
                self.mark_run_failed(run.id, prepared, target, AiPromptAction::Summary, &err)
                    .await?;
                return Err(err);
            }
        };
        let result = async {
            let summary = parse_summary_output(&model.raw_content)?;
            let output = AiOutput {
                id: AiOutputId::new(),
                document_id: target.output_document_id(),
                output_type: AiOutputType::Summary,
                content: serde_json::Value::String(summary),
                ai_run_id: Some(run.id),
                created_at: Utc::now(),
            };
            self.store
                .upsert_output(
                    &output,
                    MutationSideEffects::with_event(ai_output_completed(
                        prepared.user_id,
                        target.run_document_id(),
                        AiOutputType::Summary,
                        run.id,
                    )),
                )
                .await?;
            Ok(())
        }
        .await;
        self.finish_run(
            run.id,
            prepared,
            target,
            AiPromptAction::Summary,
            &model,
            result,
        )
        .await
    }

    async fn run_tags(
        &self,
        prepared: &PreparedAction,
        target: ActionTarget,
    ) -> Result<(), AppError> {
        let run = self
            .start_run(prepared, target, AiPromptAction::Tags)
            .await?;
        let model = match self
            .run_model_completion(prepared, AiPromptAction::Tags)
            .await
        {
            Ok(model) => model,
            Err(err) => {
                self.mark_run_failed(run.id, prepared, target, AiPromptAction::Tags, &err)
                    .await?;
                return Err(err);
            }
        };
        let result = async {
            let tags = parse_tags_output(&model.raw_content)?;
            let output = AiOutput {
                id: AiOutputId::new(),
                document_id: target.output_document_id(),
                output_type: AiOutputType::Tags,
                content: serde_json::to_value(&tags)
                    .map_err(|err| AppError::Repository(Box::new(err)))?,
                ai_run_id: Some(run.id),
                created_at: Utc::now(),
            };
            self.store
                .upsert_output(
                    &output,
                    MutationSideEffects::with_event(ai_output_completed(
                        prepared.user_id,
                        target.run_document_id(),
                        AiOutputType::Tags,
                        run.id,
                    )),
                )
                .await?;
            Ok(())
        }
        .await;
        self.finish_run(
            run.id,
            prepared,
            target,
            AiPromptAction::Tags,
            &model,
            result,
        )
        .await
    }

    async fn run_entities(
        &self,
        prepared: &PreparedAction,
        target: ActionTarget,
    ) -> Result<(), AppError> {
        let run = self
            .start_run(prepared, target, AiPromptAction::Entities)
            .await?;
        let model = match self
            .run_model_completion(prepared, AiPromptAction::Entities)
            .await
        {
            Ok(model) => model,
            Err(err) => {
                self.mark_run_failed(run.id, prepared, target, AiPromptAction::Entities, &err)
                    .await?;
                return Err(err);
            }
        };
        let result = async {
            let entities = parse_entities_output(&model.raw_content)?;
            let resolver =
                EntityResolver::with_store(self.entity_store.clone(), self.ai_client.clone())
                    .with_credential_cipher(self.credential_cipher.clone());
            let resolved = resolver
                .resolve_document_entities(
                    prepared.user_id,
                    &prepared.config,
                    &prepared.document_title,
                    &entities,
                )
                .await?;
            self.entity_store
                .set_document_mentions(prepared.user_id, target.run_document_id(), &resolved)
                .await?;
            let output = AiOutput {
                id: AiOutputId::new(),
                document_id: target.output_document_id(),
                output_type: AiOutputType::Entities,
                content: serde_json::to_value(&entities)
                    .map_err(|err| AppError::Repository(Box::new(err)))?,
                ai_run_id: Some(run.id),
                created_at: Utc::now(),
            };
            self.store
                .upsert_output(
                    &output,
                    MutationSideEffects::with_event(ai_output_completed(
                        prepared.user_id,
                        target.run_document_id(),
                        AiOutputType::Entities,
                        run.id,
                    )),
                )
                .await?;
            Ok(())
        }
        .await;
        self.finish_run(
            run.id,
            prepared,
            target,
            AiPromptAction::Entities,
            &model,
            result,
        )
        .await
    }

    async fn prepare_document_action(
        &self,
        document_id: DocumentId,
        action: AiPromptAction,
    ) -> Result<Option<PreparedAction>, AppError> {
        let Some(document) = self.store.find_document(document_id).await? else {
            return Ok(None);
        };

        // Prefer structured prepared content via the id-reuse bridge; net-new feed-prepared
        // documents (no legacy item) fall back to document-addressable readable text.
        let plain_text = match self.store.load_content(document_id).await? {
            Some(prepared) if !prepared.root_text.trim().is_empty() => prepared.root_text,
            _ => match self.store.load_readable(document_id).await? {
                Some(text) if !text.trim().is_empty() => text,
                _ => return Ok(None),
            },
        };

        let Some(config) = self.store.mila_config(document.user_id).await? else {
            return Ok(None);
        };
        if !config.enabled {
            return Ok(None);
        }

        let system_prompt = self.resolve_system_prompt(document.user_id, action).await?;
        let input_budget =
            super::budget::action_input_budget(config.model_context_window, &system_prompt, action);
        let user_prompt = build_document_user_prompt(&document, &plain_text, input_budget);

        Ok(Some(PreparedAction {
            user_id: document.user_id,
            config,
            system_prompt,
            user_prompt,
            document_title: document.title,
        }))
    }

    async fn document_user(&self, document_id: DocumentId) -> Result<UserId, AppError> {
        self.store
            .find_document(document_id)
            .await?
            .map(|document| document.user_id)
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "document",
                    id: document_id.to_string(),
                })
            })
    }

    async fn resolve_system_prompt(
        &self,
        user_id: UserId,
        action: AiPromptAction,
    ) -> Result<String, AppError> {
        if let Some(preset) = self.store.default_preset(user_id, action).await?
            && !preset.system_prompt.trim().is_empty()
        {
            return Ok(preset.system_prompt);
        }
        let system_preset = self.store.system_preset(action).await?.ok_or_else(|| {
            AppError::Domain(DomainError::InvariantViolation {
                message: format!("no system preset found for action {:?}", action),
            })
        })?;
        Ok(system_preset.system_prompt)
    }

    async fn start_run(
        &self,
        prepared: &PreparedAction,
        target: ActionTarget,
        action: AiPromptAction,
    ) -> Result<AiRun, AppError> {
        let run = AiRun {
            id: AiRunId::new(),
            user_id: prepared.user_id,
            document_id: Some(target.run_document_id()),
            action,
            provider: prepared.config.chat_api_base.clone(),
            model: prepared.config.chat_model.clone(),
            input_tokens: None,
            output_tokens: None,
            is_byok: prepared
                .config
                .chat_api_key_enc
                .as_ref()
                .is_some_and(|value| !value.is_empty()),
            status: "running".into(),
            error_message: None,
            started_at: Utc::now(),
            completed_at: None,
        };

        self.store.create_run(&run).await
    }
}
