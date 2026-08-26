//! Sanitizer allowlist shared by every reader-HTML write path.

use lol_html::errors::RewritingError;
use lol_html::{RewriteStrSettings, element, rewrite_str};

pub(crate) const YOUTUBE_EMBED_PREFIXES: [&str; 2] = [
    "https://www.youtube.com/embed/",
    "https://www.youtube-nocookie.com/embed/",
];

/// Never combine with `allowed_classes`: ammonia panics at `clean()` when both are set.
pub(crate) fn reader_sanitizer() -> ammonia::Builder<'static> {
    let mut builder = ammonia::Builder::default();
    builder
        .add_tags(&["iframe"])
        .add_tag_attributes(
            "iframe",
            &[
                "src",
                "width",
                "height",
                "frameborder",
                "allowfullscreen",
                "allow",
            ],
        )
        .add_generic_attributes(&["class"])
        .add_tag_attributes("span", &["data-t"]);
    builder
}

/// Must run before `reader_sanitizer`, which allows `iframe` without checking `src`.
pub(crate) fn drop_foreign_iframes(html: &str) -> Result<String, RewritingError> {
    let settings =
        RewriteStrSettings::new().append_element_content_handler(element!("iframe", |el| {
            let allowed = el.get_attribute("src").is_some_and(|src| {
                YOUTUBE_EMBED_PREFIXES
                    .iter()
                    .any(|prefix| src.starts_with(prefix))
            });
            if !allowed {
                el.remove();
            }
            Ok(())
        }));
    rewrite_str(html, settings)
}
