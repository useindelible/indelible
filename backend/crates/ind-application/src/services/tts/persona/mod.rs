use std::sync::Arc;

use chrono::Utc;
use ind_domain::{
    DomainError, TtsPersonaStatus, TtsProvider, TtsVoicePersona, TtsVoicePersonaId, UserId,
};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::adapter_error;
use super::credentials::TtsProviderCredentialResolver;
use super::entitlements::TtsEntitlements;
use crate::AppError;
use crate::ports::{TtsAdapter, TtsDesignRequest};
use crate::repos::tts_voice_persona::TtsVoicePersonaRepository;

pub type PersonaAdapterResolver =
    Arc<dyn Fn(TtsProvider) -> Option<Arc<dyn TtsAdapter>> + Send + Sync>;

#[derive(Debug, Clone)]
pub struct CreatePersonaInput {
    pub display_name: String,
    pub description: Option<String>,
    pub provider: TtsProvider,
    pub provider_voice_id: Option<String>,
    pub provider_model: Option<String>,
    pub design_prompt: Option<String>,
    pub style_prompt: Option<String>,
    pub pace: Option<String>,
    pub energy: Option<String>,
    pub warmth: Option<String>,
    pub formality: Option<String>,
    pub pronunciation_prefs: serde_json::Value,
}

pub struct PersonaService {
    repo: Arc<dyn TtsVoicePersonaRepository>,
    entitlements: Option<Arc<TtsEntitlements>>,
    adapters: Option<PersonaAdapterResolver>,
    credentials: Option<Arc<dyn TtsProviderCredentialResolver>>,
}

impl PersonaService {
    pub fn new(repo: Arc<dyn TtsVoicePersonaRepository>) -> Self {
        Self {
            repo,
            entitlements: None,
            adapters: None,
            credentials: None,
        }
    }

    pub fn with_entitlements(mut self, entitlements: Arc<TtsEntitlements>) -> Self {
        self.entitlements = Some(entitlements);
        self
    }

    pub fn with_adapters(mut self, adapters: PersonaAdapterResolver) -> Self {
        self.adapters = Some(adapters);
        self
    }

    pub fn with_credentials(mut self, credentials: Arc<dyn TtsProviderCredentialResolver>) -> Self {
        self.credentials = Some(credentials);
        self
    }

    pub async fn list(&self, user_id: UserId) -> Result<Vec<TtsVoicePersona>, AppError> {
        self.repo.list_for_user(user_id).await
    }

    pub async fn get(
        &self,
        user_id: UserId,
        id: TtsVoicePersonaId,
    ) -> Result<TtsVoicePersona, AppError> {
        self.repo
            .get(id, user_id)
            .await?
            .ok_or_else(|| not_found(id))
    }

    pub async fn create(
        &self,
        user_id: UserId,
        input: CreatePersonaInput,
    ) -> Result<TtsVoicePersona, AppError> {
        if let Some(entitlements) = self.entitlements.as_ref() {
            entitlements.authorize_persona_creation(input.provider)?;
        }

        let display_name = trimmed_required("display_name", &input.display_name)?;
        let now = Utc::now();
        let prompt_hash = prompt_hash(
            input.design_prompt.as_deref(),
            input.style_prompt.as_deref(),
        );

        let wants_provider_design = input.provider_voice_id.is_none()
            && (input.design_prompt.is_some() || input.style_prompt.is_some());

        let draft = TtsVoicePersona {
            id: TtsVoicePersonaId::from_uuid(Uuid::now_v7()),
            user_id: Some(user_id),
            display_name,
            description: input.description.clone(),
            provider: input.provider,
            provider_voice_id: input.provider_voice_id.clone(),
            provider_model: input.provider_model.clone(),
            design_prompt: input.design_prompt.clone(),
            style_prompt: input.style_prompt.clone(),
            pace: input.pace.clone(),
            energy: input.energy.clone(),
            warmth: input.warmth.clone(),
            formality: input.formality.clone(),
            pronunciation_prefs: input.pronunciation_prefs.clone(),
            status: TtsPersonaStatus::Active,
            is_builtin: false,
            prompt_hash,
            created_at: now,
            updated_at: now,
        };

        if !wants_provider_design {
            return self.repo.insert(&draft).await;
        }

        let adapter = self
            .adapters
            .as_ref()
            .and_then(|resolve| resolve(input.provider))
            .ok_or_else(|| {
                AppError::Domain(DomainError::Validation {
                    field: "provider".into(),
                    message: format!(
                        "no adapter registered for provider {}",
                        input.provider.as_str()
                    ),
                })
            })?;

        if !adapter.supports_voice_design() {
            return Err(AppError::Domain(DomainError::Validation {
                field: "design_prompt".into(),
                message: format!(
                    "{} does not support provider-side voice design",
                    input.provider.as_str()
                ),
            }));
        }

        let credentials = self
            .credentials
            .as_ref()
            .ok_or_else(|| AppError::ExternalService {
                service: "tts".into(),
                message: "tts credential resolver is not configured".into(),
            })?
            .resolve(input.provider)
            .await?;

        let design_prompt_text = input.design_prompt.as_deref().unwrap_or("");
        let design = adapter
            .design_voice(TtsDesignRequest {
                persona: &draft,
                design_prompt: design_prompt_text,
                style_prompt: input.style_prompt.as_deref(),
                api_key: credentials.api_key.as_deref(),
                api_base: credentials.api_base.as_deref(),
            })
            .await
            .map_err(adapter_error)?;

        let provider_voice_id =
            design
                .provider_voice_id
                .ok_or_else(|| AppError::ExternalService {
                    service: "tts".into(),
                    message: "provider did not return voice id".into(),
                })?;

        let mut persona = draft;
        persona.provider_voice_id = Some(provider_voice_id);
        if let Some(design_model) = design.provider_model {
            persona.provider_model = Some(design_model);
        }
        self.repo.insert(&persona).await
    }
}

fn trimmed_required(field: &'static str, value: &str) -> Result<String, AppError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(AppError::Domain(DomainError::Validation {
            field: field.into(),
            message: "is required".into(),
        }))
    } else {
        Ok(trimmed.to_string())
    }
}

fn prompt_hash(design: Option<&str>, style: Option<&str>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(design.unwrap_or("").as_bytes());
    hasher.update(b"\n");
    hasher.update(style.unwrap_or("").as_bytes());
    let digest = hasher.finalize();
    format!("{digest:x}")
}

fn not_found(id: TtsVoicePersonaId) -> AppError {
    AppError::Domain(DomainError::NotFound {
        entity: "tts_voice_persona",
        id: id.to_string(),
    })
}
