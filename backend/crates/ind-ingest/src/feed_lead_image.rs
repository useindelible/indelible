//! Lead-image extraction for feed entries.
//!
//! Mirrors the browser extension's `extractCoverUrl()` precedence, adapted to the data a feed
//! exposes server-side: publisher-declared media (`media:thumbnail` / image `media:content`) is the
//! analog of `og:image` and comes first; the first substantial `<img>` in the entry content is the
//! fallback. Only `http(s)` URLs are accepted; relative URLs resolve against the entry URL.

use feed_rs::model::Entry;
use scraper::{Html, Selector};
use url::Url;

/// Extract a lead-image URL for a feed entry, or `None` when nothing usable is present.
pub fn extract_feed_lead_image(
    entry: &Entry,
    content_html: Option<&str>,
    summary_html: Option<&str>,
    base_url: Option<&str>,
) -> Option<String> {
    media_image(entry)
        .and_then(|raw| normalize_image_url(&raw, base_url))
        .or_else(|| {
            // Content (`<content:encoded>`) preferred; summary (`<description>`) is the fallback for
            // feeds that only carry their body HTML there (e.g. many RSS 2.0 feeds).
            [content_html, summary_html]
                .into_iter()
                .flatten()
                .filter_map(first_content_image)
                .find_map(|raw| normalize_image_url(&raw, base_url))
        })
}

/// First image-typed `media:content` URL, else the first `media:thumbnail`.
fn media_image(entry: &Entry) -> Option<String> {
    for media in &entry.media {
        if let Some(url) = media
            .content
            .iter()
            .find(|content| content.url.is_some() && is_image_content_type(content))
            .and_then(|content| content.url.as_ref())
        {
            return Some(url.to_string());
        }
        if let Some(thumbnail) = media
            .thumbnails
            .iter()
            .find(|thumbnail| !thumbnail.image.uri.trim().is_empty())
        {
            return Some(thumbnail.image.uri.clone());
        }
    }
    None
}

fn is_image_content_type(content: &feed_rs::model::MediaContent) -> bool {
    content
        .content_type
        .as_ref()
        .is_some_and(|mime| mime.to_string().starts_with("image/"))
}

/// First non-tracking `<img src>` in an HTML fragment.
fn first_content_image(html: &str) -> Option<String> {
    let document = Html::parse_fragment(html);
    let selector = Selector::parse("img").ok()?;
    for element in document.select(&selector) {
        let value = element.value();
        let Some(src) = value
            .attr("src")
            .map(str::trim)
            .filter(|src| !src.is_empty())
        else {
            continue;
        };
        if is_tracking_pixel(value) {
            continue;
        }
        return Some(src.to_string());
    }
    None
}

/// Skip images whose declared width or height is below 100px (tracking pixels and spacers).
/// Non-numeric dimensions (e.g. `100px`, `100%`) are treated as unknown and not skipped.
fn is_tracking_pixel(element: &scraper::node::Element) -> bool {
    let too_small = |attr: &str| {
        element
            .attr(attr)
            .and_then(|raw| raw.parse::<u32>().ok())
            .is_some_and(|dim| dim < 100)
    };
    too_small("width") || too_small("height")
}

/// Accept absolute `http(s)` URLs; resolve relative/protocol-relative URLs against `base_url`.
/// Reject empty, `data:`, and non-`http(s)` URLs.
fn normalize_image_url(raw: &str, base_url: Option<&str>) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() || raw.starts_with("data:") {
        return None;
    }

    if let Ok(parsed) = Url::parse(raw) {
        return http_or_none(parsed);
    }

    let base = base_url.and_then(|base| Url::parse(base).ok())?;
    base.join(raw).ok().and_then(http_or_none)
}

fn http_or_none(url: Url) -> Option<String> {
    matches!(url.scheme(), "http" | "https").then(|| url.to_string())
}
