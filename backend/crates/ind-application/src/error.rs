use ind_domain::DomainError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error("authentication required")]
    Auth,

    #[error("access forbidden")]
    Forbidden,

    #[error("rate limited")]
    RateLimited,

    #[error("quota exceeded: {quota}")]
    QuotaExceeded { quota: &'static str },

    #[error("payment required: {feature}")]
    PaymentRequired { feature: &'static str },

    #[error("external service error from {service}: {message}")]
    ExternalService { service: String, message: String },

    /// The AI provider is unreachable or in a transient failure state (offline, restarting,
    /// or mid model-lifecycle). `message` is an internal diagnostic; HTTP mapping must never
    /// forward it to clients.
    #[error("provider unavailable: {message}")]
    ProviderUnavailable { message: String },

    #[error("repository error: {0}")]
    Repository(#[from] Box<dyn std::error::Error + Send + Sync>),
}
