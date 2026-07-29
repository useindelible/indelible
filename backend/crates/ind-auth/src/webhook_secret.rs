use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::RngCore;
use sha2::{Digest, Sha256};

use crate::{CipherError, CredentialCipher};

pub fn generate_webhook_secret() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    format!("whsec_{}", URL_SAFE_NO_PAD.encode(bytes))
}

pub fn webhook_secret_hash(raw_secret: &str) -> String {
    hex::encode(Sha256::digest(raw_secret.as_bytes()))
}

pub fn webhook_secret_preview(raw_secret: &str) -> String {
    let suffix = raw_secret
        .chars()
        .rev()
        .take(4)
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    format!("whsec_••••{suffix}")
}

pub fn seal_webhook_secret(
    cipher: &CredentialCipher,
    raw_secret: &str,
) -> (String, Vec<u8>, String) {
    (
        webhook_secret_hash(raw_secret),
        cipher.seal(raw_secret.as_bytes()),
        webhook_secret_preview(raw_secret),
    )
}

pub fn open_webhook_secret(
    cipher: &CredentialCipher,
    ciphertext: &[u8],
) -> Result<String, WebhookSecretOpenError> {
    let plaintext = cipher.open(ciphertext)?;
    String::from_utf8(plaintext).map_err(WebhookSecretOpenError::Utf8)
}

#[derive(Debug, thiserror::Error)]
pub enum WebhookSecretOpenError {
    #[error(transparent)]
    Cipher(#[from] CipherError),
    #[error("webhook secret is not valid utf-8: {0}")]
    Utf8(std::string::FromUtf8Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::STANDARD;

    #[test]
    fn generated_and_sealed_webhook_secrets_preserve_public_contracts() {
        let raw = generate_webhook_secret();
        assert!(raw.starts_with("whsec_"));
        assert_eq!(raw.len(), 49);

        let cipher = CredentialCipher::from_base64(&STANDARD.encode([7_u8; 32])).unwrap();
        let (hash, sealed, preview) = seal_webhook_secret(&cipher, &raw);
        assert_eq!(hash, webhook_secret_hash(&raw));
        assert!(preview.ends_with(&raw[raw.len() - 4..]));
        assert_eq!(open_webhook_secret(&cipher, &sealed).unwrap(), raw);
        assert!(open_webhook_secret(&cipher, b"invalid").is_err());
    }
}
