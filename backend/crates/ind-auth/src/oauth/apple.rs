use jsonwebtoken::{Algorithm, EncodingKey, Header};
use oauth2::basic::{BasicErrorResponse, BasicTokenType};
use oauth2::{
    AuthUrl, AuthorizationCode, Client, ClientId, ClientSecret, CsrfToken, EndpointNotSet,
    EndpointSet, ExtraTokenFields, RedirectUrl, RefreshToken, RevocationErrorResponseType, Scope,
    StandardErrorResponse, StandardRevocableToken, StandardTokenIntrospectionResponse,
    StandardTokenResponse, TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};

use super::{AppleOAuthConfig, OAuthAuthorizationUrl, OAuthUserInfo, error::OAuthError};
use ind_domain::OAuthProvider;

const APPLE_AUTH_URL: &str = "https://appleid.apple.com/auth/authorize";
const APPLE_TOKEN_URL: &str = "https://appleid.apple.com/auth/token";
const APPLE_ISSUER: &str = "https://appleid.apple.com";

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AppleExtraFields {
    #[serde(default)]
    id_token: Option<String>,
}
impl ExtraTokenFields for AppleExtraFields {}

type AppleTokenResponse = StandardTokenResponse<AppleExtraFields, BasicTokenType>;
type AppleRevocationErrorResponse = StandardErrorResponse<RevocationErrorResponseType>;

type AppleClient = Client<
    BasicErrorResponse,
    AppleTokenResponse,
    StandardTokenIntrospectionResponse<AppleExtraFields, BasicTokenType>,
    StandardRevocableToken,
    AppleRevocationErrorResponse,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointSet,
>;

#[derive(Debug, Deserialize)]
struct AppleIdTokenClaims {
    sub: String,
    email: Option<String>,
    #[serde(default)]
    email_verified: Option<bool>,
}

pub struct AppleOAuth {
    config: AppleOAuthConfig,
    http_client: oauth2::reqwest::Client,
}

impl AppleOAuth {
    pub fn new(config: &AppleOAuthConfig) -> Result<Self, OAuthError> {
        build_client(config, None)?;

        Ok(Self {
            config: config.clone(),
            http_client: oauth2::reqwest::Client::new(),
        })
    }

    pub fn authorization_url(&self) -> OAuthAuthorizationUrl {
        #[expect(
            clippy::expect_used,
            reason = "config was already validated by build_client in AppleOAuth::new; rebuilding from the same stored config cannot fail"
        )]
        let client = build_client(&self.config, None).expect("validated Apple OAuth config");
        let (url, csrf_state) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("name".into()))
            .add_scope(Scope::new("email".into()))
            .url();

        let url = format!("{url}&response_mode=form_post");

        OAuthAuthorizationUrl {
            url,
            csrf_state: csrf_state.secret().clone(),
            issuer: Some(APPLE_ISSUER.to_string()),
            oidc_flow: None,
        }
    }

    pub async fn exchange_code(&self, code: &str) -> Result<OAuthUserInfo, OAuthError> {
        let client_secret = generate_client_secret(&self.config)?;
        let client = build_client(&self.config, Some(client_secret))?;
        let token_response = client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .request_async(&self.http_client)
            .await
            .map_err(|e| OAuthError::Exchange(e.to_string()))?;

        let access_token = token_response.access_token().secret().clone();

        let id_token = token_response
            .extra_fields()
            .id_token
            .as_deref()
            .ok_or_else(|| OAuthError::Exchange("missing id_token in response".into()))?;

        let claims = decode_id_token_claims(id_token)?;

        let verified_email = match (claims.email, claims.email_verified) {
            (Some(email), Some(true)) => Some(email),
            (Some(_email), _) => None,
            (None, _) => None,
        };

        let email_verified = verified_email.as_ref().map(|_| true);

        Ok(OAuthUserInfo {
            provider: OAuthProvider::Apple,
            provider_user_id: claims.sub,
            email: verified_email,
            display_name: None,
            avatar_url: None,
            access_token,
            refresh_token: token_response
                .refresh_token()
                .map(|t: &RefreshToken| t.secret().clone()),
            email_verified,
            allow_auto_create: true,
        })
    }
}

fn build_client(
    config: &AppleOAuthConfig,
    client_secret: Option<String>,
) -> Result<AppleClient, OAuthError> {
    let client = Client::new(ClientId::new(config.client_id.clone()));
    let client = if let Some(secret) = client_secret {
        client.set_client_secret(ClientSecret::new(secret))
    } else {
        client
    };

    let client: AppleClient = client
        .set_auth_uri(
            AuthUrl::new(APPLE_AUTH_URL.to_string())
                .map_err(|e| OAuthError::Configuration(e.to_string()))?,
        )
        .set_token_uri(
            TokenUrl::new(APPLE_TOKEN_URL.to_string())
                .map_err(|e| OAuthError::Configuration(e.to_string()))?,
        )
        .set_redirect_uri(
            RedirectUrl::new(config.redirect_uri.clone())
                .map_err(|e| OAuthError::Configuration(e.to_string()))?,
        );

    Ok(client)
}

#[derive(Debug, Serialize)]
struct AppleClientSecretClaims<'a> {
    iss: &'a str,
    iat: i64,
    exp: i64,
    aud: &'static str,
    sub: &'a str,
}

fn generate_client_secret(config: &AppleOAuthConfig) -> Result<String, OAuthError> {
    let now = chrono::Utc::now().timestamp();
    let claims = AppleClientSecretClaims {
        iss: &config.team_id,
        iat: now,
        exp: now + 300,
        aud: "https://appleid.apple.com",
        sub: &config.client_id,
    };
    let mut header = Header::new(Algorithm::ES256);
    header.kid = Some(config.key_id.clone());

    let key = EncodingKey::from_ec_pem(config.private_key_pem.as_bytes())
        .map_err(|e| OAuthError::Configuration(e.to_string()))?;
    jsonwebtoken::encode(&header, &claims, &key)
        .map_err(|e| OAuthError::Configuration(e.to_string()))
}

/// Decode the payload of an Apple id_token without cryptographic verification.
/// Full signature verification against Apple's JWKS should be performed at the
/// HTTP/middleware layer; this function only extracts claims for user identification.
fn decode_id_token_claims(id_token: &str) -> Result<AppleIdTokenClaims, OAuthError> {
    use base64::Engine;
    let parts: Vec<&str> = id_token.split('.').collect();
    if parts.len() != 3 {
        return Err(OAuthError::Exchange(
            "malformed id_token: expected 3 parts".into(),
        ));
    }

    let payload_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(parts[1])
        .map_err(|e| OAuthError::Exchange(format!("id_token base64 decode error: {e}")))?;

    serde_json::from_slice(&payload_bytes)
        .map_err(|e| OAuthError::Exchange(format!("id_token JSON parse error: {e}")))
}

#[cfg(test)]
mod tests {
    use base64::Engine;

    use super::*;

    fn config(redirect_uri: &str) -> AppleOAuthConfig {
        AppleOAuthConfig {
            client_id: "com.example.app".into(),
            team_id: "team-id".into(),
            key_id: "key-id".into(),
            private_key_pem: "unused-in-authorization".into(),
            redirect_uri: redirect_uri.into(),
        }
    }

    #[test]
    fn authorization_contract_uses_apple_scopes_form_post_and_fresh_state() {
        let apple = AppleOAuth::new(&config("https://example.com/auth/apple/callback")).unwrap();
        let first = apple.authorization_url();
        let second = apple.authorization_url();

        assert!(first.url.starts_with(APPLE_AUTH_URL));
        assert!(first.url.contains("scope=name+email"));
        assert!(first.url.contains("response_mode=form_post"));
        assert_eq!(first.issuer.as_deref(), Some(APPLE_ISSUER));
        assert_ne!(first.csrf_state, second.csrf_state);
        assert!(AppleOAuth::new(&config("not a URL")).is_err());
    }

    #[test]
    fn id_token_claim_extraction_accepts_valid_claims_and_rejects_malformed_payloads() {
        let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(r#"{"sub":"001234.abcdef","email":"user@example.com","email_verified":true}"#);
        let claims = decode_id_token_claims(&format!("header.{payload}.signature")).unwrap();
        assert_eq!(claims.sub, "001234.abcdef");
        assert_eq!(claims.email.as_deref(), Some("user@example.com"));
        assert_eq!(claims.email_verified, Some(true));

        for token in [
            "header.payload",
            "header.%%%invalid.signature",
            "header.e30.signature",
        ] {
            assert!(decode_id_token_claims(token).is_err(), "{token}");
        }
    }
}
