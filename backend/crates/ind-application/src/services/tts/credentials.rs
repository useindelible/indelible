use std::collections::HashMap;

use async_trait::async_trait;
use ind_domain::TtsProvider;

use crate::AppError;

/// Resolved credentials for a provider call. TTS is deployment-configured:
/// credentials come from server config/env, never from a user-owned settings row.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolvedTtsCredentials {
    pub api_key: Option<String>,
    pub api_base: Option<String>,
}

#[async_trait]
pub trait TtsProviderCredentialResolver: Send + Sync {
    async fn resolve(&self, provider: TtsProvider) -> Result<ResolvedTtsCredentials, AppError>;
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeploymentTtsCredential {
    pub api_key: String,
    pub api_base: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct DefaultTtsCredentialResolver {
    configured: HashMap<TtsProvider, DeploymentTtsCredential>,
}

impl DefaultTtsCredentialResolver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_credential(
        mut self,
        provider: TtsProvider,
        credential: DeploymentTtsCredential,
    ) -> Self {
        if !credential.api_key.trim().is_empty() {
            self.configured.insert(provider, credential);
        }
        self
    }
}

#[async_trait]
impl TtsProviderCredentialResolver for DefaultTtsCredentialResolver {
    async fn resolve(&self, provider: TtsProvider) -> Result<ResolvedTtsCredentials, AppError> {
        if provider == TtsProvider::Mock {
            return Ok(ResolvedTtsCredentials::default());
        }

        let credential =
            self.configured
                .get(&provider)
                .ok_or_else(|| AppError::ExternalService {
                    service: "tts".into(),
                    message: format!(
                        "deployment credentials are not configured for {}",
                        provider.as_str()
                    ),
                })?;

        Ok(ResolvedTtsCredentials {
            api_key: Some(credential.api_key.clone()),
            api_base: credential.api_base.clone(),
        })
    }
}
