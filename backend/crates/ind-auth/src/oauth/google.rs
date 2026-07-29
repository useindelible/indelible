use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointNotSet, EndpointSet,
    RedirectUrl, RefreshToken, Scope, TokenResponse, TokenUrl,
};
use serde::Deserialize;

use super::{GoogleOAuthConfig, OAuthAuthorizationUrl, OAuthUserInfo, error::OAuthError};
use ind_domain::OAuthProvider;

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v3/userinfo";
const GOOGLE_ISSUER: &str = "https://accounts.google.com";

type ConfiguredClient =
    BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>;

#[derive(Debug, Deserialize)]
struct GoogleUserInfo {
    sub: String,
    email: Option<String>,
    name: Option<String>,
    picture: Option<String>,
}

pub struct GoogleOAuth {
    client: ConfiguredClient,
    http_client: oauth2::reqwest::Client,
}

impl GoogleOAuth {
    pub fn new(config: &GoogleOAuthConfig) -> Result<Self, OAuthError> {
        let client = BasicClient::new(ClientId::new(config.client_id.clone()))
            .set_client_secret(ClientSecret::new(config.client_secret.clone()))
            .set_auth_uri(
                AuthUrl::new(GOOGLE_AUTH_URL.to_string())
                    .map_err(|e| OAuthError::Configuration(e.to_string()))?,
            )
            .set_token_uri(
                TokenUrl::new(GOOGLE_TOKEN_URL.to_string())
                    .map_err(|e| OAuthError::Configuration(e.to_string()))?,
            )
            .set_redirect_uri(
                RedirectUrl::new(config.redirect_uri.clone())
                    .map_err(|e| OAuthError::Configuration(e.to_string()))?,
            );

        Ok(Self {
            client,
            http_client: oauth2::reqwest::Client::new(),
        })
    }

    pub fn authorization_url(&self) -> OAuthAuthorizationUrl {
        let (url, csrf_state) = self
            .client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("openid".into()))
            .add_scope(Scope::new("email".into()))
            .add_scope(Scope::new("profile".into()))
            .add_extra_param("access_type", "offline")
            .add_extra_param("prompt", "consent")
            .url();

        OAuthAuthorizationUrl {
            url: url.to_string(),
            csrf_state: csrf_state.secret().clone(),
            issuer: Some(GOOGLE_ISSUER.to_string()),
            oidc_flow: None,
        }
    }

    pub async fn exchange_code(&self, code: &str) -> Result<OAuthUserInfo, OAuthError> {
        let token_response = self
            .client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .request_async(&self.http_client)
            .await
            .map_err(|e| OAuthError::Exchange(e.to_string()))?;

        let access_token = token_response.access_token().secret().clone();

        let response_body = self
            .http_client
            .get(GOOGLE_USERINFO_URL)
            .bearer_auth(&access_token)
            .send()
            .await
            .map_err(|e| OAuthError::Exchange(e.to_string()))?
            .error_for_status()
            .map_err(|e| OAuthError::Exchange(e.to_string()))?
            .text()
            .await
            .map_err(|e| OAuthError::Exchange(e.to_string()))?;

        let user_info: GoogleUserInfo = serde_json::from_str(&response_body)
            .map_err(|e| OAuthError::Exchange(e.to_string()))?;
        let email_verified = user_info.email.as_ref().map(|_| true);

        Ok(OAuthUserInfo {
            provider: OAuthProvider::Google,
            provider_user_id: user_info.sub,
            email: user_info.email,
            display_name: user_info.name,
            avatar_url: user_info.picture,
            access_token,
            refresh_token: token_response
                .refresh_token()
                .map(|t: &RefreshToken| t.secret().clone()),
            email_verified,
            allow_auto_create: true,
        })
    }
}
