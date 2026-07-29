/// Headers every server-sent-events response must carry.
///
/// `no-cache` plus `x-accel-buffering` defeat proxy buffering, which otherwise
/// makes nginx batch or hang a stream; `no-store` keeps authenticated streams
/// out of shared caches (the global no-store layer skips responses that set
/// their own header).
pub(crate) fn stream_headers() -> [(http::HeaderName, &'static str); 2] {
    [
        (http::header::CACHE_CONTROL, "private, no-cache, no-store"),
        (http::HeaderName::from_static("x-accel-buffering"), "no"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_headers_defeat_proxy_buffering_and_shared_caches() {
        let headers = stream_headers();

        assert_eq!(headers[0].0, http::header::CACHE_CONTROL);
        assert_eq!(headers[0].1, "private, no-cache, no-store");
        assert_eq!(headers[1].0, "x-accel-buffering");
        assert_eq!(headers[1].1, "no");
    }
}
