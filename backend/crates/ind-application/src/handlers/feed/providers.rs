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
