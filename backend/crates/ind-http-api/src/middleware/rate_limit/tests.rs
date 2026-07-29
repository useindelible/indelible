use axum::body::to_bytes;
use http::StatusCode;

use super::*;

async fn is_limited(limiter: &EndpointLimiter<String>, key: &str) -> bool {
    limiter
        .evaluate(key.into())
        .await
        .retry_after_secs
        .is_some()
}

#[tokio::test]
async fn limiter_isolates_keys_resets_and_fails_closed_at_capacity() {
    let limiter = EndpointLimiter::<String>::new(RateLimitRule::new(1, Duration::from_millis(10)));
    assert!(!is_limited(&limiter, "a").await);
    assert!(is_limited(&limiter, "a").await);
    assert!(!is_limited(&limiter, "b").await);
    tokio::time::sleep(Duration::from_millis(15)).await;
    assert!(!is_limited(&limiter, "a").await);

    let capped = EndpointLimiter::<String>::with_max_buckets(
        RateLimitRule::new(5, Duration::from_secs(60)),
        1,
    );
    assert!(!is_limited(&capped, "known").await);
    assert!(is_limited(&capped, "unknown").await);
}

#[tokio::test]
async fn rejection_preserves_public_status_body_and_headers() {
    let response = rate_limit_response(&RateLimitDecision {
        limit: 5,
        remaining: 0,
        reset_epoch_secs: 1_700_000_000,
        retry_after_secs: Some(17),
    });
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(response.headers()[http::header::RETRY_AFTER], "17");
    let body = to_bytes(response.into_body(), 1024).await.unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&body).unwrap()["error"]["code"],
        "rate_limited"
    );
}
