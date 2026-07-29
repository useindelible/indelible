use url::Url;

use crate::ports::{ParsedFeed, ParsedFeedEntry, ParsedFeedKind};
use ind_domain::{FeedType, FeedVisibility, UserId};

use super::ParsedFeedMetadata;
use crate::handlers::feed::ResolvedFeedSource;

pub(super) fn parsed_feed_metadata(
    feed: &ParsedFeed,
    forced_type: Option<FeedType>,
) -> ParsedFeedMetadata {
    let feed_type =
        forced_type.unwrap_or_else(|| detect_feed_type(feed.kind, false, &feed.entries));
    ParsedFeedMetadata {
        title: feed.title.clone().unwrap_or_else(|| "Untitled Feed".into()),
        description: feed.description.clone(),
        site_url: feed
            .links
            .iter()
            .find(|l| l.rel.as_deref() != Some("self"))
            .map(|l| l.href.clone()),
        image_url: feed.icon_url.clone().or_else(|| feed.logo_url.clone()),
        feed_type,
    }
}
pub(super) fn build_resolved_source(
    user_id: UserId,
    poll_url: &str,
    metadata: ParsedFeedMetadata,
    visibility: FeedVisibility,
    provider: Option<String>,
    is_resolvable: bool,
) -> ResolvedFeedSource {
    let source_url = metadata
        .site_url
        .clone()
        .unwrap_or_else(|| poll_url.to_string());
    let canonical_key = if visibility == FeedVisibility::Private {
        format!("private:{}:{}", user_id, normalize_url(poll_url))
    } else if let Some(key) = youtube_canonical_key(poll_url) {
        key
    } else {
        format!("public:url:{}", normalize_url(poll_url))
    };
    let domain = Url::parse(&source_url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
        .or_else(|| {
            Url::parse(poll_url)
                .ok()
                .and_then(|u| u.host_str().map(|h| h.to_string()))
        });

    ResolvedFeedSource {
        canonical_key,
        source_url,
        poll_url: poll_url.to_string(),
        title: metadata.title,
        description: metadata.description,
        site_url: metadata.site_url,
        image_url: metadata.image_url,
        domain,
        feed_type: metadata.feed_type,
        visibility,
        provider,
        is_resolvable,
    }
}
pub(super) fn detect_feed_type(
    parsed_kind: ParsedFeedKind,
    is_youtube: bool,
    entries: &[ParsedFeedEntry],
) -> FeedType {
    if is_youtube {
        return FeedType::Youtube;
    }

    if has_audio_enclosures(entries) {
        return FeedType::Podcast;
    }

    match parsed_kind {
        ParsedFeedKind::Atom => FeedType::Atom,
        _ => FeedType::Rss,
    }
}
pub(super) fn has_audio_enclosures(entries: &[ParsedFeedEntry]) -> bool {
    entries.iter().any(|entry| {
        entry.media_contents.iter().any(|content| {
            content
                .content_type
                .as_deref()
                .is_some_and(|ct| ct.starts_with("audio"))
        }) || entry.links.iter().any(|link| {
            link.media_type
                .as_deref()
                .is_some_and(|mt| mt.starts_with("audio/"))
        })
    })
}
pub(super) fn normalize_url(url: &str) -> String {
    let mut parsed = match Url::parse(url) {
        Ok(parsed) => parsed,
        Err(_) => return url.trim().to_string(),
    };
    let _ = parsed.set_password(None);
    parsed.set_fragment(None);
    if let Some(host) = parsed.host_str().map(|host| host.to_ascii_lowercase()) {
        let _ = parsed.set_host(Some(&host));
    }
    if parsed.path() != "/" {
        let trimmed = parsed.path().trim_end_matches('/').to_string();
        parsed.set_path(&trimmed);
    }
    parsed.to_string()
}
pub(super) fn youtube_canonical_key(poll_url: &str) -> Option<String> {
    let parsed = Url::parse(poll_url).ok()?;
    if !parsed
        .host_str()
        .is_some_and(|host| host.contains("youtube.com"))
    {
        return None;
    }
    if parsed.path() != "/feeds/videos.xml" {
        return None;
    }
    for (key, value) in parsed.query_pairs() {
        if key == "channel_id" {
            return Some(format!("youtube:channel:{value}"));
        }
        if key == "user" {
            return Some(format!("youtube:user:{}", value.to_ascii_lowercase()));
        }
    }
    None
}
pub(in crate::handlers::feed) fn is_private_feed_url(parsed: &Url) -> bool {
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return true;
    }

    if parsed.query_pairs().any(|(key, _)| {
        matches!(
            key.as_ref(),
            "token" | "rss_token" | "auth" | "apikey" | "api_key" | "key" | "access_token"
        )
    }) {
        return true;
    }

    let path = parsed.path().to_ascii_lowercase();
    (path.contains("/collections/") || path.contains("/collection/")) && path.ends_with("/rss")
}
pub(super) fn looks_like_html(content_type: Option<&str>, body: &[u8]) -> bool {
    content_type.is_some_and(|ct| ct.contains("text/html"))
        || std::str::from_utf8(body)
            .ok()
            .is_some_and(|body| body.to_ascii_lowercase().contains("<html"))
}
pub(in crate::handlers::feed) fn discover_feed_url(
    base_url: &Url,
    html: Option<&str>,
) -> Option<Url> {
    let html = html?;
    let lower = html.to_ascii_lowercase();
    let mut start = 0usize;
    while let Some(link_pos) = lower[start..].find("<link") {
        let absolute_pos = start + link_pos;
        let end = lower[absolute_pos..].find('>')? + absolute_pos;
        let segment = &html[absolute_pos..=end];
        let lower_segment = &lower[absolute_pos..=end];
        if lower_segment.contains("alternate")
            && (lower_segment.contains("application/rss+xml")
                || lower_segment.contains("application/atom+xml"))
            && let Some(href) = extract_href(segment)
            && let Ok(url) = base_url.join(&href)
        {
            return Some(url);
        }
        start = end + 1;
    }
    None
}
pub(super) fn extract_href(segment: &str) -> Option<String> {
    let lower = segment.to_ascii_lowercase();
    let href_pos = lower.find("href=")?;
    let value = &segment[href_pos + 5..];
    let mut chars = value.chars();
    let quote = chars.next()?;
    if quote == '"' || quote == '\'' {
        let rest: String = chars.collect();
        let end = rest.find(quote)?;
        Some(rest[..end].to_string())
    } else {
        let end = value
            .find(|c: char| c.is_ascii_whitespace() || c == '>')
            .unwrap_or(value.len());
        Some(value[..end].trim().to_string())
    }
}
