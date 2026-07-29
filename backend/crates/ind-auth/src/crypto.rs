use std::sync::LazyLock;

use argon2::{
    Algorithm, Argon2, Params, Version,
    password_hash::{PasswordHasher, PasswordVerifier, SaltString},
};
use base64::engine::{Engine, general_purpose::URL_SAFE_NO_PAD};
use rand::Rng;
use sha2::{Digest, Sha256};

use crate::error::AuthError;

/// Pre-computed Argon2 hash used in the login "user not found" path so that
/// it performs a single verify_password call — the same work as the "wrong
/// password" path — preventing timing-based username enumeration.
pub static DUMMY_HASH: LazyLock<String> = LazyLock::new(|| {
    #[expect(
        clippy::expect_used,
        reason = "hashing a compile-time-constant string with valid Argon2 params cannot fail"
    )]
    let hash = hash_password("dummy-timing-safe").expect("DUMMY_HASH generation failed");
    hash
});

fn argon2_instance() -> Argon2<'static> {
    #[expect(
        clippy::expect_used,
        reason = "Argon2 params are compile-time constants within the library's accepted ranges"
    )]
    let params = Params::new(19456, 2, 1, None).expect("valid Argon2 params");
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
}

pub fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
    let hash = argon2_instance()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AuthError::HashError(e.to_string()))?;
    Ok(hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AuthError> {
    let parsed = argon2::password_hash::PasswordHash::new(hash)
        .map_err(|e| AuthError::HashError(e.to_string()))?;
    match argon2_instance().verify_password(password.as_bytes(), &parsed) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(AuthError::HashError(e.to_string())),
    }
}

fn generate_token_bytes() -> [u8; 32] {
    let mut buf = [0u8; 32];
    rand::rng().fill(&mut buf);
    buf
}

fn encode_token(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

pub fn generate_session_token() -> String {
    encode_token(&generate_token_bytes())
}

pub fn generate_refresh_token() -> String {
    format!("indr_{}", encode_token(&generate_token_bytes()))
}

pub fn generate_authorization_code() -> String {
    format!("indc_{}", encode_token(&generate_token_bytes()))
}

pub fn generate_api_token() -> String {
    format!("ind_{}", encode_token(&generate_token_bytes()))
}

pub fn generate_verification_token() -> String {
    encode_token(&generate_token_bytes())
}

pub fn generate_password_reset_token() -> String {
    encode_token(&generate_token_bytes())
}

pub fn generate_email_token() -> String {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    (0..8)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

pub fn hash_token(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    let mut hex = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write;
        #[expect(
            clippy::expect_used,
            reason = "writing formatted bytes into a String never fails"
        )]
        let () = write!(hex, "{byte:02x}").expect("writing to String cannot fail");
    }
    hex
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_hashing_is_salted_and_verifiable() {
        let hash = hash_password("correct-password").unwrap();
        assert!(hash.starts_with("$argon2id$"));
        for parameter in ["m=19456", "t=2", "p=1"] {
            assert!(hash.contains(parameter));
        }
        assert_ne!(hash, hash_password("correct-password").unwrap());
        assert!(verify_password("correct-password", &hash).unwrap());
        assert!(!verify_password("wrong-password", &hash).unwrap());
        assert!(matches!(
            verify_password("password", "not-a-hash"),
            Err(AuthError::HashError(_))
        ));
    }

    #[test]
    fn random_token_shapes_and_hash_properties_are_stable() {
        type TokenGenerator = (fn() -> String, &'static str, usize);
        let generators: [TokenGenerator; 4] = [
            (generate_session_token, "", 43),
            (generate_api_token, "ind_", 47),
            (generate_verification_token, "", 43),
            (generate_password_reset_token, "", 43),
        ];
        for (generate, prefix, length) in generators {
            let first = generate();
            assert_eq!(first.len(), length);
            assert!(first.starts_with(prefix));
            assert_ne!(first, generate());
        }
        let hash = hash_token("token-a");
        assert_eq!(hash.len(), 64);
        assert_eq!(hash, hash_token("token-a"));
        assert_ne!(hash, hash_token("token-b"));
    }
}
