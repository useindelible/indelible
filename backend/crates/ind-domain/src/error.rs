#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("validation error on field `{field}`: {message}")]
    Validation { field: String, message: String },

    #[error("{entity} not found: {id}")]
    NotFound { entity: &'static str, id: String },

    #[error("{entity} conflict: {message}")]
    Conflict {
        entity: &'static str,
        message: String,
    },

    #[error("invariant violation: {message}")]
    InvariantViolation { message: String },

    #[error("{entity} is in invalid state: current=`{current}`, expected=`{expected}`")]
    InvalidState {
        entity: &'static str,
        current: String,
        expected: String,
    },

    #[error("entitlement `{feature}` is not granted to this account")]
    Entitlement { feature: &'static str },

    #[error("quote is invalid, expired, or does not match the request")]
    QuoteInvalid,
}
