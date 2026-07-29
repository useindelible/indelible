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
    pub jti: String,
    pub iat: i64,
    pub exp: i64,
}

impl JwtClaims {
    pub fn user_id(&self) -> Result<UserId, AuthError> {
        self.sub.parse().map_err(|_| AuthError::TokenInvalid)
    }

    pub fn client_type(&self) -> Result<ClientType, AuthError> {
        match self.ct.as_str() {
            "web" => Ok(ClientType::Web),
            "ios" => Ok(ClientType::Ios),
            "android" => Ok(ClientType::Android),
            "desktop" => Ok(ClientType::Desktop),
            "extension" => Ok(ClientType::Extension),
            "cli" => Ok(ClientType::Cli),
            _ => Err(AuthError::TokenInvalid),
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
    secret: &[u8],
) -> Result<(String, i64), AuthError> {
    let now = Utc::now();
    let exp = now + Duration::seconds(ACCESS_TOKEN_LIFETIME_SECS);

    let claims = JwtClaims {
        sub: user_id.to_string(),
        ct: client_type_to_claim(client_type).to_string(),
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

    data.claims.client_type()?;

    Ok(data.claims)
}

#[cfg(test)]
mod tests {
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use serde_json::Value;

    use super::*;

    const TEST_SECRET: &[u8] = b"test-jwt-secret-at-least-32-bytes-long";

    #[test]
    fn access_token_claims_omit_scope_data_and_expire_after_fifteen_minutes() {
        let user_id = UserId::new();
        let (token, expires_at) = sign_access_token(user_id, ClientType::Web, TEST_SECRET).unwrap();

        let payload = token.split('.').nth(1).unwrap();
        let claims: Value =
            serde_json::from_slice(&URL_SAFE_NO_PAD.decode(payload).unwrap()).unwrap();

        assert_eq!(claims["sub"], user_id.to_string());
        assert_eq!(claims["ct"], "web");
        assert_eq!(
            claims["exp"].as_i64().unwrap() - claims["iat"].as_i64().unwrap(),
            15 * 60
        );
        assert_eq!(claims["exp"], expires_at);
        assert!(claims.get("scopes").is_none());
        assert!(claims.get("scope").is_none());
    }

    #[test]
    fn validation_rejects_unknown_and_missing_client_type_claims() {
        let now = Utc::now().timestamp();
        for claims in [
            serde_json::json!({
                "sub": UserId::new().to_string(),
                "ct": "browser",
                "scopes": [],
                "jti": "unknown-client-type",
                "iat": now,
                "exp": now + 300,
            }),
            serde_json::json!({
                "sub": UserId::new().to_string(),
                "scopes": [],
                "jti": "missing-client-type",
                "iat": now,
                "exp": now + 300,
            }),
        ] {
            let token = jsonwebtoken::encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(TEST_SECRET),
            )
            .unwrap();

            assert!(matches!(
                validate_access_token(&token, TEST_SECRET),
                Err(AuthError::TokenInvalid)
            ));
        }
    }

    #[test]
    fn sign_and_validate_roundtrip() {
        let user_id = UserId::new();
        let (token, expires_at) = sign_access_token(user_id, ClientType::Web, TEST_SECRET).unwrap();

        assert!(token.starts_with("eyJ"));
        assert!(expires_at > Utc::now().timestamp());

        let claims = validate_access_token(&token, TEST_SECRET).unwrap();
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.ct, "web");
    }

    #[test]
    fn validation_rejects_wrong_secret_garbage_and_algorithm_substitution() {
        let user_id = UserId::new();
        let (valid_token, _) = sign_access_token(user_id, ClientType::Web, TEST_SECRET).unwrap();
        let claims = JwtClaims {
            sub: UserId::new().to_string(),
            ct: "web".into(),
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
                jti: "test".to_string(),
                iat: 0,
                exp: 0,
            };
            assert!(matches!(claims.client_type(), Ok(actual) if actual == ct));
        }
    }
}
