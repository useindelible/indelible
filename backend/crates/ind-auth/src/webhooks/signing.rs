use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub fn sign_webhook(secret: &str, timestamp: &str, body: &[u8]) -> String {
    #[expect(
        clippy::expect_used,
        reason = "HMAC-SHA256 accepts a key of any length; new_from_slice never returns Err"
    )]
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts any key length");
    mac.update(timestamp.as_bytes());
    mac.update(b".");
    mac.update(body);
    format!("v1={}", hex::encode(mac.finalize().into_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_binds_secret_timestamp_and_body() {
        let base = sign_webhook("secret", "100", b"payload");
        assert!(base.starts_with("v1="));
        assert_eq!(base.len(), 67);
        assert_ne!(base, sign_webhook("other", "100", b"payload"));
        assert_ne!(base, sign_webhook("secret", "101", b"payload"));
        assert_ne!(base, sign_webhook("secret", "100", b"changed"));
    }
}
