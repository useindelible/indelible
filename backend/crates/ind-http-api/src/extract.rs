use axum::extract::FromRequest;
use axum::response::IntoResponse;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::error::{ApiError, FieldError};
use crate::validation::validation_errors_to_field_errors;

pub struct Json<T>(pub T);

impl<S, T> FromRequest<S> for Json<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = ApiError;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(req, state).await {
            Ok(axum::Json(value)) => Ok(Json(value)),
            Err(rejection) => Err(ApiError::from(rejection)),
        }
    }
}

impl<T: Serialize> IntoResponse for Json<T> {
    fn into_response(self) -> axum::response::Response {
        axum::Json(self.0).into_response()
    }
}

pub trait Validate {
    fn validate(&self) -> Result<(), Vec<FieldError>>;
}

impl<T> Validate for T
where
    T: validator::Validate,
{
    fn validate(&self) -> Result<(), Vec<FieldError>> {
        validator::Validate::validate(self)
            .map_err(|errors| validation_errors_to_field_errors(&errors))
    }
}

pub struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = ApiError;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        match value.validate() {
            Ok(()) => Ok(ValidatedJson(value)),
            Err(errors) => Err(ApiError::ValidationError { errors }),
        }
    }
}
