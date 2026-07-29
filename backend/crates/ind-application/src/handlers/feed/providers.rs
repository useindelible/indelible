use url::Url;

pub(super) enum YoutubeRouteKind {
    Handle(String),     // RSSHub path: /youtube/user/@{handle}
    ChannelId(String),  // RSSHub path: /youtube/channel/{id}
    LegacyUser(String), // RSSHub path: /youtube/user/{name}
    CustomUrl(String),  // RSSHub path: /youtube/c/{name}
}

pub(super) struct YoutubeParseResult {
    pub(super) route_kind: YoutubeRouteKind,
    pub(super) canonical_key: String,
    pub(super) public_source_url: String,
    pub(super) input_provider: Option<String>,
}

pub(super) fn parse_youtube_url(parsed: &Url) -> Option<YoutubeParseResult> {
    let host = parsed.host_str()?.to_ascii_lowercase();

    if host.contains("rsshub.") {
        let segments: Vec<&str> = parsed.path().split('/').filter(|s| !s.is_empty()).collect();
        if segments.len() >= 3 && segments[0] == "youtube" {
            return match segments[1] {
                "user" => {
                    let raw = segments[2];
                    let name = raw.trim_start_matches('@').to_ascii_lowercase();
                    let is_handle = raw.starts_with('@');
                    Some(YoutubeParseResult {
                        route_kind: if is_handle {
                            YoutubeRouteKind::Handle(name.clone())
                        } else {
                            YoutubeRouteKind::LegacyUser(name.clone())
                        },
                        canonical_key: format!("youtube:user:{name}"),
                        public_source_url: if is_handle {
                            format!("https://www.youtube.com/@{name}")
                        } else {
                            format!("https://www.youtube.com/user/{name}")
                        },
                        input_provider: Some("rsshub".into()),
                    })
                }
                "channel" => {
                    let id = segments[2].to_string();
                    Some(YoutubeParseResult {
                        route_kind: YoutubeRouteKind::ChannelId(id.clone()),
                        canonical_key: format!("youtube:channel:{id}"),
                        public_source_url: format!("https://www.youtube.com/channel/{id}"),
                        input_provider: Some("rsshub".into()),
                    })
                }
                "c" => {
                    let name = segments[2].to_ascii_lowercase();
                    Some(YoutubeParseResult {
                        route_kind: YoutubeRouteKind::CustomUrl(name.clone()),
                        canonical_key: format!("youtube:custom:{name}"),
                        public_source_url: format!("https://www.youtube.com/c/{name}"),
                        input_provider: Some("rsshub".into()),
                    })
                }
                _ => None,
            };
        }
        return None;
    }

    if host != "youtube.com" && host != "www.youtube.com" {
        return None;
    }

    // Skip native RSS feed URLs — fall through to generic RSS handler
    if parsed.path() == "/feeds/videos.xml" {
        return None;
    }

    let segments: Vec<&str> = parsed.path().split('/').filter(|s| !s.is_empty()).collect();
    if segments.is_empty() {
        return None;
    }

    if segments[0].starts_with('@') {
        let handle = segments[0].trim_start_matches('@').to_ascii_lowercase();
        if handle.is_empty() {
            return None;
        }
        return Some(YoutubeParseResult {
            route_kind: YoutubeRouteKind::Handle(handle.clone()),
            canonical_key: format!("youtube:user:{handle}"),
            public_source_url: format!("https://www.youtube.com/@{handle}"),
            input_provider: None,
        });
    }

    if segments.len() < 2 {
        return None;
    }

    match segments[0] {
        "channel" => {
            let id = segments[1].to_string();
            Some(YoutubeParseResult {
                route_kind: YoutubeRouteKind::ChannelId(id.clone()),
                canonical_key: format!("youtube:channel:{id}"),
                public_source_url: format!("https://www.youtube.com/channel/{id}"),
                input_provider: None,
            })
        }
        "user" => {
            let name = segments[1].to_ascii_lowercase();
            Some(YoutubeParseResult {
                route_kind: YoutubeRouteKind::LegacyUser(name.clone()),
                canonical_key: format!("youtube:user:{name}"),
                public_source_url: format!("https://www.youtube.com/user/{name}"),
                input_provider: None,
            })
        }
        "c" => {
            let name = segments[1].to_ascii_lowercase();
            Some(YoutubeParseResult {
                route_kind: YoutubeRouteKind::CustomUrl(name.clone()),
                canonical_key: format!("youtube:custom:{name}"),
                public_source_url: format!("https://www.youtube.com/c/{name}"),
                input_provider: None,
            })
        }
        _ => None,
    }
}

/// Extracts the channel id from a fetched YouTube channel page. Only the
/// canonical link (or the page's own RSS alternate link) names the page's
/// channel — the body embeds other channels' ids in recommendation shelves,
/// so a bare `channelId` scan would misresolve.
pub(super) fn channel_id_from_page(html: &str) -> Option<String> {
    let canonical = link_tags(html).find_map(|tag| {
        let rel = attribute_value(tag, "rel")?;
        if !rel.eq_ignore_ascii_case("canonical") {
            return None;
        }
        let href = attribute_value(tag, "href")?;
        let idx = href.find("/channel/")?;
        youtube_channel_id_at(&href[idx + "/channel/".len()..])
    });
    if canonical.is_some() {
        return canonical;
    }

    link_tags(html).find_map(|tag| {
        let href = attribute_value(tag, "href")?;
        let idx = href.find("channel_id=")?;
        youtube_channel_id_at(&href[idx + "channel_id=".len()..])
    })
}

/// The inner text of every `<link …>` tag, so attributes are read per tag
/// rather than by scanning forward from one of them.
fn link_tags(html: &str) -> impl Iterator<Item = &str> {
    html.split('<').filter_map(|chunk| {
        let rest = chunk
            .strip_prefix("link")
            .or_else(|| chunk.strip_prefix("LINK"))?;
        if !rest.starts_with(|c: char| c.is_ascii_whitespace()) {
            return None;
        }
        Some(&rest[..rest.find('>').unwrap_or(rest.len())])
    })
}

/// Reads one attribute out of a tag body regardless of attribute order or
/// quote style (double, single, or unquoted).
fn attribute_value<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let mut rest = tag;
    loop {
        let idx = rest.to_ascii_lowercase().find(name)?;
        let after_name = &rest[idx + name.len()..];
        let before_ok = idx == 0
            || rest[..idx]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_ascii_whitespace());
        let value = after_name.trim_start();
        if before_ok && let Some(value) = value.strip_prefix('=') {
            let value = value.trim_start();
            return match value.chars().next() {
                Some(quote @ ('"' | '\'')) => {
                    let value = &value[quote.len_utf8()..];
                    Some(&value[..value.find(quote)?])
                }
                // An unquoted value ends at whitespace — a slash belongs to the
                // value (`http://…`) unless it is the tag's self-closing mark.
                Some(_) => {
                    let raw = &value[..value.find(char::is_whitespace).unwrap_or(value.len())];
                    Some(raw.strip_suffix('/').unwrap_or(raw))
                }
                None => None,
            };
        }
        rest = &rest[idx + name.len()..];
    }
}

/// Reads a channel id at the start of `slice`: `UC` followed by 22 id chars.
fn youtube_channel_id_at(slice: &str) -> Option<String> {
    let id: String = slice
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .take(25)
        .collect();
    (id.len() == 24 && id.starts_with("UC")).then_some(id)
}

pub(super) struct TwitterParseResult {
    pub(super) handle: String,
    pub(super) canonical_key: String,
    pub(super) public_source_url: String,
    pub(super) input_provider: Option<String>,
}

pub(super) fn parse_twitter_url(parsed: &Url) -> Option<TwitterParseResult> {
    let host = parsed.host_str()?.to_ascii_lowercase();
    let segments: Vec<&str> = parsed.path().split('/').filter(|s| !s.is_empty()).collect();
    if segments.is_empty() {
        return None;
    }

    if host.contains("nitter.") && segments.len() >= 2 && segments[1] == "rss" {
        let handle = segments[0].trim_start_matches('@').to_ascii_lowercase();
        return Some(TwitterParseResult {
            canonical_key: format!("twitter:user:{handle}"),
            public_source_url: format!("https://x.com/{handle}"),
            input_provider: Some("nitter".into()),
            handle,
        });
    }

    if host.contains("rsshub.")
        && segments.len() >= 3
        && segments[0] == "twitter"
        && segments[1] == "user"
    {
        let handle = segments[2].trim_start_matches('@').to_ascii_lowercase();
        return Some(TwitterParseResult {
            canonical_key: format!("twitter:user:{handle}"),
            public_source_url: format!("https://x.com/{handle}"),
            input_provider: Some("rsshub".into()),
            handle,
        });
    }

    if host == "x.com" || host == "www.x.com" || host == "twitter.com" || host == "www.twitter.com"
    {
        let handle = segments[0].trim_start_matches('@').to_ascii_lowercase();
        if ["home", "explore", "search", "intent", "i", "share"].contains(&handle.as_str()) {
            return None;
        }
        return Some(TwitterParseResult {
            canonical_key: format!("twitter:user:{handle}"),
            public_source_url: format!("https://x.com/{handle}"),
            input_provider: None,
            handle,
        });
    }

    None
}
