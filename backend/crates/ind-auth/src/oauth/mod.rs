pub mod apple;
pub mod error;
pub mod google;
pub mod oidc;
pub mod service;

pub use ind_application::ports::{OAuthAuthorizationUrl, OidcFlow};
use ind_domain::OAuthProvider;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct GoogleOAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppleOAuthConfig {
    pub client_id: String,
    pub team_id: String,
    pub key_id: String,
    pub private_key_pem: String,
    pub redirect_uri: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OidcOAuthConfig {
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub provider_name: String,
    pub scopes: Vec<String>,
    pub auto_create_users: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct OAuthConfig {
    pub google: Option<GoogleOAuthConfig>,
    pub apple: Option<AppleOAuthConfig>,
    pub oidc: Option<OidcOAuthConfig>,
}

#[derive(Debug, Clone, Copy)]
pub struct OAuthConfigInput<'a> {
    pub google_client_id: Option<&'a str>,
    pub google_client_secret: Option<&'a str>,
    pub apple_client_id: Option<&'a str>,
    pub apple_team_id: Option<&'a str>,
    pub apple_key_id: Option<&'a str>,
    pub apple_private_key_pem: Option<&'a str>,
    pub oidc_enabled: bool,
    pub oidc_issuer_url: Option<&'a str>,
    pub oidc_client_id: Option<&'a str>,
    pub oidc_client_secret: Option<&'a str>,
    pub oidc_provider_name: &'a str,
    pub oidc_scopes: &'a [String],
    pub oidc_auto_create_users: bool,
    pub base_url: &'a str,
}

impl OAuthConfig {
    pub fn google(&self) -> Option<&GoogleOAuthConfig> {
        self.google.as_ref()
    }

    pub fn apple(&self) -> Option<&AppleOAuthConfig> {
        self.apple.as_ref()
    }

    pub fn oidc(&self) -> Option<&OidcOAuthConfig> {
        self.oidc.as_ref()
    }

    pub fn provider(&self, provider: OAuthProvider) -> Option<OAuthProviderConfigRef<'_>> {
        match provider {
            OAuthProvider::Google => self.google.as_ref().map(OAuthProviderConfigRef::Google),
            OAuthProvider::Apple => self.apple.as_ref().map(OAuthProviderConfigRef::Apple),
            OAuthProvider::Oidc => self.oidc.as_ref().map(OAuthProviderConfigRef::Oidc),
        }
    }

    pub fn configured_providers(&self) -> Vec<OAuthProvider> {
        let mut providers = Vec::new();
        if self.google.is_some() {
            providers.push(OAuthProvider::Google);
        }
        if self.apple.is_some() {
            providers.push(OAuthProvider::Apple);
        }
        if self.oidc.is_some() {
            providers.push(OAuthProvider::Oidc);
        }
        providers
    }
}

pub fn build_oauth_config(input: OAuthConfigInput<'_>) -> Option<OAuthConfig> {
    let google = match (input.google_client_id, input.google_client_secret) {
        (Some(id), Some(secret)) => Some(GoogleOAuthConfig {
            client_id: id.to_string(),
            client_secret: secret.to_string(),
            redirect_uri: format!("{}/api/v1/auth/oauth/google/callback", input.base_url),
        }),
        _ => None,
    };

    let apple = match (
        input.apple_client_id,
        input.apple_team_id,
        input.apple_key_id,
        input.apple_private_key_pem,
    ) {
        (Some(client_id), Some(team_id), Some(key_id), Some(private_key_pem)) => {
            Some(AppleOAuthConfig {
                client_id: client_id.to_string(),
                team_id: team_id.to_string(),
                key_id: key_id.to_string(),
                private_key_pem: private_key_pem.to_string(),
                redirect_uri: format!("{}/api/v1/auth/oauth/apple/callback", input.base_url),
            })
        }
        _ => None,
    };

    let oidc = match (
        input.oidc_enabled,
        input.oidc_issuer_url,
        input.oidc_client_id,
        input.oidc_client_secret,
    ) {
        (true, Some(issuer_url), Some(client_id), Some(client_secret)) => Some(OidcOAuthConfig {
            issuer_url: issuer_url.to_string(),
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            redirect_uri: format!("{}/api/v1/auth/oauth/oidc/callback", input.base_url),
            provider_name: if input.oidc_provider_name.trim().is_empty() {
                "OpenID Connect".to_string()
            } else {
                input.oidc_provider_name.to_string()
            },
            scopes: if input.oidc_scopes.is_empty() {
                vec![
                    "openid".to_string(),
                    "email".to_string(),
                    "profile".to_string(),
                ]
            } else {
                input.oidc_scopes.to_vec()
            },
            auto_create_users: input.oidc_auto_create_users,
        }),
        _ => None,
    };

    if google.is_none() && apple.is_none() && oidc.is_none() {
        None
    } else {
        Some(OAuthConfig {
            google,
            apple,
            oidc,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OAuthProviderConfigRef<'a> {
    Google(&'a GoogleOAuthConfig),
    Apple(&'a AppleOAuthConfig),
    Oidc(&'a OidcOAuthConfig),
}

#[derive(Debug)]
pub struct OAuthUserInfo {
    pub provider: OAuthProvider,
    pub provider_user_id: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub email_verified: Option<bool>,
    pub allow_auto_create: bool,
}

pub use error::OAuthError;
pub use service::{OAuthCallbackContext, OAuthCallbackResult, OAuthService};
