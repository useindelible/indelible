use axum::body::to_bytes;
use axum::response::IntoResponse;
use http::StatusCode;

use super::*;

#[tokio::test]
async fn provider_unavailable_maps_to_503_with_stable_code_and_retry_after() {
    let api_error = ApiError::from(ind_application::AppError::ProviderUnavailable {
        message: "connect error: Connection refused (os error 61)".into(),
    });
    let response = api_error.into_response();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        response
            .headers()
            .get(http::header::RETRY_AFTER)
            .expect("Retry-After header"),
        "60"
    );
    let body = to_bytes(response.into_body(), 1024).await.unwrap();
    let body = String::from_utf8(body.to_vec()).unwrap();
    assert!(body.contains("ai_provider_unavailable"));
    assert!(body.contains("AI provider is unavailable"));
    assert!(!body.contains("os error 61"));
}

#[tokio::test]
async fn internal_errors_keep_details_out_of_the_public_response() {
    let response = ApiError::Internal {
        message: "database password leaked".into(),
    }
    .into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = to_bytes(response.into_body(), 1024).await.unwrap();
    let body = String::from_utf8(body.to_vec()).unwrap();
    assert!(body.contains("internal_error"));
    assert!(!body.contains("database password leaked"));
}
