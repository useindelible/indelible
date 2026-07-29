use axum::extract::rejection::JsonRejection;
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use ind_application::AppError;
use ind_application::ports::OAuthError;
use ind_domain::{ApiPermission, DomainError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProblemDetail {
    #[serde(rename = "type")]
    pub problem_type: String,
    pub title: String,
    pub status: u16,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<FieldError>>,
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("bad request: {message}")]
    BadRequest { message: String },

    #[error("validation error")]
    ValidationError { errors: Vec<FieldError> },

    #[error("unauthorized: {message}")]
    Unauthorized { message: String },

    #[error("forbidden: {message}")]
    Forbidden { message: String },

    #[error("insufficient permissions")]
    InsufficientPermissions { required: Vec<ApiPermission> },

    #[error("{entity} not found: {id}")]
    NotFound { entity: &'static str, id: String },

    #[error("{entity} conflict: {message}")]
    Conflict {
        entity: &'static str,
        message: String,
    },

    #[error("payload too large")]
    PayloadTooLarge,

    #[error("range not satisfiable (total={total})")]
    RangeNotSatisfiable { total: i64 },

    /// Upstream provider failure surfaced to the client. The user-facing
    /// detail is a generic placeholder so we never leak provider-specific
    /// error messages — the original cause is recorded in the trace logs.
    #[error("external service error from {service}")]
    ExternalService { service: String },

    #[error("rate limited")]
    RateLimited,

    #[error("quota exceeded: {quota}")]
    QuotaExceeded { quota: &'static str },

    #[error("payment required: {feature}")]
    PaymentRequired { feature: &'static str },

    #[error("service unavailable: {message}")]
    ServiceUnavailable { message: String },

    /// The user's AI provider is offline or mid-restart. Detail stays generic so provider
    /// diagnostics never reach clients; responses carry `Retry-After` so clients can back off.
    #[error("AI provider unavailable")]
    ProviderUnavailable,

    #[error("internal server error: {message}")]
    Internal { message: String },
}

impl ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest { .. } => StatusCode::BAD_REQUEST,
            Self::ValidationError { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Unauthorized { .. } => StatusCode::UNAUTHORIZED,
            Self::Forbidden { .. } => StatusCode::FORBIDDEN,
            Self::InsufficientPermissions { .. } => StatusCode::FORBIDDEN,
            Self::NotFound { .. } => StatusCode::NOT_FOUND,
            Self::Conflict { .. } => StatusCode::CONFLICT,
            Self::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Self::RangeNotSatisfiable { .. } => StatusCode::RANGE_NOT_SATISFIABLE,
            Self::ExternalService { .. } => StatusCode::BAD_GATEWAY,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            Self::QuotaExceeded { .. } => StatusCode::TOO_MANY_REQUESTS,
            Self::PaymentRequired { .. } => StatusCode::PAYMENT_REQUIRED,
            Self::ServiceUnavailable { .. } => StatusCode::SERVICE_UNAVAILABLE,
            Self::ProviderUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            Self::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn problem_code(&self) -> &str {
        match self {
            Self::BadRequest { .. } => "bad_request",
            Self::ValidationError { .. } => "validation_error",
            Self::Unauthorized { .. } => "auth_required",
            Self::Forbidden { .. } => "forbidden",
            Self::InsufficientPermissions { .. } => "insufficient_permissions",
            Self::NotFound { .. } => "not_found",
            Self::Conflict { .. } => "conflict",
            Self::PayloadTooLarge => "payload_too_large",
            Self::RangeNotSatisfiable { .. } => "range_not_satisfiable",
            Self::ExternalService { .. } => "external_service_error",
            Self::RateLimited => "rate_limited",
            Self::QuotaExceeded { .. } => "quota_exceeded",
            Self::PaymentRequired { .. } => "payment_required",
            Self::ServiceUnavailable { .. } => "service_unavailable",
            Self::ProviderUnavailable => "ai_provider_unavailable",
            Self::Internal { .. } => "internal_error",
        }
    }

    fn title(&self) -> &str {
        match self {
            Self::BadRequest { .. } => "Bad Request",
            Self::ValidationError { .. } => "Validation Error",
            Self::Unauthorized { .. } => "Unauthorized",
            Self::Forbidden { .. } => "Forbidden",
            Self::InsufficientPermissions { .. } => "Insufficient Permissions",
            Self::NotFound { .. } => "Not Found",
            Self::Conflict { .. } => "Conflict",
            Self::PayloadTooLarge => "Payload Too Large",
            Self::RangeNotSatisfiable { .. } => "Range Not Satisfiable",
            Self::ExternalService { .. } => "Bad Gateway",
            Self::RateLimited => "Rate Limited",
            Self::QuotaExceeded { .. } => "Quota Exceeded",
            Self::PaymentRequired { .. } => "Payment Required",
            Self::ServiceUnavailable { .. } => "Service Unavailable",
            Self::ProviderUnavailable => "AI Provider Unavailable",
            Self::Internal { .. } => "Internal Server Error",
        }
    }

    fn detail(&self) -> String {
        match self {
            Self::Internal { .. } => "An unexpected error occurred".to_string(),
            Self::InsufficientPermissions { .. } => {
                "The personal access token lacks the required permission.".to_string()
            }
            Self::ExternalService { .. } => {
                "Upstream provider failed; please try again later.".to_string()
            }
            Self::ProviderUnavailable => {
                "AI provider is unavailable. Start your provider and try again.".to_string()
            }
            other => other.to_string(),
        }
    }

    fn to_problem_detail(&self) -> ProblemDetail {
        let code = self.problem_code();
        ProblemDetail {
            problem_type: format!("https://indelible.app/problems/{}", code.replace('_', "-")),
            title: self.title().to_string(),
            status: self.status_code().as_u16(),
            detail: self.detail(),
            code: Some(code.to_string()),
            errors: match self {
                Self::ValidationError { errors } => Some(
                    errors
                        .iter()
                        .map(|e| FieldError {
                            field: e.field.clone(),
                            message: e.message.clone(),
                        })
                        .collect(),
                ),
                _ => None,
            },
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();

        if status.is_server_error() {
            tracing::error!(error = %self, "internal server error");
        } else {
            tracing::warn!(error = %self, status = status.as_u16(), "client error");
        }

        let problem = self.to_problem_detail();
        let body = serde_json::to_vec(&problem).unwrap_or_default();

        let mut response = (
            status,
            [(http::header::CONTENT_TYPE, "application/problem+json")],
            body,
        )
            .into_response();

        if matches!(self, Self::ProviderUnavailable) {
            response.headers_mut().insert(
                http::header::RETRY_AFTER,
                http::HeaderValue::from_static("60"),
            );
        }

        if let Self::InsufficientPermissions { required } = &self {
            let scope = required
                .iter()
                .map(|permission| permission.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            let challenge = format!("Bearer error=\"insufficient_scope\", scope=\"{scope}\"");
            if let Ok(value) = http::HeaderValue::from_str(&challenge) {
                response
                    .headers_mut()
                    .insert(http::header::WWW_AUTHENTICATE, value);
            }
        }

        response
    }
}

impl From<AppError> for ApiError {
    fn from(err: AppError) -> Self {
        match err {
            AppError::Domain(domain_err) => Self::from(domain_err),
            AppError::Auth => Self::Unauthorized {
                message: "authentication required".to_string(),
            },
            AppError::Forbidden => Self::Forbidden {
                message: "access forbidden".to_string(),
            },
            AppError::RateLimited => Self::RateLimited,
            AppError::QuotaExceeded { quota } => Self::QuotaExceeded { quota },
            AppError::PaymentRequired { feature } => Self::PaymentRequired { feature },
            AppError::ExternalService { message, .. } if message == "payload too large" => {
                Self::PayloadTooLarge
            }
            AppError::ExternalService { service, message } => {
                tracing::error!(service = %service, error = %message, "external service failure");
                Self::ExternalService { service }
            }
            AppError::ProviderUnavailable { message } => {
                tracing::warn!(error = %message, "ai provider unavailable");
                Self::ProviderUnavailable
            }
            AppError::Repository(err) => {
                tracing::error!(error = %err, "repository error");
                Self::Internal {
                    message: err.to_string(),
                }
            }
        }
    }
}

impl From<DomainError> for ApiError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::Validation { field, message } => Self::ValidationError {
                errors: vec![FieldError { field, message }],
            },
            DomainError::NotFound { entity, id } => Self::NotFound { entity, id },
            DomainError::Conflict { entity, message } => Self::Conflict { entity, message },
            DomainError::InvariantViolation { message } => {
                tracing::error!(error = %message, "invariant violation");
                Self::Internal { message }
            }
            DomainError::InvalidState {
                entity,
                current,
                expected,
            } => {
                tracing::error!(
                    entity = entity,
                    current = %current,
                    expected = %expected,
                    "invalid state"
                );
                Self::Internal {
                    message: format!("{entity} is in invalid state"),
                }
            }
            DomainError::Entitlement { feature } => Self::PaymentRequired { feature },
            DomainError::QuoteInvalid => Self::BadRequest {
                message: "quote is invalid, expired, or does not match the request".to_string(),
            },
        }
    }
}

impl From<ind_auth::AuthError> for ApiError {
    fn from(err: ind_auth::AuthError) -> Self {
        use ind_auth::AuthError;
        match err {
            AuthError::InvalidCredentials | AuthError::AccountNotFound => Self::Unauthorized {
                message: "invalid credentials".to_string(),
            },
            AuthError::AccountDisabled => Self::Forbidden {
                message: "account disabled".to_string(),
            },
            AuthError::EmailNotVerified => Self::Forbidden {
                message: "email not verified".to_string(),
            },
            AuthError::SessionExpired | AuthError::SessionNotFound => Self::Unauthorized {
                message: "session invalid".to_string(),
            },
            AuthError::TokenExpired => Self::BadRequest {
                message: "token expired".to_string(),
            },
            AuthError::TokenInvalid => Self::BadRequest {
                message: "token invalid".to_string(),
            },
            AuthError::TokenRevoked => Self::BadRequest {
                message: "token revoked".to_string(),
            },
            AuthError::TokenAlreadyUsed => Self::BadRequest {
                message: "token already used".to_string(),
            },
            AuthError::HashError(msg) => {
                tracing::error!(error = %msg, "password hash error");
                Self::Internal {
                    message: "internal error".to_string(),
                }
            }
            AuthError::PasswordTooWeak => Self::ValidationError {
                errors: vec![FieldError {
                    field: "password".to_string(),
                    message: "password does not meet strength requirements".to_string(),
                }],
            },
            AuthError::EmailAlreadyExists => Self::Conflict {
                entity: "user",
                message: "email already exists".to_string(),
            },
            AuthError::RateLimited => Self::RateLimited,
            AuthError::ValidationError { field, message } => Self::ValidationError {
                errors: vec![FieldError { field, message }],
            },
            AuthError::ConfirmationRequired => Self::BadRequest {
                message: "confirmation required".to_string(),
            },
            AuthError::MailTransportUnavailable => Self::ServiceUnavailable {
                message: "changing your email address needs an outbound mail transport, \
                          which this server has not configured yet"
                    .to_string(),
            },
            AuthError::SignupsDisabled => Self::Forbidden {
                message: "signups are disabled".to_string(),
            },
            AuthError::Repo(app_err) => Self::from(app_err),
        }
    }
}

impl From<OAuthError> for ApiError {
    fn from(err: OAuthError) -> Self {
        match err {
            OAuthError::Configuration(message) => Self::Internal {
                message: format!("OAuth configuration error: {message}"),
            },
            OAuthError::Exchange(message) => Self::BadRequest { message },
            OAuthError::ProviderNotConfigured(provider) => Self::BadRequest {
                message: format!("OAuth provider {provider:?} is not configured"),
            },
            OAuthError::InvalidState => Self::BadRequest {
                message: "OAuth state mismatch".to_string(),
            },
            OAuthError::IdentityAlreadyLinked => Self::Conflict {
                entity: "oauth_identity",
                message: "identity already linked".to_string(),
            },
            OAuthError::CannotUnlinkOnly => Self::BadRequest {
                message: "cannot unlink the only authentication method".to_string(),
            },
            OAuthError::IdentityNotFound => Self::NotFound {
                entity: "oauth_identity",
                id: String::new(),
            },
            OAuthError::UserDeactivated => Self::Forbidden {
                message: "account disabled".to_string(),
            },
            OAuthError::App(app_err) => Self::from(app_err),
        }
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        Self::BadRequest {
            message: err.to_string(),
        }
    }
}

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        Self::BadRequest {
            message: rejection.body_text(),
        }
    }
}

#[cfg(test)]
mod tests;
