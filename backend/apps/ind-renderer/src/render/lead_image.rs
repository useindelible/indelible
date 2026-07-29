//! Lead-image extraction from a rendered page's HTML.
//!
//! Mirrors the browser extension's `extractCoverUrl()`: `og:image` -> `twitter:image` -> the first
//! substantial `<img>` in the article body. The in-page Defuddle path applies the
//! same precedence against the live DOM; this Rust path handles saved/monolith HTML and is the
//! unit-tested reference for the precedence rules.

use scraper::{Html, Selector};
use url::Url;

const ARTICLE_SCOPES: &[&str] = &[
    "article",
    "[role=\"main\"]",
    "main",
    ".post-content",
    ".entry-content",
    ".article-body",
    "body",
];

pub(crate) fn lead_image_from_html(html: &str, base_url: Option<&str>) -> Option<String> {
    let document = Html::parse_document(html);
    meta_image(&document)
        .or_else(|| first_article_image(&document))
        .and_then(|raw| normalize(&raw, base_url))
}

fn meta_image(document: &Html) -> Option<String> {
    const META_SELECTORS: &[&str] = &[
        "meta[property=\"og:image\"]",
        "meta[name=\"twitter:image\"]",
        "meta[name=\"twitter:image:src\"]",
    ];
    for raw in META_SELECTORS {
        let selector = Selector::parse(raw).ok()?;
        if let Some(content) = document
            .select(&selector)
            .filter_map(|el| el.value().attr("content"))
            .map(str::trim)
            .find(|c| !c.is_empty())
        {
            return Some(content.to_string());
        }
    }
    None
}

fn first_article_image(document: &Html) -> Option<String> {
    for scope in ARTICLE_SCOPES {
        let Ok(scope_selector) = Selector::parse(scope) else {
            continue;
        };
        let Ok(img_selector) = Selector::parse("img") else {
            continue;
        };
        if let Some(root) = document.select(&scope_selector).next() {
            for img in root.select(&img_selector) {
                let value = img.value();
                let Some(src) = value.attr("src").map(str::trim).filter(|s| !s.is_empty()) else {
                    continue;
                };
                if is_too_small(value) {
                    continue;
                }
                return Some(src.to_string());
            }
        }
    }
    None
}

/// Skip images whose declared width or height is below 100px. Non-numeric dimensions are unknown
/// and not skipped.
fn is_too_small(element: &scraper::node::Element) -> bool {
    let small = |attr: &str| {
        element
            .attr(attr)
            .and_then(|raw| raw.parse::<u32>().ok())
            .is_some_and(|dim| dim < 100)
    };
    small("width") || small("height")
}

/// Resolve relative URLs against `base_url`; accept only `http(s)`; reject `data:`/empty.
fn normalize(raw: &str, base_url: Option<&str>) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() || raw.starts_with("data:") {
        return None;
    }
    if let Ok(parsed) = Url::parse(raw) {
        return http_or_none(parsed);
    }
    let base = base_url.and_then(|b| Url::parse(b).ok())?;
    base.join(raw).ok().and_then(http_or_none)
}

fn http_or_none(url: Url) -> Option<String> {
    matches!(url.scheme(), "http" | "https").then(|| url.to_string())
}

#[cfg(test)]
mod tests {
    use super::lead_image_from_html;

    #[test]
    fn lead_image_precedence_resolves_safe_substantial_article_images() {
        let cases = [
            (
                r#"<meta property="og:image" content="/og.jpg">
                <article><img src="/article.jpg" width="800"></article>"#,
                Some("https://example.com/og.jpg"),
            ),
            (
                r#"<meta name="twitter:image" content="https://cdn.example/twitter.jpg">"#,
                Some("https://cdn.example/twitter.jpg"),
            ),
            (
                r#"<article><img src="pixel.gif" width="1" height="1">
                <img src="cover.jpg" width="640" height="480"></article>"#,
                Some("https://example.com/books/cover.jpg"),
            ),
            (
                r#"<article><img src="data:image/png;base64,bad"></article>"#,
                None,
            ),
            (r#"<article><img src="javascript:bad()"></article>"#, None),
        ];
        for (html, expected) in cases {
            assert_eq!(
                lead_image_from_html(html, Some("https://example.com/books/page")).as_deref(),
                expected,
                "{html}"
            );
        }
    }
}
