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

/// Collect one multipart field into memory, tracking a running total across
/// fields and rejecting with 413 once `limit` is exceeded.
pub(crate) async fn read_multipart_field_bytes(
    mut field: axum::extract::multipart::Field<'_>,
    total_bytes: &mut usize,
    limit: usize,
) -> Result<bytes::Bytes, ApiError> {
    let mut buf = bytes::BytesMut::new();
    while let Some(chunk) = field.chunk().await.map_err(|err| ApiError::BadRequest {
        message: format!("error reading upload: {err}"),
    })? {
        *total_bytes += chunk.len();
        if *total_bytes > limit {
            return Err(ApiError::PayloadTooLarge);
        }
        buf.extend_from_slice(&chunk);
    }
    Ok(buf.freeze())
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
