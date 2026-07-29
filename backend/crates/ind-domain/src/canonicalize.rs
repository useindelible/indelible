use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct CanonicalUrl(String);

impl CanonicalUrl {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl std::fmt::Display for CanonicalUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone)]
pub struct CanonicalizationConfig {
    pub tracking_params: Vec<String>,
    pub max_url_length: usize,
}

impl Default for CanonicalizationConfig {
    fn default() -> Self {
        Self {
            tracking_params: [
                "utm_source",
                "utm_medium",
                "utm_campaign",
                "utm_content",
                "utm_term",
                "fbclid",
                "gclid",
                "ref",
                "source",
                "mc_cid",
                "mc_eid",
                // Transient challenge/UI params; these do not identify different content.
                "captcha",
                "__readwiseLocation",
                // YouTube junk params — strip for dedup while preserving `v` (video ID)
                // and `t` (start time, intentionally kept).
                "list",
                "si",
                "pp",
                "ab_channel",
                "feature",
                "index",
                "start_radio",
            ]
            .iter()
            .map(|s| String::from(*s))
            .collect(),
            max_url_length: 2048,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CanonicalizeError {
    #[error("invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
    #[error("URL exceeds maximum length of {max} characters")]
    TooLong { max: usize },
}

pub fn canonicalize_url(
    raw: &str,
    config: &CanonicalizationConfig,
) -> Result<CanonicalUrl, CanonicalizeError> {
    if raw.len() > config.max_url_length {
        return Err(CanonicalizeError::TooLong {
            max: config.max_url_length,
        });
    }

    let mut url = Url::parse(raw)?;

    url.set_fragment(None);

    if url.scheme() == "http" {
        // url::Url::set_scheme returns Err(()) for non-special URLs (e.g. IP addresses);
        // we accept the no-op: the URL stays http:// rather than failing canonicalization.
        let _ = url.set_scheme("https");
    }

    if let Some(host) = url.host_str().map(|h| h.to_lowercase()) {
        let normalized_host = host.strip_prefix("www.").unwrap_or(&host);
        let _ = url.set_host(Some(normalized_host));
    }

    let filtered_params: Vec<(String, String)> = url
        .query_pairs()
        .filter(|(key, _)| {
            let key_lower = key.to_lowercase();
            !config
                .tracking_params
                .iter()
                .any(|tp| tp.to_lowercase() == key_lower)
        })
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();

    if filtered_params.is_empty() {
        url.set_query(None);
    } else {
        let mut sorted_params = filtered_params;
        sorted_params.sort_by(|a, b| a.0.cmp(&b.0));
        let query_string: String =
            sorted_params
                .iter()
                .enumerate()
                .fold(String::new(), |mut acc, (i, (k, v))| {
                    if i > 0 {
                        acc.push('&');
                    }
                    acc.push_str(
                        &url::form_urlencoded::Serializer::new(String::new())
                            .append_pair(k, v)
                            .finish(),
                    );
                    acc
                });
        url.set_query(Some(&query_string));
    }

    let path = url.path().to_string();
    if path.len() > 1 && path.ends_with('/') {
        url.set_path(&path[..path.len() - 1]);
    }

    Ok(CanonicalUrl(url.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalization_table_covers_dedup_boundaries() {
        let config = CanonicalizationConfig::default();
        for (raw, expected) in [
            (
                "http://WWW.Example.com/path/?utm_source=x&b=2&a=1#part",
                "https://example.com/path?a=1&b=2",
            ),
            (
                "https://youtube.com/watch?v=abc&list=x&si=y&t=4",
                "https://youtube.com/watch?t=4&v=abc",
            ),
            ("https://www2.example.com/", "https://www2.example.com/"),
            (
                "https://example.com/%E2%9C%93",
                "https://example.com/%E2%9C%93",
            ),
        ] {
            assert_eq!(
                canonicalize_url(raw, &config).unwrap().as_str(),
                expected,
                "{raw}"
            );
        }
        assert!(canonicalize_url("not a url", &config).is_err());
    }
}
