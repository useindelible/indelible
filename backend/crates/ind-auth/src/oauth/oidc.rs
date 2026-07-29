use openidconnect::core::{
    CoreAuthenticationFlow, CoreClient, CoreGenderClaim, CoreProviderMetadata,
};
use openidconnect::reqwest;
use openidconnect::{
    AccessTokenHash, AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce,
    OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse,
};

use super::{OAuthAuthorizationUrl, OAuthUserInfo, OidcFlow, OidcOAuthConfig, error::OAuthError};
use ind_domain::OAuthProvider;

pub struct OidcOAuth {
    config: OidcOAuthConfig,
    http_client: reqwest::Client,
}

impl OidcOAuth {
    pub fn new(config: &OidcOAuthConfig) -> Result<Self, OAuthError> {
        IssuerUrl::new(config.issuer_url.clone())
            .map_err(|e| OAuthError::Configuration(e.to_string()))?;
        RedirectUrl::new(config.redirect_uri.clone())
            .map_err(|e| OAuthError::Configuration(e.to_string()))?;

        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| OAuthError::Configuration(e.to_string()))?;

        Ok(Self {
            config: config.clone(),
            http_client,
        })
    }

    pub async fn authorization_url(&self) -> Result<OAuthAuthorizationUrl, OAuthError> {
        let metadata = self.provider_metadata().await?;
        let client = self.client_from_metadata(metadata)?;
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let mut request = client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .set_pkce_challenge(pkce_challenge);

        for scope in normalized_scopes(&self.config.scopes)
            .into_iter()
            .filter(|scope| scope != "openid")
        {
            request = request.add_scope(Scope::new(scope));
        }

        let (url, csrf_state, nonce) = request.url();
        let csrf_state = csrf_state.secret().clone();

        Ok(OAuthAuthorizationUrl {
            url: url.to_string(),
            csrf_state: csrf_state.clone(),
            issuer: Some(self.config.issuer_url.clone()),
            oidc_flow: Some(OidcFlow {
                csrf_state,
                nonce: nonce.secret().clone(),
                pkce_verifier: pkce_verifier.secret().clone(),
            }),
        })
    }

    pub async fn exchange_code(
        &self,
        code: &str,
        flow: &OidcFlow,
    ) -> Result<OAuthUserInfo, OAuthError> {
        let metadata = self.provider_metadata().await?;
        let client = self.client_from_metadata(metadata)?;
        let token_response = client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .map_err(|e| OAuthError::Exchange(e.to_string()))?
            .set_pkce_verifier(PkceCodeVerifier::new(flow.pkce_verifier.clone()))
            .request_async(&self.http_client)
            .await
            .map_err(|e| OAuthError::Exchange(e.to_string()))?;

        let id_token = token_response.id_token().ok_or_else(|| {
            OAuthError::Exchange("OIDC provider did not return an id_token".into())
        })?;
        let verifier = client.id_token_verifier();
        let nonce = Nonce::new(flow.nonce.clone());
        let claims = id_token
            .claims(&verifier, &nonce)
            .map_err(|e| OAuthError::Exchange(e.to_string()))?;

        if let Some(expected_access_token_hash) = claims.access_token_hash() {
            let actual_access_token_hash = AccessTokenHash::from_token(
                token_response.access_token(),
                id_token
                    .signing_alg()
                    .map_err(|e| OAuthError::Exchange(e.to_string()))?,
                id_token
                    .signing_key(&verifier)
                    .map_err(|e| OAuthError::Exchange(e.to_string()))?,
            )
            .map_err(|e| OAuthError::Exchange(e.to_string()))?;
            if actual_access_token_hash != *expected_access_token_hash {
                return Err(OAuthError::Exchange(
                    "OIDC access token hash verification failed".into(),
                ));
            }
        }

        let expected_subject = claims.subject().clone();
        let mut email = claims.email().map(|email| email.as_str().to_string());
        let mut email_verified = claims.email_verified();
        let mut display_name = localized_claim_to_string(claims.name()).or_else(|| {
            claims
                .preferred_username()
                .map(|name| name.as_str().to_string())
        });
        let mut avatar_url = localized_claim_to_string(claims.picture());

        if let Ok(request) = client.user_info(
            token_response.access_token().to_owned(),
            Some(expected_subject.clone()),
        )
            && let Ok(userinfo) = request
                .request_async::<openidconnect::EmptyAdditionalClaims, reqwest::Client, CoreGenderClaim>(
                    &self.http_client,
                )
                .await
        {
            email = email.or_else(|| userinfo.email().map(|value| value.as_str().to_string()));
            email_verified = email_verified.or(userinfo.email_verified());
            display_name = display_name
                .or_else(|| localized_claim_to_string(userinfo.name()))
                .or_else(|| {
                    userinfo
                        .preferred_username()
                        .map(|value| value.as_str().to_string())
                });
            avatar_url = avatar_url.or_else(|| localized_claim_to_string(userinfo.picture()));
        }

        Ok(OAuthUserInfo {
            provider: OAuthProvider::Oidc,
            provider_user_id: expected_subject.as_str().to_string(),
            email,
            display_name,
            avatar_url,
            access_token: token_response.access_token().secret().clone(),
            refresh_token: token_response
                .refresh_token()
                .map(|token| token.secret().clone()),
            email_verified,
            allow_auto_create: self.config.auto_create_users,
        })
    }

    async fn provider_metadata(&self) -> Result<CoreProviderMetadata, OAuthError> {
        let issuer = IssuerUrl::new(self.config.issuer_url.clone())
            .map_err(|e| OAuthError::Configuration(e.to_string()))?;
        CoreProviderMetadata::discover_async(issuer, &self.http_client)
            .await
            .map_err(|e| OAuthError::Exchange(format!("{e:?}")))
    }

    fn client_from_metadata(
        &self,
        metadata: CoreProviderMetadata,
    ) -> Result<
        CoreClient<
            openidconnect::EndpointSet,
            openidconnect::EndpointNotSet,
            openidconnect::EndpointNotSet,
            openidconnect::EndpointNotSet,
            openidconnect::EndpointMaybeSet,
            openidconnect::EndpointMaybeSet,
        >,
        OAuthError,
    > {
        Ok(CoreClient::from_provider_metadata(
            metadata,
            ClientId::new(self.config.client_id.clone()),
            Some(ClientSecret::new(self.config.client_secret.clone())),
        )
        .set_redirect_uri(
            RedirectUrl::new(self.config.redirect_uri.clone())
                .map_err(|e| OAuthError::Configuration(e.to_string()))?,
        ))
    }
}

fn normalized_scopes(configured: &[String]) -> Vec<String> {
    let mut scopes = configured
        .iter()
        .map(|scope| scope.trim())
        .filter(|scope| !scope.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    if !scopes.iter().any(|scope| scope == "openid") {
        scopes.insert(0, "openid".to_string());
    }

    scopes
}

fn localized_claim_to_string<T>(claim: Option<&openidconnect::LocalizedClaim<T>>) -> Option<String>
where
    T: std::ops::Deref<Target = String>,
{
    claim
        .and_then(|values| {
            values
                .get(None)
                .or_else(|| values.iter().next().map(|(_, value)| value))
        })
        .map(|value| value.as_str().to_string())
}
