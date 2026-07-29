// RFC 2369 List-Unsubscribe + RFC 8058 List-Unsubscribe-Post parser.
//
// The List-Unsubscribe header may contain one or more URI references separated
// by commas, each wrapped in angle brackets. Common shapes:
//   `<mailto:u@example.com>`
//   `<https://example.com/u>`
//   `<https://example.com/u>, <mailto:u@example.com>`
// When a sender supports one-click unsubscribe, they also send:
//   `List-Unsubscribe-Post: List-Unsubscribe=One-Click`
// We promote the https target to `one_click_post_url` only when that header is
// present; otherwise the https target stays in `web_url` (cannot be auto-acted
// upon without a browser).

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UnsubscribeTargets {
    pub one_click_post_url: Option<String>,
    pub mailto_addr: Option<String>,
    pub web_url: Option<String>,
}

impl UnsubscribeTargets {
    pub fn is_empty(&self) -> bool {
        self.one_click_post_url.is_none() && self.mailto_addr.is_none() && self.web_url.is_none()
    }
}

pub fn parse_unsubscribe_targets(
    list_unsubscribe: Option<&str>,
    list_unsubscribe_post: Option<&str>,
) -> UnsubscribeTargets {
    let Some(raw) = list_unsubscribe.map(str::trim).filter(|s| !s.is_empty()) else {
        return UnsubscribeTargets::default();
    };

    let one_click = list_unsubscribe_post
        .map(|v| v.eq_ignore_ascii_case("List-Unsubscribe=One-Click"))
        .unwrap_or(false);

    let mut mailto_addr: Option<String> = None;
    let mut https_url: Option<String> = None;

    for part in raw.split(',') {
        let token = part.trim().trim_start_matches('<').trim_end_matches('>');
        let token = token.trim();
        if token.is_empty() {
            continue;
        }
        let lower = token.to_ascii_lowercase();
        if let Some(rest) = lower.strip_prefix("mailto:") {
            if mailto_addr.is_none() {
                let addr_end = rest.find('?').unwrap_or(rest.len());
                let addr = &token[7..7 + addr_end];
                mailto_addr = Some(addr.trim().to_string());
            }
        } else if (lower.starts_with("https://") || lower.starts_with("http://"))
            && https_url.is_none()
        {
            https_url = Some(token.to_string());
        }
    }

    let (one_click_post_url, web_url) = match (one_click, https_url) {
        (true, Some(url)) => (Some(url), None),
        (false, Some(url)) => (None, Some(url)),
        (_, None) => (None, None),
    };

    UnsubscribeTargets {
        one_click_post_url,
        mailto_addr,
        web_url,
    }
}
