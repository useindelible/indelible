mod prepare;
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

/// Sanitize untrusted HTML for safe rendering in the reader. Uses ammonia's
/// vetted default allowlist, which strips `<script>`/`<style>`/`<iframe>`,
/// inline event handlers, and dangerous URL schemes while preserving article
/// formatting. Content write paths call `prepare_reader_html` (which sanitizes
/// AND makes anchors durable); this remains the documented fallback when
/// preparation fails, so a rewriter error can never fail ingest.
pub fn sanitize_reader_html(html: &str) -> String {
    ammonia::clean(html)
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
