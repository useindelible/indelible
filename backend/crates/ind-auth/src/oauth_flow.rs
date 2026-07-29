use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use chrono::{DateTime, Utc};
use ind_application::error::AppError;
use ind_application::repos::oauth_flow::OAuthFlowRepository;
use ind_domain::ClientType;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::crypto::hash_token;
use crate::oauth::OidcFlow;

const OAUTH_FLOW_VERSION: u8 = 1;
const OAUTH_FLOW_NONCE_LEN: usize = 12;

#[derive(Debug, thiserror::Error)]
pub enum OAuthFlowError {
    #[error("failed to serialize OAuth flow")]
    Serialize(#[from] serde_json::Error),
    #[error("failed to seal OAuth flow")]
    Seal,
    #[error("invalid OAuth flow")]
    Invalid,
}

#[derive(Debug, thiserror::Error)]
pub enum OAuthFlowStorageError {
    #[error("failed to seal OAuth flow: {0}")]
    Seal(#[source] OAuthFlowError),
    #[error("invalid OAuth flow")]
    Invalid(#[source] OAuthFlowError),
    #[error("invalid OAuth flow expiration")]
    InvalidExpiration,
    #[error(transparent)]
    Repository(#[from] AppError),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoredOAuthFlow {
    pub provider: String,
    pub csrf_state: String,
    pub issuer: Option<String>,
    pub oidc_flow: Option<OidcFlow>,
    pub kind: StoredOAuthFlowKind,
    pub expires_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StoredOAuthFlowKind {
    Web,
    Native(NativeOAuthFlow),
}

impl StoredOAuthFlowKind {
    pub fn as_storage_kind(&self) -> &'static str {
        match self {
            Self::Web => "web",
            Self::Native(_) => "native",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NativeOAuthFlow {
    pub platform: ClientType,
    pub redirect_uri: String,
    pub code_challenge: String,
    pub app_state: String,
}

pub fn seal_oauth_flow(flow: &StoredOAuthFlow, secret: &[u8]) -> Result<Vec<u8>, OAuthFlowError> {
    let plaintext = serde_json::to_vec(flow)?;
    let key = oauth_flow_cookie_key(secret);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let mut nonce_bytes = [0u8; OAUTH_FLOW_NONCE_LEN];
    rand::rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|_| OAuthFlowError::Seal)?;

    let mut sealed = Vec::with_capacity(1 + nonce_bytes.len() + ciphertext.len());
    sealed.push(OAUTH_FLOW_VERSION);
    sealed.extend_from_slice(&nonce_bytes);
    sealed.extend_from_slice(&ciphertext);
    Ok(sealed)
}

pub fn open_oauth_flow(sealed: &[u8], secret: &[u8]) -> Result<StoredOAuthFlow, OAuthFlowError> {
    if sealed.len() <= 1 + OAUTH_FLOW_NONCE_LEN || sealed[0] != OAUTH_FLOW_VERSION {
        return Err(OAuthFlowError::Invalid);
    }

    let key = oauth_flow_cookie_key(secret);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let nonce = Nonce::from_slice(&sealed[1..1 + OAUTH_FLOW_NONCE_LEN]);
    let plaintext = cipher
        .decrypt(nonce, &sealed[1 + OAUTH_FLOW_NONCE_LEN..])
        .map_err(|_| OAuthFlowError::Invalid)?;

    serde_json::from_slice(&plaintext).map_err(|_| OAuthFlowError::Invalid)
}

pub async fn store_oauth_flow(
    repo: &dyn OAuthFlowRepository,
    flow: &StoredOAuthFlow,
    secret: &[u8],
) -> Result<(), OAuthFlowStorageError> {
    let sealed_flow = seal_oauth_flow(flow, secret).map_err(OAuthFlowStorageError::Seal)?;
    let state_hash = hash_token(&flow.csrf_state);
    let expires_at = DateTime::<Utc>::from_timestamp(flow.expires_at, 0)
        .ok_or(OAuthFlowStorageError::InvalidExpiration)?;

    repo.upsert(
        &state_hash,
        &flow.provider,
        flow.kind.as_storage_kind(),
        sealed_flow,
        expires_at,
    )
    .await?;

    Ok(())
}

pub async fn consume_oauth_flow(
    repo: &dyn OAuthFlowRepository,
    raw_state: &str,
    secret: &[u8],
) -> Result<Option<StoredOAuthFlow>, OAuthFlowStorageError> {
    let state_hash = hash_token(raw_state);
    let sealed_flow = repo.consume(&state_hash).await?;

    sealed_flow
        .map(|sealed_flow| {
            open_oauth_flow(&sealed_flow, secret).map_err(OAuthFlowStorageError::Invalid)
        })
        .transpose()
}

fn oauth_flow_cookie_key(secret: &[u8]) -> [u8; 32] {
    let digest = Sha256::digest([b"indelible oauth flow cookie v1".as_slice(), secret].concat());
    let mut key = [0u8; 32];
    key.copy_from_slice(&digest);
    key
}
