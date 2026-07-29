use std::time::{SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use ind_domain::UserId;
use sha2::Sha256;
use subtle::ConstantTimeEq;

pub const ASSET_COOKIE_MAX_AGE_SECS: u64 = 3600;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, thiserror::Error)]
pub enum AssetCookieSecretError {
    #[error("asset cookie secret must not be empty")]
    Empty,
    #[error("asset cookie secret must be valid hex: {0}")]
    InvalidHex(#[from] hex::FromHexError),
}

pub fn decode_asset_cookie_secret(secret_hex: &str) -> Result<Vec<u8>, AssetCookieSecretError> {
    if secret_hex.is_empty() {
        return Err(AssetCookieSecretError::Empty);
    }
    Ok(hex::decode(secret_hex)?)
}

pub fn sign_asset_cookie(user_id: &UserId, secret: &[u8]) -> String {
    #[expect(
        clippy::expect_used,
        reason = "system clock before the Unix epoch is an unrecoverable host misconfiguration, not fallible input"
    )]
    let expires = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before epoch")
        .as_secs()
        + ASSET_COOKIE_MAX_AGE_SECS;
    let payload = format!("v1:{user_id}:{expires}");
    let sig = asset_cookie_signature(&payload, secret);
    format!("{payload}:{sig}")
}

pub fn verify_asset_cookie(value: &str, secret: &[u8]) -> Option<UserId> {
    let parts: Vec<&str> = value.splitn(4, ':').collect();
    if parts.len() != 4 || parts[0] != "v1" {
        return None;
    }

    let user_id_str = parts[1];
    let expires_str = parts[2];
    let provided_sig = parts[3];

    let expires: u64 = expires_str.parse().ok()?;
    #[expect(
        clippy::expect_used,
        reason = "system clock before the Unix epoch is an unrecoverable host misconfiguration, not fallible input"
    )]
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before epoch")
        .as_secs();
    if now > expires {
        return None;
    }

    let payload = format!("v1:{user_id_str}:{expires_str}");
    let expected_sig = asset_cookie_signature(&payload, secret);
    if provided_sig
        .as_bytes()
        .ct_eq(expected_sig.as_bytes())
        .into()
    {
        user_id_str.parse().ok()
    } else {
        None
    }
}

fn asset_cookie_signature(payload: &str, secret: &[u8]) -> String {
    #[expect(
        clippy::expect_used,
        reason = "HMAC-SHA256 accepts a key of any length; new_from_slice never returns Err"
    )]
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts any key size");
    mac.update(payload.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &[u8] = b"test-secret-key-at-least-32-bytes-long!!";

    #[test]
    fn cookie_round_trip_rejects_tampering_expiry_and_malformed_values() {
        let user = UserId::new();
        let cookie = sign_asset_cookie(&user, SECRET);
        assert_eq!(verify_asset_cookie(&cookie, SECRET), Some(user));

        let expired_payload = format!("v1:{user}:1000000000");
        let expired = format!(
            "{expired_payload}:{}",
            asset_cookie_signature(&expired_payload, SECRET)
        );
        for invalid in [
            cookie.replacen(&user.to_string(), &UserId::new().to_string(), 1),
            expired,
            "garbage".into(),
            "v2:a:b:c".into(),
        ] {
            assert_eq!(verify_asset_cookie(&invalid, SECRET), None);
        }
        assert_eq!(verify_asset_cookie(&cookie, b"wrong-secret"), None);
    }

    #[test]
    fn secret_decoder_enforces_nonempty_hex() {
        let valid = "de".repeat(32);
        assert_eq!(decode_asset_cookie_secret(&valid).unwrap().len(), 32);
        assert!(decode_asset_cookie_secret("").is_err());
        assert!(decode_asset_cookie_secret("not-hex").is_err());
    }
}
