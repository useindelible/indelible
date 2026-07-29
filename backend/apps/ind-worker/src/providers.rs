use std::sync::Arc;

use ind_integrations::email::{InboundEmailProvider, ResendProvider};
use secrecy::ExposeSecret;

use crate::config::WorkerConfig;

pub fn build_credential_cipher(config: &WorkerConfig) -> Option<Arc<ind_auth::CredentialCipher>> {
    let key = config.auth.credential_key.as_ref()?;
    match ind_auth::CredentialCipher::from_base64(key.expose_secret()) {
        Ok(cipher) => Some(Arc::new(cipher)),
        Err(e) => {
            tracing::warn!(
                error = %e,
                "auth.credential_key is set but invalid; integration OAuth token decryption disabled"
            );
            None
        }
    }
}

#[expect(
    clippy::expect_used,
    reason = "resend provider requires its secrets; missing or invalid config is a fatal worker boot misconfiguration"
)]
pub fn build_email_ingest_provider(config: &WorkerConfig) -> Option<Arc<dyn InboundEmailProvider>> {
    let provider_name = config.email_ingest.provider.as_deref()?;
    match provider_name {
        "resend" => {
            let secret = config
                .email_ingest
                .webhook_secret
                .as_ref()
                .expect("EMAIL_INGEST_WEBHOOK_SECRET required when provider=resend");
            let api_key = config
                .email_ingest
                .resend_api_key
                .as_ref()
                .expect("RESEND_API_KEY required when provider=resend");
            tracing::info!("email ingest provider configured: resend");
            Some(Arc::new(
                ResendProvider::new(secret.expose_secret(), api_key.expose_secret().to_owned())
                    .expect("failed to create ResendProvider"),
            ))
        }
        other => {
            tracing::warn!(provider = other, "unknown email ingest provider, skipping");
            None
        }
    }
}
