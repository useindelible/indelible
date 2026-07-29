use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Redirect};
use ind_domain::IntegrationOAuthProvider;

use crate::error::ApiError;
use crate::extract::ValidatedJson;
use crate::middleware::AccountAccess;
use crate::response::ApiResponse;
use crate::state::AppState;

use super::dto::{AuthorizeIntegrationRequest, AuthorizeIntegrationResponse, CallbackQuery};

#[utoipa::path(
    post,
    path = "/api/v1/integrations/{provider}/authorize",
    params(("provider" = String, Path, description = "Integration provider slug")),
    request_body = AuthorizeIntegrationRequest,
    responses(
        (status = 200, description = "OAuth authorize URL", body = AuthorizeIntegrationResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Provider not configured"),
    ),
    security(("session_cookie" = [])),
    tag = "Integrations",
)]
pub async fn authorize_integration(
    AccountAccess(auth_user): AccountAccess,
    State(state): State<AppState>,
    Path(provider): Path<String>,
    ValidatedJson(body): ValidatedJson<AuthorizeIntegrationRequest>,
) -> Result<ApiResponse<AuthorizeIntegrationResponse>, ApiError> {
    let provider = parse_oauth_provider(&provider)?;
    let ops = state.integration_ops.as_ref().ok_or(ApiError::NotFound {
        entity: "integrations",
        id: provider.as_str().to_string(),
    })?;

    let start = ops
        .authorize(auth_user.user_id, provider, body.redirect_after)
        .await
        .map_err(ApiError::from)?;

    Ok(ApiResponse::new(AuthorizeIntegrationResponse {
        authorize_url: start.authorize_url,
    }))
}

/// Callback failure category. The frontend hub reads
/// `integration_error={kind}&provider={p}` and renders a banner accordingly.
fn redirect_with_error(state: &AppState, provider_slug: &str, kind: &str) -> Redirect {
    // `provider_slug` may come from a request path before parsing succeeded;
    // build the URL via `url::Url` so the query string is percent-encoded and
    // an attacker can't smuggle extra query params via `&` in the slug.
    let frontend = state.config.frontend_url.trim_end_matches('/');
    let target = match url::Url::parse(&format!("{frontend}/preferences/integrations")) {
        Ok(mut url) => {
            url.query_pairs_mut()
                .append_pair("integration_error", kind)
                .append_pair("provider", provider_slug);
            url.into()
        }
        Err(_) => format!("{frontend}/preferences/integrations?integration_error={kind}"),
    };
    Redirect::temporary(&target)
}

#[utoipa::path(
    get,
    path = "/api/v1/integrations/{provider}/callback",
    params(
        ("provider" = String, Path, description = "Integration provider slug"),
        ("code" = Option<String>, Query, description = "OAuth authorization code"),
        ("state" = Option<String>, Query, description = "OAuth state token"),
        ("error" = Option<String>, Query, description = "OAuth error code from provider"),
        ("error_description" = Option<String>, Query, description = "OAuth error description"),
    ),
    responses(
        (status = 302, description = "Always redirects to the frontend integrations hub. On success: ?connected={provider}. On failure: ?integration_error={denied|provider_error|server}&provider={provider}."),
    ),
    tag = "Integrations",
)]
pub async fn integration_callback(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Query(query): Query<CallbackQuery>,
) -> impl IntoResponse {
    let provider_slug = provider.clone();
    let parsed_provider = match parse_oauth_provider(&provider) {
        Ok(p) => p,
        Err(_) => {
            tracing::warn!(provider = %provider_slug, "integration callback: unknown provider");
            return redirect_with_error(&state, &provider_slug, "server");
        }
    };
    let provider_str = parsed_provider.as_str();

    if let Some(error) = query.error.as_deref() {
        tracing::info!(
            provider = provider_str,
            error,
            description = query.error_description.as_deref().unwrap_or(""),
            "integration callback returned provider error"
        );
        let kind = if error == "access_denied" {
            "denied"
        } else {
            "provider_error"
        };
        return redirect_with_error(&state, provider_str, kind);
    }

    let Some(code) = query.code.as_deref() else {
        tracing::warn!(
            provider = provider_str,
            "integration callback missing code parameter"
        );
        return redirect_with_error(&state, provider_str, "server");
    };
    let Some(callback_state) = query.state.as_deref() else {
        tracing::warn!(
            provider = provider_str,
            "integration callback missing state parameter"
        );
        return redirect_with_error(&state, provider_str, "server");
    };

    let Some(ops) = state.integration_ops.as_ref() else {
        tracing::error!(
            provider = provider_str,
            "integration ops not configured for callback"
        );
        return redirect_with_error(&state, provider_str, "server");
    };

    if let Err(err) = ops.callback(parsed_provider, code, callback_state).await {
        // Map AppError to a distinct redirect kind so the client can
        // distinguish CSRF/state failures from upstream-provider issues
        // from backend misconfig. Server-side errors log at error level
        // so alerting fires; client-side / provider-side issues stay at
        // warn.
        use ind_application::AppError;
        use ind_domain::DomainError;
        let (kind, is_server_err) = match &err {
            AppError::Domain(DomainError::Validation { field, .. }) if field == "state" => {
                ("state_invalid", false)
            }
            AppError::Domain(DomainError::Validation { field, .. }) if field == "provider" => {
                ("provider_mismatch", false)
            }
            AppError::Domain(DomainError::Validation { field, .. }) if field == "credentials" => {
                ("invalid_credentials", false)
            }
            AppError::Domain(DomainError::NotFound {
                entity: "integration_provider",
                ..
            }) => ("provider_not_configured", true),
            AppError::ExternalService { .. } => ("provider_error", false),
            AppError::Repository(_) => ("server", true),
            _ => ("server", true),
        };
        if is_server_err {
            tracing::error!(
                provider = provider_str,
                error = %err,
                kind = kind,
                "integration callback server-side failure"
            );
        } else {
            tracing::warn!(
                provider = provider_str,
                error = %err,
                kind = kind,
                "integration callback failed"
            );
        }
        return redirect_with_error(&state, provider_str, kind);
    }

    let frontend = state.config.frontend_url.trim_end_matches('/');
    let target = format!("{frontend}/preferences/integrations?connected={provider_str}");
    Redirect::temporary(&target)
}
fn parse_oauth_provider(raw: &str) -> Result<IntegrationOAuthProvider, ApiError> {
    match raw {
        "notion" => Ok(IntegrationOAuthProvider::Notion),
        _ => Err(ApiError::NotFound {
            entity: "integration_provider",
            id: raw.to_string(),
        }),
    }
}
