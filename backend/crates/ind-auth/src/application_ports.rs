use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use ind_application::ports::{
    AccountOperations, ApiTokenOperations, AuthOperations, AuthPortError, ExtensionAuthOperations,
    OAuthAuthorizationUrl, OAuthCallbackContext, OAuthCallbackResult, OAuthError, OAuthOperations,
    OAuthTokenResult, OnboardingOperations, TokenValidator, UserLookup, ValidatedApiToken,
};
use ind_application::repos::api_token::ApiTokenRepository;
use ind_application::repos::user::UserRepository;
use ind_domain::{ApiToken, ApiTokenId, ClientType, OAuthProvider, RefreshToken, User, UserId};

use crate::api_token::ApiTokenService;
use crate::oauth::OAuthService;
use crate::{
    AuthError, AuthService, AuthorizationCodeService, LoginRequest, LoginResponse,
    OnboardingStatus, RegisterRequest, RegisterResponse, UserProfile,
};

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub struct AuthOperationsService(pub AuthService);
pub struct ApiTokenOperationsService<R: ApiTokenRepository>(pub ApiTokenService<R>);
pub struct OAuthOperationsService(pub OAuthService);
pub struct ExtensionAuthOperationsService(pub AuthorizationCodeService);

pub struct UserLookupService {
    repo: Arc<dyn UserRepository>,
}

impl UserLookupService {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }
}

impl UserLookup for UserLookupService {
    fn get_user_by_id(&self, id: UserId) -> BoxFuture<'_, Result<Option<User>, AuthPortError>> {
        Box::pin(async move {
            self.repo
                .find_by_id(id)
                .await
                .map_err(AuthPortError::rejected)
        })
    }
}

impl<R: ApiTokenRepository> TokenValidator for ApiTokenOperationsService<R> {
    fn validate_api_token(
        &self,
        raw_token: &str,
    ) -> BoxFuture<'_, Result<ValidatedApiToken, AuthPortError>> {
        let token = raw_token.to_owned();
        Box::pin(async move {
            self.0
                .validate_api_token(&token)
                .await
                .map(|validated| ValidatedApiToken {
                    token: validated.token,
                })
                .map_err(AuthPortError::rejected)
        })
    }
}

impl AuthOperations for AuthOperationsService {
    fn register(
        &self,
        req: RegisterRequest,
        client_type: ClientType,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> BoxFuture<'_, Result<RegisterResponse, AuthError>> {
        Box::pin(self.0.register(req, client_type, ip, user_agent))
    }

    fn has_any_users(&self) -> BoxFuture<'_, Result<bool, AuthError>> {
        Box::pin(async move { self.0.has_any_users().await.map_err(AuthError::from) })
    }

    fn login(
        &self,
        req: LoginRequest,
        client_type: ClientType,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> BoxFuture<'_, Result<LoginResponse, AuthError>> {
        Box::pin(self.0.login(req, client_type, ip, user_agent))
    }

    fn refresh(
        &self,
        raw_refresh_token: &str,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> BoxFuture<'_, Result<crate::session::RefreshResult, AuthError>> {
        let token = raw_refresh_token.to_owned();
        Box::pin(async move { self.0.refresh(&token, ip, user_agent).await })
    }

    fn logout_by_refresh_token(
        &self,
        raw_refresh_token: &str,
    ) -> BoxFuture<'_, Result<(), AuthError>> {
        let token = raw_refresh_token.to_owned();
        Box::pin(async move { self.0.logout_by_refresh_token(&token).await })
    }

    fn logout_all(&self, user_id: UserId) -> BoxFuture<'_, Result<u64, AuthError>> {
        Box::pin(self.0.logout_all(user_id))
    }

    fn list_active_refresh_families(
        &self,
        user_id: UserId,
    ) -> BoxFuture<'_, Result<Vec<RefreshToken>, AuthError>> {
        Box::pin(self.0.list_active_refresh_families(user_id))
    }

    fn forgot_password(&self, email: &str) -> BoxFuture<'_, Result<Option<String>, AuthError>> {
        let email = email.to_owned();
        Box::pin(async move { self.0.forgot_password(&email).await })
    }

    fn reset_password(
        &self,
        raw_token: &str,
        new_password: &str,
    ) -> BoxFuture<'_, Result<User, AuthError>> {
        let token = raw_token.to_owned();
        let password = new_password.to_owned();
        Box::pin(async move { self.0.reset_password(&token, &password).await })
    }

    fn verify_email(&self, raw_token: &str) -> BoxFuture<'_, Result<User, AuthError>> {
        let token = raw_token.to_owned();
        Box::pin(async move { self.0.verify_email(&token).await })
    }

    fn resend_verification(
        &self,
        user_id: &UserId,
    ) -> BoxFuture<'_, Result<Option<String>, AuthError>> {
        let id = *user_id;
        Box::pin(async move { self.0.resend_verification(&id).await })
    }

    fn create_tokens_for_user(
        &self,
        user_id: UserId,
        client_type: ClientType,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> BoxFuture<'_, Result<OAuthTokenResult, AuthError>> {
        Box::pin(async move {
            let (refresh_token, raw_refresh_token, access_token, expires_at) = self
                .0
                .create_tokens_for_user(user_id, client_type, ip, user_agent)
                .await?;
            Ok(OAuthTokenResult {
                refresh_token,
                raw_refresh_token,
                access_token,
                expires_at,
            })
        })
    }
}

impl AccountOperations for AuthOperationsService {
    fn get_profile(&self, user_id: UserId) -> BoxFuture<'_, Result<UserProfile, AuthError>> {
        Box::pin(self.0.get_profile(user_id))
    }

    fn update_profile(
        &self,
        user_id: UserId,
        req: crate::UpdateProfileRequest,
    ) -> BoxFuture<'_, Result<UserProfile, AuthError>> {
        Box::pin(self.0.update_profile(user_id, req))
    }

    fn change_password(
        &self,
        user_id: UserId,
        req: crate::ChangePasswordRequest,
    ) -> BoxFuture<'_, Result<(), AuthError>> {
        Box::pin(self.0.change_password(user_id, req))
    }

    fn change_email(
        &self,
        user_id: UserId,
        new_email: String,
        password: String,
    ) -> BoxFuture<'_, Result<(), AuthError>> {
        Box::pin(async move {
            self.0.change_email(&user_id, &new_email, &password).await?;
            Ok(())
        })
    }

    fn delete_account(
        &self,
        user_id: UserId,
        confirmation: String,
    ) -> BoxFuture<'_, Result<(), AuthError>> {
        Box::pin(
            self.0
                .delete_account(user_id, crate::DeleteAccountRequest { confirmation }),
        )
    }
}

impl OnboardingOperations for AuthOperationsService {
    fn get_onboarding(
        &self,
        user_id: UserId,
    ) -> BoxFuture<'_, Result<OnboardingStatus, AuthError>> {
        Box::pin(self.0.get_onboarding_status(user_id))
    }

    fn complete_step(
        &self,
        user_id: UserId,
        step: i16,
        _data: serde_json::Value,
    ) -> BoxFuture<'_, Result<OnboardingStatus, AuthError>> {
        Box::pin(self.0.advance_onboarding(user_id, step))
    }

    fn skip_onboarding(
        &self,
        user_id: UserId,
    ) -> BoxFuture<'_, Result<OnboardingStatus, AuthError>> {
        Box::pin(self.0.complete_onboarding(user_id))
    }
}

impl<R: ApiTokenRepository> ApiTokenOperations for ApiTokenOperationsService<R> {
    fn list_tokens(&self, user_id: UserId) -> BoxFuture<'_, Result<Vec<ApiToken>, AuthError>> {
        Box::pin(self.0.list_api_tokens(user_id))
    }

    fn create_token(
        &self,
        user_id: UserId,
        name: String,
        scopes: Vec<String>,
        expires_in: Option<chrono::Duration>,
    ) -> BoxFuture<'_, Result<(ApiToken, String), AuthError>> {
        Box::pin(async move {
            let response = self
                .0
                .create_api_token(crate::CreateApiTokenRequest {
                    user_id,
                    name,
                    scopes: crate::api_token::strings_to_scopes(&scopes)?,
                    expires_in,
                })
                .await?;
            Ok((response.token, response.raw_token))
        })
    }

    fn revoke_token(
        &self,
        user_id: UserId,
        token_id: ApiTokenId,
    ) -> BoxFuture<'_, Result<(), AuthError>> {
        Box::pin(self.0.revoke_api_token(user_id, token_id))
    }
}

impl ExtensionAuthOperations for ExtensionAuthOperationsService {
    fn create_authorization_code(
        &self,
        user_id: UserId,
        client_type: ClientType,
        code_challenge: String,
        code_challenge_method: String,
        redirect_uri: String,
    ) -> BoxFuture<'_, Result<String, AuthError>> {
        Box::pin(async move {
            Ok(self
                .0
                .create_code(
                    user_id,
                    client_type,
                    code_challenge,
                    code_challenge_method,
                    redirect_uri,
                )
                .await?
                .raw_code)
        })
    }

    fn exchange_authorization_code(
        &self,
        raw_code: &str,
        code_verifier: &str,
        redirect_uri: &str,
        client_type: ClientType,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> BoxFuture<'_, Result<ind_application::ports::ExtensionTokenResult, AuthError>> {
        let code = raw_code.to_owned();
        let verifier = code_verifier.to_owned();
        let uri = redirect_uri.to_owned();
        Box::pin(async move {
            let result = self
                .0
                .exchange_code(&code, &verifier, &uri, client_type, ip, user_agent)
                .await?;
            Ok(ind_application::ports::ExtensionTokenResult {
                access_token: result.access_token,
                refresh_token: result.refresh_token,
                expires_at: result.expires_at,
                refresh_token_expires_at: result.refresh_token_expires_at,
            })
        })
    }
}

impl OAuthOperations for OAuthOperationsService {
    fn oauth_start(
        &self,
        provider: OAuthProvider,
    ) -> BoxFuture<'_, Result<OAuthAuthorizationUrl, OAuthError>> {
        Box::pin(async move { self.0.oauth_start(provider).await })
    }

    fn oauth_callback(
        &self,
        provider: OAuthProvider,
        code: &str,
        state: &str,
        context: OAuthCallbackContext,
    ) -> BoxFuture<'_, Result<OAuthCallbackResult, OAuthError>> {
        let code = code.to_owned();
        let state = state.to_owned();
        Box::pin(async move {
            self.0
                .oauth_callback(provider, &code, &state, context)
                .await
        })
    }
}
