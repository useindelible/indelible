mod prepare;
mod reader_allowlist;
mod toc;

pub use prepare::{ANCHOR_ID_PREFIX, PrepareError, prepare_reader_html};
use scraper::{ElementRef, Html, Node, Selector};
pub use toc::{ArticleToc, ArticleTocEntry, ArticleTocStatus, derive_article_toc};

pub fn html_to_text(html: &str) -> String {
    let document = Html::parse_document(html);
    #[expect(
        clippy::expect_used,
        reason = "\"body\" is a constant, always-valid CSS selector"
    )]
    let body_selector = Selector::parse("body").expect("body selector is valid");
    let root = document
        .select(&body_selector)
        .next()
        .unwrap_or_else(|| document.root_element());
    let mut parts = Vec::new();

    for node in root.descendants() {
        let Node::Text(text) = node.value() else {
            continue;
        };
        if node
            .ancestors()
            .filter_map(ElementRef::wrap)
            .any(|element| is_non_content_html_element(&element))
        {
            continue;
        }

        let text = text.trim();
        if !text.is_empty() {
            parts.push(text);
        }
    }

    normalize_whitespace(&parts.join(" "))
}

pub fn html_to_markdown(html: &str) -> String {
    let html = html_body_fragment(html);
    html2md::parse_html(&html).trim().to_string()
}

fn html_body_fragment(html: &str) -> String {
    let document = Html::parse_document(html);
    #[expect(
        clippy::expect_used,
        reason = "\"body\" is a constant, always-valid CSS selector"
    )]
    let body_selector = Selector::parse("body").expect("body selector is valid");
    document
        .select(&body_selector)
        .next()
        .map(|body| body.inner_html())
        .unwrap_or_else(|| html.to_string())
}

/// Keeps `class`, `span[data-t]`, and YouTube-embed iframes; drops every other iframe.
/// Infallible: fallback for `prepare_reader_html` on the content write paths.
pub fn sanitize_reader_html(html: &str) -> String {
    match reader_allowlist::drop_foreign_iframes(html) {
        Ok(filtered) => reader_allowlist::reader_sanitizer()
            .clean(&filtered)
            .to_string(),
        Err(_) => reader_allowlist::reader_sanitizer()
            .rm_tags(&["iframe"])
            .clean(html)
            .to_string(),
    }
}

fn is_non_content_html_element(element: &ElementRef<'_>) -> bool {
    matches!(
        element.value().name(),
        "head" | "title" | "style" | "script" | "noscript" | "template" | "svg" | "canvas"
    ) || element.value().attr("hidden").is_some()
        || element.value().attr("aria-hidden") == Some("true")
}

fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
