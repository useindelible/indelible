use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use hkdf::Hkdf;
use rand::RngCore;
use sha2::Sha256;

const CIPHER_VERSION: u8 = 1;
const NONCE_LEN: usize = 12;
const TAG_LEN: usize = 16;

#[derive(Debug, thiserror::Error)]
pub enum CipherError {
    #[error("credential key not configured")]
    MissingKey,
    #[error("invalid credential key: expected 32 bytes base64-encoded")]
    InvalidKey,
    #[error("ciphertext is malformed")]
    InvalidCiphertext,
    #[error("decryption failed")]
    DecryptionFailed,
}

/// AES-256-GCM credential sealer.
///
/// Seal format: `version(1) || nonce(12) || ciphertext || tag(16)`.
/// The cipher is a platform primitive shared between TTS and Mila credential columns.
#[derive(Clone)]
pub struct CredentialCipher {
    key: [u8; 32],
}

impl std::fmt::Debug for CredentialCipher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CredentialCipher").finish_non_exhaustive()
    }
}

impl CredentialCipher {
    pub fn from_base64(value: &str) -> Result<Self, CipherError> {
        let bytes = BASE64
            .decode(value.trim())
            .map_err(|_| CipherError::InvalidKey)?;
        if bytes.len() != 32 {
            return Err(CipherError::InvalidKey);
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&bytes);
        Ok(Self { key })
    }

    /// Deterministically derive a subkey for auxiliary primitives (for example the design quote HMAC).
    /// Using HKDF-SHA256 with a distinct info label means we never reuse the master key for another purpose.
    #[expect(
        clippy::expect_used,
        reason = "HKDF-SHA256 expand only errors when output_len exceeds 255*32=8160 bytes; all callers request fixed subkey sizes well under that bound"
    )]
    pub fn derive_subkey(&self, info: &[u8], output_len: usize) -> Vec<u8> {
        let hk = Hkdf::<Sha256>::new(None, &self.key);
        let mut out = vec![0u8; output_len];
        hk.expand(info, &mut out)
            .expect("hkdf expand cannot fail within its output-length cap");
        out
    }

    pub fn seal(&self, plaintext: &[u8]) -> Vec<u8> {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let mut nonce_bytes = [0u8; NONCE_LEN];
        rand::rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        #[expect(
            clippy::expect_used,
            reason = "AES-256-GCM encryption only fails when plaintext exceeds the GCM ~64GiB limit; credential payloads are bounded secrets far below it"
        )]
        let ciphertext_and_tag = cipher
            .encrypt(
                nonce,
                Payload {
                    msg: plaintext,
                    aad: &[],
                },
            )
            .expect("aes-gcm encryption is infallible with valid key and random nonce");

        let mut out = Vec::with_capacity(1 + NONCE_LEN + ciphertext_and_tag.len());
        out.push(CIPHER_VERSION);
        out.extend_from_slice(&nonce_bytes);
        out.extend_from_slice(&ciphertext_and_tag);
        out
    }

    pub fn open(&self, sealed: &[u8]) -> Result<Vec<u8>, CipherError> {
        if sealed.len() < 1 + NONCE_LEN + TAG_LEN {
            return Err(CipherError::InvalidCiphertext);
        }
        let version = sealed[0];
        if version != CIPHER_VERSION {
            return Err(CipherError::InvalidCiphertext);
        }
        let nonce = Nonce::from_slice(&sealed[1..1 + NONCE_LEN]);
        let ciphertext_and_tag = &sealed[1 + NONCE_LEN..];

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        cipher
            .decrypt(
                nonce,
                Payload {
                    msg: ciphertext_and_tag,
                    aad: &[],
                },
            )
            .map_err(|_| CipherError::DecryptionFailed)
    }

    pub fn version() -> i16 {
        CIPHER_VERSION as i16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cipher(byte: u8) -> CredentialCipher {
        CredentialCipher::from_base64(&BASE64.encode([byte; 32])).unwrap()
    }

    #[test]
    fn seal_round_trip_is_randomized_and_tamper_evident() {
        let primary = cipher(0x42);
        let first = primary.seal(b"provider-secret");
        let second = primary.seal(b"provider-secret");
        assert_ne!(first, second);
        assert_eq!(primary.open(&first).unwrap(), b"provider-secret");
        assert!(cipher(0x24).open(&first).is_err());

        let mut corrupted = first;
        *corrupted.last_mut().unwrap() ^= 1;
        assert!(primary.open(&corrupted).is_err());
        assert!(primary.open(&[]).is_err());
        assert_eq!(CredentialCipher::version(), 1);
    }

    #[test]
    fn key_and_subkey_boundaries_are_stable() {
        for invalid in ["not-base64".to_string(), BASE64.encode([0_u8; 31])] {
            assert!(CredentialCipher::from_base64(&invalid).is_err());
        }
        let cipher = cipher(1);
        assert_eq!(
            cipher.derive_subkey(b"quote", 32),
            cipher.derive_subkey(b"quote", 32)
        );
        assert_ne!(
            cipher.derive_subkey(b"quote", 32),
            cipher.derive_subkey(b"token", 32)
        );
    }
}
