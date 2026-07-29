use secrecy::SecretString;

use ind_application::AppError;
use ind_auth::{CipherError, CredentialCipher};
use ind_domain::{DomainError, MilaConfig};

use crate::{AiError, AiProviderConfig};

pub fn chat_provider_from_config(
    config: &MilaConfig,
    cipher: Option<&CredentialCipher>,
) -> Result<AiProviderConfig, AppError> {
    let api_key = open_api_key(
        config.chat_api_key_enc.as_deref(),
        config.chat_cipher_version,
        cipher,
    )?
    .map(SecretString::from);

    Ok(AiProviderConfig::new(config.chat_api_base.clone(), api_key))
}

pub fn embedding_provider_from_config(
    config: &MilaConfig,
    cipher: Option<&CredentialCipher>,
) -> Result<AiProviderConfig, AppError> {
    let api_key = open_api_key(
        config.embedding_api_key_enc.as_deref(),
        config.embedding_cipher_version,
        cipher,
    )?
    .map(SecretString::from);

    Ok(AiProviderConfig::new(
        config.embedding_api_base.clone(),
        api_key,
    ))
}

fn open_api_key(
    bytes: Option<&[u8]>,
    key_cipher_version: i16,
    cipher: Option<&CredentialCipher>,
) -> Result<Option<String>, AppError> {
    let Some(bytes) = bytes.filter(|bytes| !bytes.is_empty()) else {
        return Ok(None);
    };

    if key_cipher_version == 0 {
        return plaintext_key(bytes);
    }

    let cipher = cipher.ok_or_else(|| AppError::ExternalService {
        service: "credential_cipher".into(),
        message: "AUTH_CREDENTIAL_KEY is required to open stored Mila API keys".into(),
    })?;

    match cipher.open(bytes) {
        Ok(opened) => plaintext_key(&opened),
        Err(error) => Err(cipher_error(error)),
    }
}

fn plaintext_key(bytes: &[u8]) -> Result<Option<String>, AppError> {
    Ok(Some(String::from_utf8(bytes.to_vec()).map_err(|err| {
        AppError::Domain(DomainError::InvariantViolation {
            message: format!("mila api key is not valid utf-8: {err}"),
        })
    })?))
}

fn cipher_error(error: CipherError) -> AppError {
    AppError::ExternalService {
        service: "credential_cipher".into(),
        message: error.to_string(),
    }
}

pub(crate) fn map_ai_error(error: AiError) -> AppError {
    match error {
        AiError::AuthenticationFailed { .. } => AppError::Auth,
        AiError::ProviderError {
            status_code: 429, ..
        } => AppError::RateLimited,
        AiError::EndpointDisallowed { message } => AppError::Domain(DomainError::Validation {
            field: "provider_api_base".into(),
            message,
        }),
        AiError::ProviderUnreachable { message } => AppError::ProviderUnavailable { message },
        AiError::ProviderError {
            status_code,
            message,
        } if is_transient_provider_status(status_code)
            || is_transient_lifecycle_signal(&message) =>
        {
            AppError::ProviderUnavailable { message }
        }
        AiError::ProviderError { message, .. }
        | AiError::MalformedResponse { message }
        | AiError::StreamTerminatedUnexpectedly { message } => AppError::ExternalService {
            service: "mila-provider".into(),
            message,
        },
    }
}

/// Statuses a healthy provider deployment can emit while temporarily unable to serve
/// (gateway hiccups, restarts, overload) — retrying unmodified is expected to succeed.
fn is_transient_provider_status(status_code: u16) -> bool {
    matches!(status_code, 408 | 425 | 500 | 502 | 503 | 504)
}

/// Model-lifecycle and temporary-runtime phrases local providers (LM Studio) emit while a
/// model is loading, unloading, being swapped, or mid-restart; complete conservative
/// signal set ported from the abandoned circuit-breaker classifier (eba36ec3, ind-ai
/// provider_recovery/classification.rs).
fn is_transient_lifecycle_signal(message: &str) -> bool {
    let message = message.to_ascii_lowercase();
    message.contains("model is unloaded")
        || message.contains("model unloaded")
        || message.contains("loading aborted")
        || message.contains("loading cancelled")
        || message.contains("terminated")
        || message.contains("prediction timeout")
        || message.contains("stream timeout")
        || message.contains("temporary lifecycle")
        || message.contains("temporary internal")
}

pub(crate) fn extract_text_window(
    text: &str,
    highlight_text: &str,
    highlight_offset: Option<usize>,
    window_chars: usize,
) -> String {
    if text.is_empty() {
        return String::new();
    }

    let text_chars: Vec<char> = text.chars().collect();
    let total = text_chars.len();
    let highlight_start = highlight_offset
        .filter(|offset| *offset <= total)
        .or_else(|| {
            text.find(highlight_text)
                .map(|idx| text[..idx].chars().count())
        })
        .unwrap_or(0);

    let highlight_len = highlight_text.chars().count().max(1);
    let start = highlight_start.saturating_sub(window_chars);
    let end = total.min(highlight_start + highlight_len + window_chars);

    text_chars[start..end].iter().collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_ai_error_routes_transient_provider_failures_to_provider_unavailable() {
        let transient_cases = [
            AiError::ProviderUnreachable {
                message: "error sending request: connection refused".into(),
            },
            AiError::ProviderError {
                status_code: 408,
                message: "Request Timeout".into(),
            },
            AiError::ProviderError {
                status_code: 425,
                message: "Too Early".into(),
            },
            AiError::ProviderError {
                status_code: 500,
                message: "Internal Server Error".into(),
            },
            AiError::ProviderError {
                status_code: 502,
                message: "Bad Gateway".into(),
            },
            AiError::ProviderError {
                status_code: 503,
                message: "Service Unavailable".into(),
            },
            AiError::ProviderError {
                status_code: 504,
                message: "Gateway Timeout".into(),
            },
            AiError::ProviderError {
                status_code: 400,
                message: "Model is unloaded".into(),
            },
            AiError::ProviderError {
                status_code: 404,
                message: "loading aborted for model qwen3".into(),
            },
            AiError::ProviderError {
                status_code: 400,
                message: "prediction was terminated by the runtime".into(),
            },
            AiError::ProviderError {
                status_code: 404,
                message: "Prediction timeout after 30s".into(),
            },
            AiError::ProviderError {
                status_code: 400,
                message: "stream timeout waiting for first token".into(),
            },
            AiError::ProviderError {
                status_code: 400,
                message: "temporary lifecycle transition in progress".into(),
            },
            AiError::ProviderError {
                status_code: 400,
                message: "temporary internal error, please retry".into(),
            },
        ];

        for case in transient_cases {
            let label = case.to_string();
            assert!(
                matches!(map_ai_error(case), AppError::ProviderUnavailable { .. }),
                "expected ProviderUnavailable for: {label}"
            );
        }
    }

    #[test]
    fn map_ai_error_keeps_non_transient_mappings_unchanged() {
        assert!(matches!(
            map_ai_error(AiError::AuthenticationFailed {
                message: "invalid api key".into(),
            }),
            AppError::Auth
        ));
        assert!(matches!(
            map_ai_error(AiError::ProviderError {
                status_code: 429,
                message: "rate limit exceeded".into(),
            }),
            AppError::RateLimited
        ));
        assert!(matches!(
            map_ai_error(AiError::EndpointDisallowed {
                message: "loopback only".into(),
            }),
            AppError::Domain(DomainError::Validation { .. })
        ));
        assert!(matches!(
            map_ai_error(AiError::ProviderError {
                status_code: 404,
                message: "model not found".into(),
            }),
            AppError::ExternalService { .. }
        ));
        assert!(matches!(
            map_ai_error(AiError::ProviderError {
                status_code: 400,
                message: "invalid request payload".into(),
            }),
            AppError::ExternalService { .. }
        ));
        assert!(matches!(
            map_ai_error(AiError::MalformedResponse {
                message: "unexpected eof".into(),
            }),
            AppError::ExternalService { .. }
        ));
        assert!(matches!(
            map_ai_error(AiError::StreamTerminatedUnexpectedly {
                message: "body stream ended".into(),
            }),
            AppError::ExternalService { .. }
        ));
    }
}
