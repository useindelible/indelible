use ind_domain::ItemType;

/// Infer the library content type for a URL-bearing save.
///
/// This intentionally recognizes only concrete Twitter/X status URLs as tweets.
/// Profile pages, RSSHub feed URLs, and ordinary articles with social tracking
/// parameters remain articles.
pub fn infer_item_type_for_url(url: &str) -> ItemType {
    inferred_url_item_type(url).unwrap_or(ItemType::Article)
}

fn inferred_url_item_type(url: &str) -> Option<ItemType> {
    if is_youtube_url(url) {
        Some(ItemType::Video)
    } else if is_twitter_status_url(url) {
        Some(ItemType::Tweet)
    } else {
        None
    }
}

/// Returns true when `url` identifies a supported YouTube video.
///
/// Recognised forms:
/// - `https://www.youtube.com/watch?v=...`
/// - `https://m.youtube.com/watch?v=...`
/// - `https://music.youtube.com/watch?v=...`
/// - `https://youtube.com/watch?v=...`
/// - `https://youtube.com/shorts/...`
/// - `https://youtu.be/...`
pub fn is_youtube_url(url: &str) -> bool {
    ind_domain::youtube_video_id(url).is_some()
}

/// Returns true for direct Twitter/X status URLs.
pub fn is_twitter_status_url(url: &str) -> bool {
    let Ok(parsed) = url::Url::parse(url) else {
        return false;
    };
    let Some(host) = parsed.host_str() else {
        return false;
    };
    let host = host.to_ascii_lowercase();
    if !matches!(
        host.as_str(),
        "x.com" | "www.x.com" | "twitter.com" | "www.twitter.com" | "mobile.twitter.com"
    ) {
        return false;
    }

    let segments = parsed
        .path_segments()
        .map(|segments| segments.collect::<Vec<_>>())
        .unwrap_or_default();
    match segments.as_slice() {
        [handle, "status", status_id, ..] => {
            !handle.is_empty()
                && !status_id.is_empty()
                && status_id.chars().all(|c| c.is_ascii_digit())
        }
        ["i", "web", "status", status_id, ..] => {
            !status_id.is_empty() && status_id.chars().all(|c| c.is_ascii_digit())
        }
        _ => false,
    }
}

/// Extract the stable video ID from a supported YouTube URL.
pub fn extract_youtube_video_id(url: &str) -> Option<String> {
    ind_domain::youtube_video_id(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn youtube_shorts_use_the_video_dispatch_path() {
        let url = "https://www.youtube.com/shorts/abc123?feature=share";

        assert!(is_youtube_url(url));
        assert_eq!(extract_youtube_video_id(url).as_deref(), Some("abc123"));
        assert_eq!(infer_item_type_for_url(url), ItemType::Video);
    }
}
