use chrono::Utc;
use reqwest::header::{ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED};

use ind_application::AppError;
use ind_application::handlers::feed::{next_poll_after_failure, next_poll_after_success};
use ind_application::handlers::provider_candidates::{
    ProviderCandidate, instance_for_url, twitter_candidates, twitter_handle_from_canonical,
    youtube_candidates, youtube_rsshub_path_from_canonical,
};
use ind_domain::{
    ActiveSubscription, CanonicalizationConfig, DocumentOriginType, FeedAutosaveJob, FeedDelivery,
    FeedDeliveryId, FeedPollJob, FeedSource, FeedSourceEntry, FeedSourceEntryId, FeedType,
    FeedVisibility, PollOutcome,
};

use crate::context::FeedJobDeps;

use super::http_client::build_feed_http_client;
use super::util::{extract_domain, strip_html};

const MISSING_ENTRY_ID_SENTINEL: &str = "indelible:missing-entry-id";

pub async fn handle_feed_poll(ctx: &FeedJobDeps, job: FeedPollJob) -> Result<(), AppError> {
    let Some(source) = ctx.feed_repo.find_source_by_id(job.source_id).await? else {
        return Ok(());
    };

    let subscriptions = ctx
        .feed_repo
        .list_active_subscriptions_for_source(source.id)
        .await?;
    if subscriptions.is_empty() {
        ctx.feed_repo.clear_source_lease(source.id).await?;
        ctx.feed_repo.delete_source_if_orphaned(source.id).await?;
        return Ok(());
    }

    let client = build_feed_http_client(ctx.egress_policy.clone());
    let now = Utc::now();
    let candidates = build_poll_candidates(ctx, &source).await?;

    let mut last_error: Option<String> = None;
    for (i, candidate) in candidates.iter().enumerate() {
        let is_primary = i == 0;
        match attempt_single_poll(&client, candidate, &source, is_primary).await {
            Ok(PollAttempt::NotModified {
                etag,
                last_modified,
            }) => {
                if let Some(id) = candidate.instance_id {
                    let _ = ctx.feed_repo.record_provider_instance_success(id).await;
                }
                let state = PollOutcome {
                    source_id: source.id,
                    last_polled_at: Some(now),
                    next_poll_at: Some(next_poll_after_success(
                        &subscriptions,
                        now,
                        ctx.feed_poll_schedule,
                    )),
                    last_etag: etag.or_else(|| source.last_etag.clone()),
                    last_modified: last_modified.or_else(|| source.last_modified.clone()),
                    consecutive_failures: 0,
                    last_error: None,
                };
                ctx.feed_repo
                    .mark_source_poll_success(source.id, state, None)
                    .await?;
                return Ok(());
            }
            Ok(PollAttempt::Fetched(fetched)) => {
                let FetchedFeed {
                    feed,
                    etag,
                    last_modified,
                } = *fetched;
                if let Some(id) = candidate.instance_id {
                    let _ = ctx.feed_repo.record_provider_instance_success(id).await;
                }
                let fallback_transport = if candidate.url != source.poll_url {
                    Some((candidate.url.clone(), candidate.provider_type.clone()))
                } else {
                    None
                };
                return persist_polled_feed(
                    ctx,
                    &source,
                    &subscriptions,
                    feed,
                    now,
                    PollResponseMeta {
                        fallback_transport,
                        last_etag: etag,
                        last_modified,
                    },
                )
                .await;
            }
            Err(err) => {
                if let Some(id) = candidate.instance_id {
                    let _ = ctx.feed_repo.record_provider_instance_failure(id).await;
                }
                last_error = Some(err);
            }
        }
    }

    let error = last_error.unwrap_or_else(|| {
        "no provider instance returned a valid feed for this source".to_string()
    });
    record_poll_failure(ctx, &source, &subscriptions, error).await?;
    Ok(())
}

struct FetchedFeed {
    feed: feed_rs::model::Feed,
    etag: Option<String>,
    last_modified: Option<String>,
}

enum PollAttempt {
    NotModified {
        etag: Option<String>,
        last_modified: Option<String>,
    },
    Fetched(Box<FetchedFeed>),
}

async fn build_poll_candidates(
    ctx: &FeedJobDeps,
    source: &FeedSource,
) -> Result<Vec<ProviderCandidate>, AppError> {
    let instances = ctx.feed_repo.list_all_enabled_provider_instances().await?;

    let mut out = Vec::with_capacity(instances.len() + 1);
    out.push(ProviderCandidate {
        instance_id: instance_for_url(&instances, &source.poll_url).map(|i| i.id),
        url: source.poll_url.clone(),
        provider_type: source.provider.clone().unwrap_or_default(),
    });

    if source.visibility != FeedVisibility::Public {
        return Ok(out);
    }

    let alternates = match source.feed_type {
        FeedType::Twitter => twitter_handle_from_canonical(&source.source_url)
            .map(|h| twitter_candidates(&h, &instances))
            .unwrap_or_default(),
        FeedType::Youtube => youtube_rsshub_path_from_canonical(&source.source_url)
            .map(|p| youtube_candidates(&p, &instances))
            .unwrap_or_default(),
        _ => Vec::new(),
    };

    for cand in alternates {
        if !out.iter().any(|c| c.url == cand.url) {
            out.push(cand);
        }
    }

    Ok(out)
}

async fn attempt_single_poll(
    client: &ind_egress::GuardedHttpClient,
    candidate: &ProviderCandidate,
    source: &FeedSource,
    send_validators: bool,
) -> Result<PollAttempt, String> {
    let mut request = client
        .get(&candidate.url)
        .map_err(|e| format!("feed fetch blocked: {}", e.client_message()))?;
    if send_validators {
        if let Some(etag) = source.last_etag.as_deref() {
            request = request.header(IF_NONE_MATCH, etag);
        }
        if let Some(last_modified) = source.last_modified.as_deref() {
            request = request.header(IF_MODIFIED_SINCE, last_modified);
        }
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("feed fetch failed: {e}"))?;

    let etag = response
        .headers()
        .get(ETAG)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string());
    let last_modified = response
        .headers()
        .get(LAST_MODIFIED)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string());

    if response.status() == reqwest::StatusCode::NOT_MODIFIED {
        return Ok(PollAttempt::NotModified {
            etag,
            last_modified,
        });
    }

    if !response.status().is_success() {
        let cf_challenge = response
            .headers()
            .get("cf-mitigated")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.eq_ignore_ascii_case("challenge"))
            .unwrap_or(false);
        return Err(if cf_challenge {
            "feed blocked by Cloudflare challenge (requires browser JS execution)".into()
        } else {
            format!("feed returned HTTP {}", response.status())
        });
    }

    let body = response
        .bytes()
        .await
        .map_err(|e| format!("failed to read feed body: {e}"))?;

    let parser = feed_rs::parser::Builder::new()
        .id_generator(|links, title, uri| {
            if links.is_empty() && title.is_none() {
                MISSING_ENTRY_ID_SENTINEL.to_string()
            } else {
                feed_rs::parser::generate_id(links, title, uri)
            }
        })
        .build();
    let feed = parser
        .parse(&body[..])
        .map_err(|e| format!("invalid feed payload: {e}"))?;

    Ok(PollAttempt::Fetched(Box::new(FetchedFeed {
        feed,
        etag,
        last_modified,
    })))
}

struct PollResponseMeta {
    fallback_transport: Option<(String, String)>,
    last_etag: Option<String>,
    last_modified: Option<String>,
}

async fn persist_polled_feed(
    ctx: &FeedJobDeps,
    source: &FeedSource,
    subscriptions: &[ActiveSubscription],
    feed: feed_rs::model::Feed,
    now: chrono::DateTime<Utc>,
    meta: PollResponseMeta,
) -> Result<(), AppError> {
    let PollResponseMeta {
        fallback_transport,
        last_etag,
        last_modified,
    } = meta;
    let (poll_url, provider) = fallback_transport.unwrap_or_else(|| {
        (
            source.poll_url.clone(),
            source.provider.clone().unwrap_or_default(),
        )
    });
    let provider = if provider.is_empty() {
        source.provider.clone()
    } else {
        Some(provider)
    };
    let title = feed
        .title
        .as_ref()
        .map(|t| t.content.clone())
        .unwrap_or_else(|| source.title.clone());
    let description = feed.description.as_ref().map(|d| strip_html(&d.content));
    let site_url = feed
        .links
        .iter()
        .find(|link| link.rel.as_deref() != Some("self"))
        .map(|link| link.href.clone())
        .or_else(|| source.site_url.clone());
    let image_url = feed
        .icon
        .as_ref()
        .map(|icon| icon.uri.clone())
        .or_else(|| feed.logo.as_ref().map(|logo| logo.uri.clone()))
        .or_else(|| source.image_url.clone());
    let domain = site_url
        .as_deref()
        .and_then(extract_domain)
        .or_else(|| source.domain.clone());

    ctx.feed_repo
        .update_source_details(
            source.id,
            ind_domain::SourceDetailsUpdate {
                poll_url,
                title,
                description,
                site_url,
                image_url,
                domain,
                feed_type: feed_type_for_source(source.feed_type, &feed),
                visibility: source.visibility,
                provider,
                is_resolvable: source.is_resolvable,
            },
        )
        .await?;

    let feed_language = feed.language.clone();
    let mut newest_entry_at = None;
    for entry in feed.entries {
        let guid = entry_guid(&entry);
        let entry_discovered_at = entry.published.unwrap_or(now);
        let entry_url = entry.links.first().map(|link| link.href.clone());
        // Use the same canonicalization as the save path so discovery can adopt a document that
        // already exists for the subscriber. Invalid URLs remain unlinked instead of failing poll.
        let canonical_url = entry_url.as_deref().and_then(|url| {
            ind_domain::canonicalize_url(url, &CanonicalizationConfig::default())
                .ok()
                .map(|canonical| canonical.into_string())
        });
        let title = entry
            .title
            .as_ref()
            .map(|entry_title| entry_title.content.clone())
            .unwrap_or_else(|| "Untitled Entry".into());
        let content_html = entry
            .content
            .as_ref()
            .and_then(|content| content.body.clone());
        let summary_html = entry
            .summary
            .as_ref()
            .map(|summary| summary.content.as_str());
        let excerpt = entry
            .summary
            .as_ref()
            .map(|summary| strip_html(&summary.content));
        let existing = ctx
            .feed_repo
            .find_source_entry_by_source_guid(source.id, &guid)
            .await?;
        let language = if existing
            .as_ref()
            .is_some_and(|existing| existing.language.is_some())
        {
            None
        } else {
            let content_text = content_html.as_deref().map(ind_html::html_to_text);
            ind_application::classify_search_language(
                entry.language.as_deref().or(feed_language.as_deref()),
                &[
                    &title,
                    excerpt.as_deref().unwrap_or_default(),
                    content_text.as_deref().unwrap_or_default(),
                ],
            )
            .language
        };

        let source_entry = if let Some(mut existing) = existing {
            // TASK-239: an entry created before the canonical_url column landed (or by a path that
            // did not canonicalize) has a NULL canonical_url and can never be URL-adopted.
            // Recompute it on reuse so this poll's adoption lookup below matches. A canonicalize
            // failure leaves it NULL rather than failing the poll.
            if existing.canonical_url.is_none()
                && let Some(canonical) = canonical_url.clone()
            {
                ctx.feed_repo
                    .set_source_entry_canonical_url(existing.id, &canonical)
                    .await?;
                existing.canonical_url = Some(canonical);
            }
            if existing.language.is_none()
                && let Some(detected_language) = language.as_deref()
                && ctx
                    .feed_repo
                    .set_source_entry_language_if_missing(existing.id, detected_language)
                    .await?
            {
                existing.language = Some(detected_language.to_string());
            }
            existing
        } else {
            newest_entry_at = Some(
                newest_entry_at
                    .map(|current: chrono::DateTime<Utc>| current.max(entry_discovered_at))
                    .unwrap_or(entry_discovered_at),
            );
            // Raw summary HTML (not the stripped excerpt) so feeds that put their body and lead
            // image in <description> rather than <content:encoded> still yield an image.
            let lead_image_url = ind_ingest::extract_feed_lead_image(
                &entry,
                content_html.as_deref(),
                summary_html,
                entry_url.as_deref(),
            );
            ctx.feed_repo
                .create_or_adopt_polled_source_entry(FeedSourceEntry {
                    id: FeedSourceEntryId::new(),
                    source_id: source.id,
                    guid: guid.clone(),
                    title,
                    url: entry_url,
                    canonical_url,
                    author: entry.authors.first().map(|author| author.name.clone()),
                    excerpt,
                    content_html,
                    language,
                    lead_image_url,
                    published_at: entry.published,
                    discovered_at: now,
                })
                .await?
        };

        for subscription in subscriptions {
            // The delivery upsert is idempotent and convergent: link-on-discovery sets
            // document_id when the user already prepared this entry. This is a read-only lookup;
            // it never materializes a document or enqueues a job.
            let document_id = match source_entry.canonical_url.as_deref() {
                Some(canonical) => ctx
                    .document_repo
                    .find_by_canonical_url(subscription.user_id, canonical)
                    .await?
                    .map(|doc| doc.id),
                None => ctx
                    .document_repo
                    .find_by_origin(
                        subscription.user_id,
                        DocumentOriginType::FeedSourceEntry,
                        *source_entry.id.as_uuid(),
                    )
                    .await?
                    .map(|doc| doc.id),
            };
            let autosave = if subscription.auto_save
                && (source_entry.url.is_some() || source_entry.content_html.is_some())
            {
                Some(FeedAutosaveJob {
                    feed_delivery_id: FeedDeliveryId::new(),
                    source_entry_id: source_entry.id,
                    user_id: subscription.user_id,
                    collection_id: subscription.auto_save_collection_id,
                })
            } else {
                None
            };

            let delivery_id = autosave
                .as_ref()
                .map(|job| job.feed_delivery_id)
                .unwrap_or_else(FeedDeliveryId::new);
            ctx.feed_delivery_repo
                .upsert_delivery_with_autosave(
                    FeedDelivery {
                        id: delivery_id,
                        user_id: subscription.user_id,
                        subscription_id: subscription.id,
                        source_id: source.id,
                        source_entry_id: source_entry.id,
                        document_id,
                        delivered_at: now,
                        seen_at: None,
                        dismissed_at: None,
                        hidden_at: None,
                        created_at: now,
                        updated_at: now,
                    },
                    autosave,
                    now,
                )
                .await?;
        }
    }

    let next_poll_at = next_poll_after_success(subscriptions, now, ctx.feed_poll_schedule);
    let state = PollOutcome {
        source_id: source.id,
        last_polled_at: Some(now),
        next_poll_at: Some(next_poll_at),
        last_etag,
        last_modified,
        consecutive_failures: 0,
        last_error: None,
    };

    ctx.feed_repo
        .mark_source_poll_success(source.id, state, newest_entry_at)
        .await?;

    Ok(())
}

async fn record_poll_failure(
    ctx: &FeedJobDeps,
    source: &FeedSource,
    subscriptions: &[ActiveSubscription],
    error: String,
) -> Result<(), AppError> {
    let consecutive_failures = source.consecutive_failures + 1;
    let next_poll_at = next_poll_after_failure(
        subscriptions,
        Utc::now(),
        consecutive_failures,
        ctx.feed_poll_schedule,
    );
    ctx.feed_repo
        .mark_source_poll_failure(source.id, next_poll_at, error, consecutive_failures)
        .await?;
    Ok(())
}

fn entry_guid(entry: &feed_rs::model::Entry) -> String {
    if !entry.id.is_empty() && entry.id != MISSING_ENTRY_ID_SENTINEL {
        return entry.id.clone();
    }
    if let Some(link) = entry.links.first() {
        return link.href.clone();
    }
    if let Some(title) = entry.title.as_ref() {
        return title.content.clone();
    }

    let mut content = String::new();
    if let Some(summary) = entry.summary.as_ref() {
        content.push_str("summary:");
        content.push_str(&summary.content);
    }
    if let Some(body) = entry
        .content
        .as_ref()
        .and_then(|content| content.body.as_ref())
    {
        content.push_str("|content:");
        content.push_str(body);
    }
    if let Some(published) = entry.published {
        content.push_str("|published:");
        content.push_str(&published.to_rfc3339());
    }
    for author in &entry.authors {
        content.push_str("|author:");
        content.push_str(&author.name);
    }

    format!(
        "entry-content-{}",
        uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_URL, content.as_bytes())
    )
}

fn feed_type_for_source(source_type: FeedType, feed: &feed_rs::model::Feed) -> FeedType {
    if matches!(source_type, FeedType::Twitter | FeedType::Youtube) {
        return source_type;
    }
    if feed.entries.iter().any(|entry| {
        entry.links.iter().any(|link| {
            link.media_type
                .as_ref()
                .is_some_and(|media_type: &String| media_type.starts_with("audio/"))
        })
    }) {
        FeedType::Podcast
    } else {
        match feed.feed_type {
            feed_rs::model::FeedType::Atom => FeedType::Atom,
            _ => FeedType::Rss,
        }
    }
}
