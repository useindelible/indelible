use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AiError {
    #[error("provider unreachable: {message}")]
    ProviderUnreachable { message: String },
    #[error("authentication failed: {message}")]
    AuthenticationFailed { message: String },
    #[error("provider returned status {status_code}: {message}")]
    ProviderError { status_code: u16, message: String },
    #[error("malformed provider response: {message}")]
    MalformedResponse { message: String },
    #[error("stream terminated unexpectedly: {message}")]
    StreamTerminatedUnexpectedly { message: String },
    #[error("provider endpoint is not allowed: {message}")]
    EndpointDisallowed { message: String },
}

impl AiError {
    pub(crate) fn from_transport(err: reqwest::Error) -> Self {
        Self::ProviderUnreachable {
            message: err.to_string(),
        }
    }

    pub(crate) fn from_decode(err: impl std::fmt::Display) -> Self {
        Self::MalformedResponse {
            message: err.to_string(),
        }
    }

    pub(crate) fn from_stream(err: impl std::fmt::Display) -> Self {
        Self::StreamTerminatedUnexpectedly {
            message: err.to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ProviderErrorEnvelope {
    error: Option<ProviderErrorBody>,
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ProviderErrorBody {
    Message { message: Option<String> },
    Text(String),
}

impl ProviderErrorBody {
    fn into_message(self) -> Option<String> {
        match self {
            Self::Message { message } => message,
            Self::Text(message) => Some(message),
        }
    }
}

pub(crate) async fn map_error_response(response: reqwest::Response) -> AiError {
    let status = response.status();
    let status_code = status.as_u16();
    let body = response.text().await.unwrap_or_default();
    let message = parse_provider_error_message(&body).unwrap_or_else(|| {
        status
            .canonical_reason()
            .unwrap_or("provider error")
            .to_string()
    });

    if matches!(status_code, 401 | 403) {
        AiError::AuthenticationFailed { message }
    } else {
        AiError::ProviderError {
            status_code,
            message,
        }
    }
}

pub(crate) fn parse_provider_error_message(body: &str) -> Option<String> {
    if body.trim().is_empty() {
        return None;
    }

    serde_json::from_str::<ProviderErrorEnvelope>(body)
        .ok()
        .and_then(|envelope| {
            envelope
                .error
                .and_then(ProviderErrorBody::into_message)
                .or(envelope.message)
        })
        .or_else(|| Some(body.trim().to_string()))
}
