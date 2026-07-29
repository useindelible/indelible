pub(super) fn strip_html(html: &str) -> String {
    ammonia::Builder::new()
        .tags(std::collections::HashSet::new())
        .clean(html)
        .to_string()
}

pub(super) fn extract_domain(url: &str) -> Option<String> {
    url::Url::parse(url)
        .ok()
        .and_then(|parsed| parsed.host_str().map(|host| host.to_string()))
}
