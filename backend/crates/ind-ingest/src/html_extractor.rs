use scraper::{ElementRef, Html, Selector};

use ind_application::ports::{HtmlExtractor, SpokenHtmlElement};

const SPOKEN_SELECTOR: &str = "h1, h2, h3, h4, h5, h6, p, blockquote, li, figcaption, caption";

pub struct ScraperHtmlExtractor;

impl ScraperHtmlExtractor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ScraperHtmlExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl HtmlExtractor for ScraperHtmlExtractor {
    fn extract_spoken_elements(&self, html: &str) -> Vec<SpokenHtmlElement> {
        let document = Html::parse_document(html);
        #[expect(
            clippy::expect_used,
            reason = "SPOKEN_SELECTOR is a constant, always-valid CSS selector"
        )]
        let selector = Selector::parse(SPOKEN_SELECTOR).expect("valid spoken element selector");
        let mut elements = Vec::new();

        for element in document.select(&selector) {
            if has_spoken_ancestor(&element)
                || is_excluded_element(&element)
                || has_excluded_ancestor(&element)
            {
                continue;
            }

            let text = normalize_whitespace(&element.text().collect::<Vec<_>>().join(" "));
            if text.is_empty() {
                continue;
            }

            elements.push(SpokenHtmlElement {
                tag: element.value().name().to_string(),
                text,
            });
        }

        elements
    }
}

fn normalize_whitespace(text: &str) -> String {
    let mut normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    for punctuation in [".", ",", ";", ":", "!", "?"] {
        normalized = normalized.replace(&format!(" {punctuation}"), punctuation);
    }
    normalized
}

fn has_spoken_ancestor(element: &ElementRef<'_>) -> bool {
    element
        .ancestors()
        .filter_map(ElementRef::wrap)
        .any(|ancestor| is_spoken_tag(ancestor.value().name()))
}

fn has_excluded_ancestor(element: &ElementRef<'_>) -> bool {
    element
        .ancestors()
        .filter_map(ElementRef::wrap)
        .any(|ancestor| is_excluded_element(&ancestor))
}

fn is_spoken_tag(tag: &str) -> bool {
    matches!(
        tag,
        "h1" | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "p"
            | "blockquote"
            | "li"
            | "figcaption"
            | "caption"
    )
}

fn is_excluded_element(element: &ElementRef<'_>) -> bool {
    let value = element.value();
    matches!(
        value.name(),
        "script"
            | "style"
            | "nav"
            | "header"
            | "footer"
            | "aside"
            | "form"
            | "button"
            | "input"
            | "select"
            | "textarea"
            | "noscript"
    ) || value.attr("hidden").is_some()
        || value.attr("aria-hidden") == Some("true")
}
