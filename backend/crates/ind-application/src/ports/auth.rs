use futures::future::BoxFuture;
use ind_domain::{
    ApiToken, ApiTokenId, ClientType, OAuthProvider, RefreshToken, Theme, User, UserId,
};

use crate::AppError;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("account not found")]
    AccountNotFound,

    #[error("account disabled")]
    AccountDisabled,

    #[error("email not verified")]
    EmailNotVerified,

    #[error("session expired")]
    SessionExpired,

    #[error("session not found")]
    SessionNotFound,

    #[error("token expired")]
    TokenExpired,

    #[error("token invalid")]
    TokenInvalid,

    #[error("token revoked")]
    TokenRevoked,

    #[error("token already used")]
    TokenAlreadyUsed,

    #[error("password hash error: {0}")]
    HashError(String),

    #[error("password too weak")]
    PasswordTooWeak,

    #[error("email already exists")]
    EmailAlreadyExists,

    #[error("rate limited")]
    RateLimited,

    #[error("repository error: {0}")]
    Repo(#[from] AppError),

    #[error("insufficient scope")]
    InsufficientScope,

    #[error("validation error: {field}: {message}")]
    ValidationError { field: String, message: String },

    #[error("confirmation required")]
    ConfirmationRequired,

    #[error("signups are disabled")]
    SignupsDisabled,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{message}")]
pub struct AuthPortError {
    message: String,
}

impl AuthPortError {
    pub fn rejected(error: impl ToString) -> Self {
        Self {
            message: error.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValidatedApiToken {
    pub token: ApiToken,
}

pub trait TokenValidator: Send + Sync {
    fn validate_api_token(
        &self,
        raw_token: &str,
    ) -> BoxFuture<'_, Result<ValidatedApiToken, AuthPortError>>;
}

pub trait UserLookup: Send + Sync {
    fn get_user_by_id(&self, id: UserId) -> BoxFuture<'_, Result<Option<User>, AuthPortError>>;
}

pub trait ApiTokenOperations: Send + Sync {
    fn list_tokens(&self, user_id: UserId) -> BoxFuture<'_, Result<Vec<ApiToken>, AuthError>>;

    fn create_token(
        &self,
        user_id: UserId,
        name: String,
        scopes: Vec<String>,
        expires_in: Option<chrono::Duration>,
    ) -> BoxFuture<'_, Result<(ApiToken, String), AuthError>>;

    fn revoke_token(
        &self,
        user_id: UserId,
        token_id: ApiTokenId,
    ) -> BoxFuture<'_, Result<(), AuthError>>;
}

#[derive(Debug, Clone)]
pub struct OnboardingStatus {
    pub current_step: i16,
    pub completed: bool,
    pub steps: Vec<OnboardingStepInfo>,
}

#[derive(Debug, Clone)]
pub struct OnboardingStepInfo {
    pub step: i16,
    pub name: String,
    pub completed: bool,
}

pub trait OnboardingOperations: Send + Sync {
    fn get_onboarding(&self, user_id: UserId)
    -> BoxFuture<'_, Result<OnboardingStatus, AuthError>>;

    fn complete_step(
        &self,
        user_id: UserId,
        step: i16,
        data: serde_json::Value,
    ) -> BoxFuture<'_, Result<OnboardingStatus, AuthError>>;

    fn skip_onboarding(
        &self,
        user_id: UserId,
    ) -> BoxFuture<'_, Result<OnboardingStatus, AuthError>>;
}

#[derive(Debug)]
pub struct ExtensionTokenResult {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
    pub refresh_token_expires_at: i64,
}

pub trait ExtensionAuthOperations: Send + Sync {
    fn create_authorization_code(
        &self,
        user_id: UserId,
        client_type: ind_domain::ClientType,
        code_challenge: String,
        code_challenge_method: String,
        redirect_uri: String,
    ) -> BoxFuture<'_, Result<String, AuthError>>;

    fn exchange_authorization_code(
        &self,
        raw_code: &str,
        code_verifier: &str,
        redirect_uri: &str,
        client_type: ind_domain::ClientType,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> BoxFuture<'_, Result<ExtensionTokenResult, AuthError>>;
}

#[derive(Debug, thiserror::Error)]
pub enum OAuthError {
    #[error("OAuth configuration error: {0}")]
    Configuration(String),

    #[error("OAuth exchange error: {0}")]
    Exchange(String),

    #[error("OAuth provider {0:?} is not configured")]
    ProviderNotConfigured(OAuthProvider),

    #[error("OAuth state mismatch")]
    InvalidState,

    #[error("OAuth identity already linked to another account")]
    IdentityAlreadyLinked,

    #[error("cannot unlink the only authentication method")]
    CannotUnlinkOnly,

    #[error("OAuth identity not found")]
    IdentityNotFound,

    #[error("user account is deactivated")]
    UserDeactivated,

    #[error(transparent)]
    App(#[from] AppError),
}

#[derive(Debug)]
pub struct OAuthAuthorizationUrl {
    pub url: String,
    pub csrf_state: String,
    pub issuer: Option<String>,
    pub oidc_flow: Option<OidcFlow>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OidcFlow {
    pub csrf_state: String,
    pub nonce: String,
    pub pkce_verifier: String,
}

pub struct OAuthCallbackResult {
    pub user: User,
    pub is_new_user: bool,
}

#[derive(Debug, Clone)]
pub struct OAuthCallbackContext {
    pub expected_state: String,
    pub oidc_flow: Option<OidcFlow>,
}

pub trait OAuthOperations: Send + Sync {
    fn oauth_start(
        &self,
        provider: OAuthProvider,
    ) -> BoxFuture<'_, Result<OAuthAuthorizationUrl, OAuthError>>;

    fn oauth_callback(
        &self,
        provider: OAuthProvider,
        code: &str,
        state: &str,
        context: OAuthCallbackContext,
    ) -> BoxFuture<'_, Result<OAuthCallbackResult, OAuthError>>;
}

#[derive(Debug, Clone)]
pub struct UserProfile {
    pub id: UserId,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub locale: String,
    pub timezone: String,
    pub theme: Theme,
    pub email_verified: bool,
    pub onboarding_completed: bool,
    pub has_password: bool,
    pub email_token: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub avatar_url: Option<Option<String>>,
    pub locale: Option<String>,
    pub timezone: Option<String>,
    pub theme: Option<Theme>,
}

pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

pub struct DeleteAccountRequest {
    pub confirmation: String,
}

pub trait AccountOperations: Send + Sync {
    fn get_profile(&self, user_id: UserId) -> BoxFuture<'_, Result<UserProfile, AuthError>>;

    fn update_profile(
        &self,
        user_id: UserId,
        req: UpdateProfileRequest,
    ) -> BoxFuture<'_, Result<UserProfile, AuthError>>;

    fn change_password(
        &self,
        user_id: UserId,
        req: ChangePasswordRequest,
    ) -> BoxFuture<'_, Result<(), AuthError>>;

    fn change_email(
        &self,
        user_id: UserId,
        new_email: String,
        password: String,
    ) -> BoxFuture<'_, Result<(), AuthError>>;

    fn delete_account(
        &self,
        user_id: UserId,
        confirmation: String,
    ) -> BoxFuture<'_, Result<(), AuthError>>;
}

pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: String,
}

#[derive(Debug)]
pub struct RegisterResponse {
    pub user: User,
    pub access_token: String,
    pub expires_at: i64,
    pub raw_refresh_token: String,
    pub refresh_token: RefreshToken,
    pub verification_token_sent: bool,
}

pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug)]
pub struct LoginResponse {
    pub user: User,
    pub access_token: String,
    pub expires_at: i64,
    pub raw_refresh_token: String,
    pub refresh_token: RefreshToken,
}

#[derive(Debug)]
pub struct RefreshResult {
    pub access_token: String,
    pub expires_at: i64,
    pub raw_refresh_token: String,
}

#[derive(Debug)]
pub struct OAuthTokenResult {
    pub refresh_token: RefreshToken,
    pub raw_refresh_token: String,
    pub access_token: String,
    pub expires_at: i64,
}

pub trait AuthOperations: Send + Sync {
    fn register(
        &self,
        req: RegisterRequest,
        client_type: ClientType,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> BoxFuture<'_, Result<RegisterResponse, AuthError>>;

    fn login(
        &self,
        req: LoginRequest,
        client_type: ClientType,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> BoxFuture<'_, Result<LoginResponse, AuthError>>;

    fn refresh(
        &self,
        raw_refresh_token: &str,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> BoxFuture<'_, Result<RefreshResult, AuthError>>;

    fn logout_by_refresh_token(
        &self,
        raw_refresh_token: &str,
    ) -> BoxFuture<'_, Result<(), AuthError>>;

    fn logout_all(&self, user_id: UserId) -> BoxFuture<'_, Result<u64, AuthError>>;

    fn list_active_refresh_families(
        &self,
        user_id: UserId,
    ) -> BoxFuture<'_, Result<Vec<RefreshToken>, AuthError>>;

    fn forgot_password(&self, email: &str) -> BoxFuture<'_, Result<Option<String>, AuthError>>;

    fn reset_password(
        &self,
        raw_token: &str,
        new_password: &str,
    ) -> BoxFuture<'_, Result<User, AuthError>>;

    fn verify_email(&self, raw_token: &str) -> BoxFuture<'_, Result<User, AuthError>>;

    fn resend_verification(
        &self,
        user_id: &UserId,
    ) -> BoxFuture<'_, Result<Option<String>, AuthError>>;

    fn create_tokens_for_user(
        &self,
        user_id: UserId,
        client_type: ClientType,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> BoxFuture<'_, Result<OAuthTokenResult, AuthError>>;

    fn has_any_users(&self) -> BoxFuture<'_, Result<bool, AuthError>>;
}
