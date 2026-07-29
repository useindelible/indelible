use chrono::{Duration, Utc};
use ind_domain::{ClientType, UserId};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::error::AuthError;

const ACCESS_TOKEN_LIFETIME_SECS: i64 = 15 * 60;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub ct: String,
    pub scopes: Vec<String>,
    pub jti: String,
    pub iat: i64,
    pub exp: i64,
}

impl JwtClaims {
    pub fn user_id(&self) -> Result<UserId, AuthError> {
        self.sub.parse().map_err(|_| AuthError::TokenInvalid)
    }

    pub fn client_type(&self) -> ClientType {
        match self.ct.as_str() {
            "web" => ClientType::Web,
            "ios" => ClientType::Ios,
            "android" => ClientType::Android,
            "desktop" => ClientType::Desktop,
            "extension" => ClientType::Extension,
            "cli" => ClientType::Cli,
            _ => ClientType::Web,
        }
    }
}

pub fn client_type_to_claim(ct: ClientType) -> &'static str {
    match ct {
        ClientType::Web => "web",
        ClientType::Ios => "ios",
        ClientType::Android => "android",
        ClientType::Desktop => "desktop",
        ClientType::Extension => "extension",
        ClientType::Cli => "cli",
    }
}

pub fn sign_access_token(
    user_id: UserId,
    client_type: ClientType,
    scopes: &[String],
    secret: &[u8],
) -> Result<(String, i64), AuthError> {
    let now = Utc::now();
    let exp = now + Duration::seconds(ACCESS_TOKEN_LIFETIME_SECS);

    let claims = JwtClaims {
        sub: user_id.to_string(),
        ct: client_type_to_claim(client_type).to_string(),
        scopes: scopes.to_vec(),
        jti: uuid::Uuid::now_v7().to_string(),
        iat: now.timestamp(),
        exp: exp.timestamp(),
    };

    let token = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )
    .map_err(|_| AuthError::TokenInvalid)?;

    Ok((token, exp.timestamp()))
}

pub fn validate_access_token(token: &str, secret: &[u8]) -> Result<JwtClaims, AuthError> {
    // Pin the algorithm explicitly. `Validation::default()` already resolves to
    // HS256 in jsonwebtoken 9, but stating it guards against a future default
    // change and documents that algorithm-substitution (alg:none, HS/RS
    // confusion) is rejected.
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_required_spec_claims(&["sub", "exp", "iat"]);

    let data =
        jsonwebtoken::decode::<JwtClaims>(token, &DecodingKey::from_secret(secret), &validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                _ => AuthError::TokenInvalid,
            })?;

    Ok(data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECRET: &[u8] = b"test-jwt-secret-at-least-32-bytes-long";

    #[test]
    fn sign_and_validate_roundtrip() {
        let user_id = UserId::new();
        let scopes = vec!["read".to_string(), "write".to_string()];
        let (token, expires_at) =
            sign_access_token(user_id, ClientType::Web, &scopes, TEST_SECRET).unwrap();

        assert!(token.starts_with("eyJ"));
        assert!(expires_at > Utc::now().timestamp());

        let claims = validate_access_token(&token, TEST_SECRET).unwrap();
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.ct, "web");
        assert_eq!(claims.scopes, scopes);
    }

    #[test]
    fn validation_rejects_wrong_secret_garbage_and_algorithm_substitution() {
        let user_id = UserId::new();
        let (valid_token, _) =
            sign_access_token(user_id, ClientType::Web, &["read".to_string()], TEST_SECRET)
                .unwrap();
        let claims = JwtClaims {
            sub: UserId::new().to_string(),
            ct: "web".into(),
            scopes: vec![],
            jti: "x".into(),
            iat: Utc::now().timestamp(),
            exp: (Utc::now() + Duration::seconds(300)).timestamp(),
        };
        let hs512_token = jsonwebtoken::encode(
            &Header::new(jsonwebtoken::Algorithm::HS512),
            &claims,
            &EncodingKey::from_secret(TEST_SECRET),
        )
        .unwrap();
        use base64::Engine;
        use base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"none","typ":"JWT"}"#);
        let payload = URL_SAFE_NO_PAD
            .encode(br#"{"sub":"usr","ct":"web","scopes":[],"jti":"x","iat":0,"exp":9999999999}"#);
        let unsigned_token = format!("{header}.{payload}.");
        for (invalid, secret) in [
            (valid_token.as_str(), b"wrong-secret".as_slice()),
            ("not.a.jwt", TEST_SECRET),
            (hs512_token.as_str(), TEST_SECRET),
            (unsigned_token.as_str(), TEST_SECRET),
        ] {
            assert!(matches!(
                validate_access_token(invalid, secret),
                Err(AuthError::TokenInvalid)
            ));
        }
    }

    #[test]
    fn client_type_roundtrips() {
        let variants = [
            ClientType::Web,
            ClientType::Ios,
            ClientType::Android,
            ClientType::Desktop,
            ClientType::Extension,
            ClientType::Cli,
        ];
        for ct in variants {
            let claim_str = client_type_to_claim(ct);
            let claims = JwtClaims {
                sub: "usr_test".to_string(),
                ct: claim_str.to_string(),
                scopes: vec![],
                jti: "test".to_string(),
                iat: 0,
                exp: 0,
            };
            assert_eq!(claims.client_type(), ct);
        }
    }
}
