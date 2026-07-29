use url::Url;

use crate::error::AppError;
use crate::handlers::provider_candidates::{
    ProviderCandidate, instance_for_url, twitter_candidates, youtube_candidates,
};
use crate::ports::{FetchRequest, HttpFetchError};
use ind_domain::{DomainError, FeedType, FeedVisibility, UserId};

use super::providers::{YoutubeRouteKind, parse_twitter_url, parse_youtube_url};
use super::{FeedService, ResolvedFeedSource};

mod helpers;
use helpers::{build_resolved_source, looks_like_html, normalize_url, parsed_feed_metadata};
pub(super) use helpers::{discover_feed_url, is_private_feed_url};

#[derive(Debug, Clone)]
pub(super) struct ParsedFeedMetadata {
    pub(super) title: String,
    pub(super) description: Option<String>,
    pub(super) site_url: Option<String>,
    pub(super) image_url: Option<String>,
    pub(super) feed_type: FeedType,
}

impl FeedService {
    pub async fn resolve_source(
        &self,
        user_id: UserId,
        raw_url: &str,
    ) -> Result<ResolvedFeedSource, AppError> {
        let parsed = Url::parse(raw_url).map_err(|_| {
            AppError::Domain(DomainError::Validation {
                field: "url".into(),
                message: "must be a valid absolute URL".into(),
            })
        })?;

        let visibility = if is_private_feed_url(&parsed) {
            FeedVisibility::Private
        } else {
            FeedVisibility::Public
        };

        if let Some(resolved) = self
            .try_resolve_twitter_source(user_id, &parsed, visibility)
            .await?
        {
            return Ok(resolved);
        }

        if let Some(resolved) = self
            .try_resolve_youtube_source(user_id, &parsed, visibility)
            .await?
        {
            return Ok(resolved);
        }

        let (poll_url, metadata) = self.validate_or_discover_feed(&parsed, None).await?;
        let visibility = if is_private_feed_url(&poll_url) {
            FeedVisibility::Private
        } else {
            visibility
        };
        Ok(build_resolved_source(
            user_id,
            poll_url.as_str(),
            metadata,
            visibility,
            None,
            false,
        ))
    }
    async fn try_resolve_twitter_source(
        &self,
        user_id: UserId,
        parsed: &Url,
        visibility: FeedVisibility,
    ) -> Result<Option<ResolvedFeedSource>, AppError> {
        let Some(twitter) = parse_twitter_url(parsed) else {
            return Ok(None);
        };

        let instances = self.feed_repo.list_all_enabled_provider_instances().await?;

        let mut candidates: Vec<ProviderCandidate> = Vec::new();

        // The user's URL goes first so an explicit choice wins over our priorities.
        if let Some(provider) = twitter.input_provider.clone() {
            let url = parsed.as_str().to_string();
            candidates.push(ProviderCandidate {
                instance_id: instance_for_url(&instances, &url).map(|i| i.id),
                url,
                provider_type: provider,
            });
        }

        for cand in twitter_candidates(&twitter.handle, &instances) {
            if !candidates.iter().any(|c| c.url == cand.url) {
                candidates.push(cand);
            }
        }

        let mut last_error = None;
        for candidate in &candidates {
            let candidate_url = match Url::parse(&candidate.url) {
                Ok(url) => url,
                Err(_) => continue,
            };
            match self
                .validate_or_discover_feed(&candidate_url, Some(FeedType::Twitter))
                .await
            {
                Ok((_, metadata)) => {
                    if let Some(id) = candidate.instance_id {
                        let _ = self.feed_repo.record_provider_instance_success(id).await;
                    }
                    return Ok(Some(ResolvedFeedSource {
                        canonical_key: if visibility == FeedVisibility::Private {
                            format!(
                                "private:{}:{}",
                                user_id,
                                normalize_url(candidate_url.as_str())
                            )
                        } else {
                            twitter.canonical_key.clone()
                        },
                        source_url: twitter.public_source_url.clone(),
                        poll_url: candidate.url.clone(),
                        title: metadata.title,
                        description: metadata.description,
                        site_url: metadata.site_url,
                        image_url: metadata.image_url,
                        domain: Url::parse(&twitter.public_source_url)
                            .ok()
                            .and_then(|url| url.host_str().map(|host| host.to_string())),
                        feed_type: FeedType::Twitter,
                        visibility,
                        provider: Some(candidate.provider_type.clone()),
                        is_resolvable: true,
                    }));
                }
                Err(err) => {
                    if let Some(id) = candidate.instance_id {
                        let _ = self.feed_repo.record_provider_instance_failure(id).await;
                    }
                    last_error = Some(err);
                }
            }
        }

        if twitter.input_provider.is_some() {
            return Err(last_error.unwrap_or_else(|| AppError::ExternalService {
                service: "feed_fetch".into(),
                message: "no public Twitter RSS provider returned a valid feed".into(),
            }));
        }

        Ok(None)
    }
    async fn try_resolve_youtube_source(
        &self,
        user_id: UserId,
        parsed: &Url,
        visibility: FeedVisibility,
    ) -> Result<Option<ResolvedFeedSource>, AppError> {
        let Some(youtube) = parse_youtube_url(parsed) else {
            return Ok(None);
        };

        // Native Atom first: YouTube serves an official per-channel feed that
        // outlives any third-party RSSHub instance. RSSHub remains the
        // fallback when channel-id resolution or the native feed fails.
        if let Some(channel_id) = self.resolve_youtube_channel_id(&youtube.route_kind).await {
            let atom_url =
                format!("https://www.youtube.com/feeds/videos.xml?channel_id={channel_id}");
            if let Ok(atom_parsed) = Url::parse(&atom_url)
                && let Ok((_, metadata)) = self
                    .validate_or_discover_feed(&atom_parsed, Some(FeedType::Youtube))
                    .await
            {
                return Ok(Some(ResolvedFeedSource {
                    canonical_key: if visibility == FeedVisibility::Private {
                        format!("private:{}:{}", user_id, normalize_url(&atom_url))
                    } else {
                        youtube.canonical_key.clone()
                    },
                    source_url: youtube.public_source_url.clone(),
                    poll_url: atom_url,
                    title: metadata.title,
                    description: metadata.description,
                    site_url: metadata.site_url,
                    image_url: metadata.image_url,
                    domain: Url::parse(&youtube.public_source_url)
                        .ok()
                        .and_then(|url| url.host_str().map(|host| host.to_string())),
                    feed_type: FeedType::Youtube,
                    visibility,
                    provider: None,
                    is_resolvable: true,
                }));
            }
        }

        let instances = self.feed_repo.list_all_enabled_provider_instances().await?;

        let rsshub_path = match &youtube.route_kind {
            YoutubeRouteKind::Handle(h) => format!("/youtube/user/@{h}"),
            YoutubeRouteKind::ChannelId(id) => format!("/youtube/channel/{id}"),
            YoutubeRouteKind::LegacyUser(name) => format!("/youtube/user/{name}"),
            YoutubeRouteKind::CustomUrl(name) => format!("/youtube/c/{name}"),
        };

        let mut candidates: Vec<ProviderCandidate> = Vec::new();

        if let Some(provider) = youtube.input_provider.clone() {
            let url = parsed.as_str().to_string();
            candidates.push(ProviderCandidate {
                instance_id: instance_for_url(&instances, &url).map(|i| i.id),
                url,
                provider_type: provider,
            });
        }

        for cand in youtube_candidates(&rsshub_path, &instances) {
            if !candidates.iter().any(|c| c.url == cand.url) {
                candidates.push(cand);
            }
        }

        let mut last_error = None;
        for candidate in &candidates {
            let candidate_url = match Url::parse(&candidate.url) {
                Ok(url) => url,
                Err(_) => continue,
            };
            match self
                .validate_or_discover_feed(&candidate_url, Some(FeedType::Youtube))
                .await
            {
                Ok((_, metadata)) => {
                    if let Some(id) = candidate.instance_id {
                        let _ = self.feed_repo.record_provider_instance_success(id).await;
                    }
                    return Ok(Some(ResolvedFeedSource {
                        canonical_key: if visibility == FeedVisibility::Private {
                            format!(
                                "private:{}:{}",
                                user_id,
                                normalize_url(candidate_url.as_str())
                            )
                        } else {
                            youtube.canonical_key.clone()
                        },
                        source_url: youtube.public_source_url.clone(),
                        poll_url: candidate.url.clone(),
                        title: metadata.title,
                        description: metadata.description,
                        site_url: metadata.site_url,
                        image_url: metadata.image_url,
                        domain: Url::parse(&youtube.public_source_url)
                            .ok()
                            .and_then(|url| url.host_str().map(|host| host.to_string())),
                        feed_type: FeedType::Youtube,
                        visibility,
                        provider: Some(candidate.provider_type.clone()),
                        is_resolvable: true,
                    }));
                }
                Err(err) => {
                    if let Some(id) = candidate.instance_id {
                        let _ = self.feed_repo.record_provider_instance_failure(id).await;
                    }
                    last_error = Some(err);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| AppError::ExternalService {
            service: "feed_fetch".into(),
            message: if candidates.is_empty() {
                "no RSSHub instances are configured for YouTube resolution".into()
            } else {
                "no RSSHub instance returned a valid YouTube feed for this channel".into()
            },
        }))
    }
    /// Resolves a route to its channel id: directly for channel URLs, via the
    /// public channel page's canonical link for handles and legacy names.
    /// `None` means "could not resolve" — the caller falls back to RSSHub.
    async fn resolve_youtube_channel_id(&self, route: &YoutubeRouteKind) -> Option<String> {
        let page_url = match route {
            YoutubeRouteKind::ChannelId(id) => return Some(id.clone()),
            YoutubeRouteKind::Handle(handle) => format!("https://www.youtube.com/@{handle}"),
            YoutubeRouteKind::LegacyUser(name) => format!("https://www.youtube.com/user/{name}"),
            YoutubeRouteKind::CustomUrl(name) => format!("https://www.youtube.com/c/{name}"),
        };
        let response = self
            .http_fetcher
            .fetch(
                FetchRequest::new(&page_url)
                    .with_header("User-Agent", "Indelible/1.0 (Feed Resolver)"),
            )
            .await
            .ok()?;
        if !response.is_success() {
            return None;
        }
        super::providers::channel_id_from_page(std::str::from_utf8(&response.body).ok()?)
    }

    async fn validate_or_discover_feed(
        &self,
        url: &Url,
        forced_type: Option<FeedType>,
    ) -> Result<(Url, ParsedFeedMetadata), AppError> {
        self.validate_or_discover_feed_inner(url, forced_type, 0)
            .await
    }
    async fn validate_or_discover_feed_inner(
        &self,
        url: &Url,
        forced_type: Option<FeedType>,
        depth: usize,
    ) -> Result<(Url, ParsedFeedMetadata), AppError> {
        const MAX_DISCOVERY_DEPTH: usize = 4;
        if depth > MAX_DISCOVERY_DEPTH {
            return Err(AppError::Domain(DomainError::Validation {
                field: "url".into(),
                message: "feed discovery exceeded maximum depth".into(),
            }));
        }

        let response = self
            .http_fetcher
            .fetch(
                FetchRequest::new(url.as_str())
                    .with_header("User-Agent", "Indelible/1.0 (Feed Resolver)"),
            )
            .await
            .map_err(|e| match e {
                // The guarded fetcher refused a private/internal target (SSRF):
                // surface as a 422 on the user-supplied URL, not a 5xx.
                HttpFetchError::Disallowed(message) => AppError::Domain(DomainError::Validation {
                    field: "url".into(),
                    message,
                }),
                other => AppError::ExternalService {
                    service: "feed_fetch".into(),
                    message: format!("failed to fetch feed: {other}"),
                },
            })?;

        if !response.is_success() {
            return Err(AppError::ExternalService {
                service: "feed_fetch".into(),
                message: format!("feed returned HTTP {}", response.status),
            });
        }

        let content_type = response.content_type.clone();
        let body = response.body;

        if let Ok(feed) = self.feed_parser.parse(&body[..]) {
            return Ok((url.clone(), parsed_feed_metadata(&feed, forced_type)));
        }

        if looks_like_html(content_type.as_deref(), &body)
            && let Some(discovered_url) = discover_feed_url(url, std::str::from_utf8(&body).ok())
        {
            return Box::pin(self.validate_or_discover_feed_inner(
                &discovered_url,
                forced_type,
                depth + 1,
            ))
            .await;
        }

        Err(AppError::Domain(DomainError::Validation {
            field: "url".into(),
            message: "not a valid feed or feed-discoverable website".into(),
        }))
    }
}
