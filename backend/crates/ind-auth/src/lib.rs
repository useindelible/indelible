pub mod api_token;
mod application_ports;
pub mod asset_cookie;
pub mod authorization_code;
pub mod credentials;
pub mod crypto;
pub mod error;
pub mod integration_oauth;
pub mod jwt;
pub mod login;
pub mod oauth;
pub mod oauth_flow;
pub mod onboarding;
pub mod password_reset;
pub mod profile;
pub mod refresh_token;
pub mod register;
pub mod service;
pub mod session;
mod validation;
pub mod verification;
pub mod webhook_secret;
pub mod webhooks;

pub use api_token::{
    ApiTokenService, CreateApiTokenRequest, CreateApiTokenResponse, TokenScope, ValidatedToken,
    has_scope,
};
pub use application_ports::{
    ApiTokenOperationsService, AuthOperationsService, ExtensionAuthOperationsService,
    OAuthOperationsService, UserLookupService,
};
pub use asset_cookie::{
    ASSET_COOKIE_MAX_AGE_SECS, AssetCookieSecretError, decode_asset_cookie_secret,
    sign_asset_cookie, verify_asset_cookie,
};
pub use authorization_code::AuthorizationCodeService;
pub use credentials::{CipherError, CredentialCipher};
pub use crypto::{
    generate_api_token, generate_authorization_code, generate_email_token,
    generate_password_reset_token, generate_refresh_token, generate_session_token,
    generate_verification_token, hash_password, hash_token, verify_password,
};
pub use error::AuthError;
pub use integration_oauth::{
    CompletedIntegrationFlow, IntegrationOAuthError, IntegrationOAuthFlowStore,
    IntegrationOAuthProviderAdapter, IntegrationOAuthService, ProviderTokens,
    RepositoryIntegrationOAuthFlowStore, StartedIntegrationFlow,
    integration_oauth_error_to_app_error,
    notion::NotionOAuthAdapter,
    settings::{IntegrationNotionOAuthSettings, IntegrationOAuthSettings},
};
pub use jwt::{JwtClaims, sign_access_token, validate_access_token};
pub use login::{LoginRequest, LoginResponse};
pub use oauth_flow::{
    NativeOAuthFlow, OAuthFlowError, OAuthFlowStorageError, StoredOAuthFlow, StoredOAuthFlowKind,
    consume_oauth_flow, open_oauth_flow, seal_oauth_flow, store_oauth_flow,
};
pub use onboarding::OnboardingStatus;
pub use profile::{ChangePasswordRequest, DeleteAccountRequest, UpdateProfileRequest, UserProfile};
pub use refresh_token::RefreshTokenService;
pub use register::{RegisterRequest, RegisterResponse};
pub use service::AuthService;
pub use webhook_secret::{
    WebhookSecretOpenError, generate_webhook_secret, open_webhook_secret, seal_webhook_secret,
    webhook_secret_hash, webhook_secret_preview,
};
