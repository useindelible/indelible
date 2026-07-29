use ind_egress::{EgressPolicy, GuardedClientOptions, GuardedHttpClient, UrlRules};
use reqwest::header::{ACCEPT, ACCEPT_LANGUAGE, CACHE_CONTROL, HeaderMap, HeaderValue, USER_AGENT};

#[expect(
    clippy::expect_used,
    reason = "guarded client builds from valid static options and headers; construction is infallible"
)]
pub(super) fn build_feed_http_client(policy: EgressPolicy) -> GuardedHttpClient {
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) \
             AppleWebKit/537.36 (KHTML, like Gecko) \
             Chrome/131.0.0.0 Safari/537.36",
        ),
    );
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"),
    );
    headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));
    headers.insert(CACHE_CONTROL, HeaderValue::from_static("no-cache"));

    ind_egress::build_guarded_client(
        GuardedClientOptions::new(UrlRules::ingest(), policy).default_headers(headers),
    )
    .expect("feed guarded client builds")
}
