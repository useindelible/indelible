pub mod notion;
pub mod settings;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::sync::Arc;

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Duration, Utc};
use hkdf::Hkdf;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use ind_application::AppError;
use ind_domain::{IntegrationOAuthProvider, UserId};

use crate::crypto::hash_token;

const INTEGRATION_FLOW_CONTEXT: &[u8] = b"integration-oauth-flow-seal-v1";
const INTEGRATION_FLOW_NONCE_LEN: usize = 12;
const INTEGRATION_FLOW_VERSION: u8 = 1;
const INTEGRATION_FLOW_TTL_SECS: i64 = 10 * 60;

#[derive(Debug, thiserror::Error)]
pub enum IntegrationOAuthError {
    #[error("integration provider {0:?} is not configured")]
    ProviderNotConfigured(IntegrationOAuthProvider),

    #[error("integration OAuth state mismatch or expired")]
    InvalidState,

    #[error("integration provider mismatch")]
    ProviderMismatch,

    #[error("integration OAuth exchange failed: {0}")]
    Exchange(String),

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("integration OAuth configuration error: {0}")]
    Configuration(String),

    #[error("integration OAuth flow seal error: {0}")]
    Seal(String),

    #[error("integration OAuth flow storage error: {0}")]
    Storage(String),
}

/// Map an [`IntegrationOAuthError`] to the application's [`AppError`] in a way
/// that keeps the C12 redirect mapping working: state/provider/credentials
/// failures land in `Domain::Validation` so the HTTP layer can emit distinct
/// kinds, while configuration / sealing / storage problems become
/// `Repository` because they're server-side.
pub fn integration_oauth_error_to_app_error(err: IntegrationOAuthError) -> AppError {
    use IntegrationOAuthError as E;
    match err {
        E::ProviderNotConfigured(provider) => AppError::Domain(ind_domain::DomainError::NotFound {
            entity: "integration_provider",
            id: format!("{provider:?}"),
        }),
        E::InvalidState => AppError::Domain(ind_domain::DomainError::Validation {
            field: "state".to_string(),
            message: "integration OAuth state mismatch or expired".to_string(),
        }),
        E::ProviderMismatch => AppError::Domain(ind_domain::DomainError::Validation {
            field: "provider".to_string(),
            message: "integration OAuth provider mismatch".to_string(),
        }),
        E::Exchange(message) => AppError::ExternalService {
            service: "integration_oauth".to_string(),
            message,
        },
        E::InvalidCredentials => AppError::Domain(ind_domain::DomainError::Validation {
            field: "credentials".to_string(),
            message: "invalid credentials".to_string(),
        }),
        E::Configuration(_) | E::Seal(_) | E::Storage(_) => AppError::Repository(Box::new(err)),
    }
}

#[derive(Debug, Clone)]
pub struct ProviderTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub extra: serde_json::Value,
}

#[async_trait::async_trait]
pub trait IntegrationOAuthProviderAdapter: Send + Sync {
    fn provider(&self) -> IntegrationOAuthProvider;
    fn authorize_url(&self, state: &str, redirect_uri: &str) -> String;
    async fn exchange_code(
        &self,
        code: &str,
        state: &str,
    ) -> Result<ProviderTokens, IntegrationOAuthError>;
}

#[async_trait::async_trait]
pub trait IntegrationOAuthFlowStore: Send + Sync {
    async fn insert(
        &self,
        state_hash: String,
        provider: IntegrationOAuthProvider,
        sealed_flow: Vec<u8>,
        expires_at: DateTime<Utc>,
    ) -> Result<(), IntegrationOAuthError>;

    async fn consume(
        &self,
        state_hash: String,
        provider: IntegrationOAuthProvider,
    ) -> Result<Option<Vec<u8>>, IntegrationOAuthError>;
}

pub struct RepositoryIntegrationOAuthFlowStore {
    repo: Arc<dyn ind_application::repos::oauth_flow::OAuthFlowRepository>,
}

impl RepositoryIntegrationOAuthFlowStore {
    pub fn new(repo: Arc<dyn ind_application::repos::oauth_flow::OAuthFlowRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait::async_trait]
impl IntegrationOAuthFlowStore for RepositoryIntegrationOAuthFlowStore {
    async fn insert(
        &self,
        state_hash: String,
        provider: IntegrationOAuthProvider,
        sealed_flow: Vec<u8>,
        expires_at: DateTime<Utc>,
    ) -> Result<(), IntegrationOAuthError> {
        self.repo
            .insert_strict(
                &state_hash,
                provider.as_str(),
                "integration",
                sealed_flow,
                expires_at,
            )
            .await
            .map_err(|error| IntegrationOAuthError::Storage(error.to_string()))
    }

    async fn consume(
        &self,
        state_hash: String,
        provider: IntegrationOAuthProvider,
    ) -> Result<Option<Vec<u8>>, IntegrationOAuthError> {
        self.repo
            .consume_scoped(&state_hash, provider.as_str(), "integration")
            .await
            .map_err(|error| IntegrationOAuthError::Storage(error.to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SealedIntegrationFlow {
    user_id: UserId,
    provider: IntegrationOAuthProvider,
    redirect_after: Option<String>,
    csrf_state: String,
    expires_at: i64,
}

pub struct StartedIntegrationFlow {
    pub authorize_url: String,
    pub state: String,
}

pub struct CompletedIntegrationFlow {
    pub user_id: UserId,
    pub provider: IntegrationOAuthProvider,
    pub tokens: ProviderTokens,
    pub redirect_after: Option<String>,
}

pub struct IntegrationOAuthService {
    adapters: HashMap<IntegrationOAuthProvider, Arc<dyn IntegrationOAuthProviderAdapter>>,
    flow_store: Arc<dyn IntegrationOAuthFlowStore>,
    seal_key: [u8; 32],
    redirect_uri_base: String,
}

impl IntegrationOAuthService {
    pub fn new(
        adapters: Vec<Arc<dyn IntegrationOAuthProviderAdapter>>,
        flow_store: Arc<dyn IntegrationOAuthFlowStore>,
        csrf_secret: &[u8],
        redirect_uri_base: String,
    ) -> Self {
        let mut map: HashMap<IntegrationOAuthProvider, Arc<dyn IntegrationOAuthProviderAdapter>> =
            HashMap::new();
        for adapter in adapters {
            map.insert(adapter.provider(), adapter);
        }
        Self {
            adapters: map,
            flow_store,
            seal_key: derive_seal_key(csrf_secret),
            redirect_uri_base,
        }
    }

    pub fn configured_providers(&self) -> Vec<IntegrationOAuthProvider> {
        let mut providers: Vec<_> = self.adapters.keys().copied().collect();
        providers.sort_by_key(|p| p.as_str());
        providers
    }

    pub fn has_provider(&self, provider: IntegrationOAuthProvider) -> bool {
        self.adapters.contains_key(&provider)
    }

    pub async fn start(
        &self,
        user_id: UserId,
        provider: IntegrationOAuthProvider,
        redirect_after: Option<String>,
    ) -> Result<StartedIntegrationFlow, IntegrationOAuthError> {
        let adapter = self
            .adapters
            .get(&provider)
            .ok_or(IntegrationOAuthError::ProviderNotConfigured(provider))?;

        let state = generate_state();
        let expires_at = Utc::now() + Duration::seconds(INTEGRATION_FLOW_TTL_SECS);

        let flow = SealedIntegrationFlow {
            user_id,
            provider,
            redirect_after,
            csrf_state: state.clone(),
            expires_at: expires_at.timestamp(),
        };
        let sealed = seal_flow(&flow, &self.seal_key)?;

        let state_hash = hash_token(&state);
        self.flow_store
            .insert(state_hash, provider, sealed, expires_at)
            .await?;

        let redirect_uri = self.redirect_uri_for(provider);
        let authorize_url = adapter.authorize_url(&state, &redirect_uri);

        Ok(StartedIntegrationFlow {
            authorize_url,
            state,
        })
    }

    pub async fn complete(
        &self,
        provider: IntegrationOAuthProvider,
        code: &str,
        state: &str,
    ) -> Result<CompletedIntegrationFlow, IntegrationOAuthError> {
        let adapter = self
            .adapters
            .get(&provider)
            .ok_or(IntegrationOAuthError::ProviderNotConfigured(provider))?;

        let state_hash = hash_token(state);
        let sealed = self
            .flow_store
            .consume(state_hash, provider)
            .await?
            .ok_or(IntegrationOAuthError::InvalidState)?;

        let flow = open_flow(&sealed, &self.seal_key)?;
        if flow.provider != provider {
            return Err(IntegrationOAuthError::ProviderMismatch);
        }
        if flow.csrf_state != state {
            return Err(IntegrationOAuthError::InvalidState);
        }
        let now = Utc::now().timestamp();
        if flow.expires_at < now {
            return Err(IntegrationOAuthError::InvalidState);
        }

        let tokens = adapter.exchange_code(code, state).await?;

        Ok(CompletedIntegrationFlow {
            user_id: flow.user_id,
            provider,
            tokens,
            redirect_after: flow.redirect_after,
        })
    }

    fn redirect_uri_for(&self, provider: IntegrationOAuthProvider) -> String {
        format!(
            "{}/api/v1/integrations/{}/callback",
            self.redirect_uri_base.trim_end_matches('/'),
            provider.as_str()
        )
    }
}

fn generate_state() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn derive_seal_key(csrf_secret: &[u8]) -> [u8; 32] {
    let hkdf = Hkdf::<Sha256>::new(None, csrf_secret);
    let mut key = [0u8; 32];
    #[expect(
        clippy::expect_used,
        reason = "HKDF-SHA256 expand into a fixed 32-byte buffer is well under the 8160-byte cap and cannot fail"
    )]
    let () = hkdf
        .expand(INTEGRATION_FLOW_CONTEXT, &mut key)
        .expect("HKDF expand should succeed for 32-byte key");
    key
}

fn seal_flow(
    flow: &SealedIntegrationFlow,
    key: &[u8; 32],
) -> Result<Vec<u8>, IntegrationOAuthError> {
    let plaintext = serde_json::to_vec(flow)
        .map_err(|e| IntegrationOAuthError::Seal(format!("serialize: {e}")))?;
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let mut nonce_bytes = [0u8; INTEGRATION_FLOW_NONCE_LEN];
    rand::rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|_| IntegrationOAuthError::Seal("encrypt".to_string()))?;

    let mut sealed = Vec::with_capacity(1 + nonce_bytes.len() + ciphertext.len());
    sealed.push(INTEGRATION_FLOW_VERSION);
    sealed.extend_from_slice(&nonce_bytes);
    sealed.extend_from_slice(&ciphertext);
    Ok(sealed)
}

fn open_flow(
    sealed: &[u8],
    key: &[u8; 32],
) -> Result<SealedIntegrationFlow, IntegrationOAuthError> {
    if sealed.len() < 1 + INTEGRATION_FLOW_NONCE_LEN {
        return Err(IntegrationOAuthError::Seal(
            "sealed payload too short".into(),
        ));
    }
    if sealed[0] != INTEGRATION_FLOW_VERSION {
        return Err(IntegrationOAuthError::Seal(
            "sealed payload version mismatch".into(),
        ));
    }
    let nonce = Nonce::from_slice(&sealed[1..1 + INTEGRATION_FLOW_NONCE_LEN]);
    let ciphertext = &sealed[1 + INTEGRATION_FLOW_NONCE_LEN..];
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| IntegrationOAuthError::Seal("decrypt".to_string()))?;
    serde_json::from_slice(&plaintext)
        .map_err(|e| IntegrationOAuthError::Seal(format!("deserialize: {e}")))
}
