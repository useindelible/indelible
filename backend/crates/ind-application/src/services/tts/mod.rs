pub mod cache_key;
pub mod credentials;
pub mod elements;
pub mod entitlements;
pub mod persona;
pub mod session;
pub mod synthesis;

pub use credentials::{
    DefaultTtsCredentialResolver, DeploymentTtsCredential, ResolvedTtsCredentials,
    TtsProviderCredentialResolver,
};
pub use elements::{ReadableHtmlTtsPlanner, ReadableTtsElementSource};
pub use entitlements::{Deployment, TtsEntitlements};
pub use persona::{CreatePersonaInput, PersonaAdapterResolver, PersonaService};
pub use session::{StartSessionInput, TtsResolvedChunk, TtsSessionManifest, TtsSessionService};

use ind_domain::DomainError;

use crate::AppError;
use crate::ports::TtsAdapterError;

/// Central mapping from the adapter-layer error enum to the application-layer
/// `AppError`.
pub(crate) fn adapter_error(err: TtsAdapterError) -> AppError {
    match err {
        TtsAdapterError::AuthenticationFailed(message)
        | TtsAdapterError::Unsupported(message)
        | TtsAdapterError::InvalidRequest(message) => AppError::Domain(DomainError::Validation {
            field: "tts_adapter".into(),
            message,
        }),
        TtsAdapterError::ProviderError {
            status_code,
            message,
        } => AppError::ExternalService {
            service: "tts".into(),
            message: format!("status {status_code}: {message}"),
        },
        TtsAdapterError::ProviderUnreachable(message)
        | TtsAdapterError::MalformedResponse(message) => AppError::ExternalService {
            service: "tts".into(),
            message,
        },
        // The reader UI treats the following three as recoverable external
        // failures rather than user-input validation problems. Surfacing them
        // through `ExternalService` preserves the provider diagnostic while
        // keeping the HTTP status distinct from 400-class validation errors.
        TtsAdapterError::RateLimited { retry_after_ms } => AppError::ExternalService {
            service: "tts".into(),
            message: match retry_after_ms {
                Some(ms) => format!("rate limited (retry_after_ms={ms})"),
                None => "rate limited".into(),
            },
        },
        TtsAdapterError::QuotaExhausted => AppError::ExternalService {
            service: "tts".into(),
            message: "provider quota exhausted".into(),
        },
        TtsAdapterError::Timeout => AppError::ExternalService {
            service: "tts".into(),
            message: "provider request timed out".into(),
        },
    }
}

mod operations;
pub use operations::{TtsOperationsDeps, TtsOperationsService};
