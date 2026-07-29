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

#[tokio::test]
async fn insufficient_permissions_is_an_rfc_6750_problem_response() {
    let response = ApiError::InsufficientPermissions {
        required: vec![
            ind_domain::ApiPermission::LibraryRead,
            ind_domain::ApiPermission::AiUse,
        ],
    }
    .into_response();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        response.headers().get(http::header::CONTENT_TYPE).unwrap(),
        "application/problem+json"
    );
    assert_eq!(
        response
            .headers()
            .get(http::header::WWW_AUTHENTICATE)
            .unwrap(),
        "Bearer error=\"insufficient_scope\", scope=\"library:read ai:use\""
    );
    let body = to_bytes(response.into_body(), 1024).await.unwrap();
    let problem: ProblemDetail = serde_json::from_slice(&body).unwrap();
    assert_eq!(problem.status, StatusCode::FORBIDDEN.as_u16());
    assert_eq!(problem.code.as_deref(), Some("insufficient_permissions"));
    assert_eq!(
        problem.detail,
        "The personal access token lacks the required permission."
    );
    assert_eq!(
        problem.problem_type,
        "https://indelible.app/problems/insufficient-permissions"
    );
}
