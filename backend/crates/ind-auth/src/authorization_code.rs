use std::sync::Arc;

use base64::engine::{Engine, general_purpose::URL_SAFE_NO_PAD};
use chrono::{Duration, Utc};
use ind_application::repos::authorization_code::AuthorizationCodeRepository;
use ind_domain::{AuthorizationCode, AuthorizationCodeId, ClientType, UserId};
use sha2::{Digest, Sha256};

use crate::crypto::{generate_authorization_code, hash_token};
use crate::error::AuthError;
use crate::refresh_token::RefreshTokenService;

const EXTENSION_CODE_LIFETIME_MINUTES: i64 = 5;
const NATIVE_CODE_LIFETIME_SECS: i64 = 60;

pub struct AuthorizationCodeService {
    code_repo: Arc<dyn AuthorizationCodeRepository>,
    refresh_service: Arc<RefreshTokenService>,
    allowed_redirect_uris: Vec<String>,
}

#[derive(Debug)]
pub struct AuthCodeResult {
    pub raw_code: String,
}

#[derive(Debug)]
pub struct TokenExchangeResult {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
    pub refresh_token_expires_at: i64,
}

impl AuthorizationCodeService {
    pub fn new(
        code_repo: Arc<dyn AuthorizationCodeRepository>,
        refresh_service: Arc<RefreshTokenService>,
        allowed_redirect_uris: Vec<String>,
    ) -> Self {
        Self {
            code_repo,
            refresh_service,
            allowed_redirect_uris,
        }
    }

    pub async fn create_code(
        &self,
        user_id: UserId,
        client_type: ClientType,
        code_challenge: String,
        code_challenge_method: String,
        redirect_uri: String,
    ) -> Result<AuthCodeResult, AuthError> {
        if code_challenge_method != "S256" {
            return Err(AuthError::ValidationError {
                field: "code_challenge_method".to_string(),
                message: "only S256 is supported".to_string(),
            });
        }

        if !self.allowed_redirect_uris.contains(&redirect_uri) {
            return Err(AuthError::ValidationError {
                field: "redirect_uri".to_string(),
                message: "redirect_uri is not allowed".to_string(),
            });
        }

        let now = Utc::now();
        let raw_code = generate_authorization_code();
        let code_hash = hash_token(&raw_code);
        let expires_at = match client_type {
            ClientType::Ios | ClientType::Android => {
                now + Duration::seconds(NATIVE_CODE_LIFETIME_SECS)
            }
            _ => now + Duration::minutes(EXTENSION_CODE_LIFETIME_MINUTES),
        };

        let auth_code = AuthorizationCode {
            id: AuthorizationCodeId::new(),
            user_id,
            code_hash,
            code_challenge,
            code_challenge_method,
            client_type,
            redirect_uri,
            scopes: vec!["read".to_string(), "write".to_string()],
            used_at: None,
            expires_at,
            created_at: now,
        };

        self.code_repo.create(auth_code).await?;

        Ok(AuthCodeResult { raw_code })
    }

    pub async fn exchange_code(
        &self,
        raw_code: &str,
        code_verifier: &str,
        redirect_uri: &str,
        expected_client_type: ClientType,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> Result<TokenExchangeResult, AuthError> {
        let code_hash = hash_token(raw_code);
        let auth_code = self
            .code_repo
            .consume_by_code_hash(&code_hash)
            .await?
            .ok_or(AuthError::TokenInvalid)?;

        if auth_code.expires_at < Utc::now() {
            return Err(AuthError::TokenExpired);
        }

        if auth_code.client_type != expected_client_type {
            return Err(AuthError::TokenInvalid);
        }

        if auth_code.redirect_uri != redirect_uri {
            return Err(AuthError::TokenInvalid);
        }

        // Verify PKCE: S256 means hash(code_verifier) == code_challenge
        let verifier_hash = {
            let digest = Sha256::digest(code_verifier.as_bytes());
            URL_SAFE_NO_PAD.encode(digest)
        };

        if verifier_hash != auth_code.code_challenge {
            return Err(AuthError::TokenInvalid);
        }

        // Issue tokens
        let pair = self
            .refresh_service
            .issue_tokens(auth_code.user_id, auth_code.client_type, ip, user_agent)
            .await?;

        Ok(TokenExchangeResult {
            access_token: pair.access_token,
            refresh_token: pair.raw_refresh_token,
            expires_at: pair.expires_at,
            refresh_token_expires_at: pair.refresh_token.absolute_expires_at.timestamp(),
        })
    }
}
