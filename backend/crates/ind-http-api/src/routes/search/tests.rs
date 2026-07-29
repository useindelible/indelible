use chrono::{TimeZone, Utc};
use http::header::RETRY_AFTER;
use ind_domain::SearchRateLimitStatus;

use super::*;

#[test]
fn query_normalization_and_limits_enforce_transport_bounds() {
    assert!(normalize_query("   ").is_err());
    assert_eq!(normalize_query("  rust  ").unwrap(), "rust");
    assert_eq!(clamp_limit(None, 20, 50), 20);
    assert_eq!(clamp_limit(Some(0), 20, 50), 1);
    assert_eq!(clamp_limit(Some(500), 20, 50), 50);
}

#[test]
fn search_rate_limit_headers_include_retry_contract() {
    let status = SearchRateLimitStatus {
        allowed: false,
        quota_name: "search".into(),
        limit: 10,
        remaining: 0,
        reset_at: Utc.timestamp_opt(1_700_000_000, 0).unwrap(),
        retry_after_secs: Some(17),
    };
    let mut headers = HeaderMap::new();
    apply_rate_limit_headers(&mut headers, &status).unwrap();

    assert_eq!(headers["x-ratelimit-limit"], "10");
    assert_eq!(headers["x-ratelimit-remaining"], "0");
    assert_eq!(headers[RETRY_AFTER], "17");
}
