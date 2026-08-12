//! Document-keyed YouTube ingest for transcript-enriched reader content.

use std::time::Duration as StdDuration;

use bytes::Bytes;
use ind_application::error::AppError;
use ind_application::repos::document::DocumentYoutubeEnrichment;
use ind_domain::{
    ArchiveAssetKind, ArchiveAssetStatus, DomainError, NewDocumentAsset, YoutubeIngestDocumentJob,
};

use super::html::{BuildReaderHtmlInput, build_reader_html};
use super::player::{
    DEFAULT_YOUTUBE_BASE, IOS_USER_AGENT, direct_timedtext_url, fetch_player_response,
    pick_caption_track_url, pick_largest_thumbnail,
};
use super::transcript::fetch_transcript;
use super::truncate_chars;
use crate::context::CaptureJobDeps;
use crate::jobs::ai::enqueue_document_embed_if_engaged;
use crate::jobs::search::enqueue_search_reindex_document;

pub async fn handle_youtube_ingest_document(
    ctx: &CaptureJobDeps,
    job: YoutubeIngestDocumentJob,
) -> Result<(), AppError> {
    let video_id =
        ind_application::dispatch::extract_youtube_video_id(&job.url).ok_or_else(|| {
            AppError::Domain(DomainError::InvariantViolation {
                message: format!(
                    "youtube_ingest: could not extract video ID from {}",
                    job.url
                ),
            })
        })?;

    let storage = ctx.object_storage.as_ref().ok_or_else(|| {
        AppError::Repository("object storage is not configured for the worker".into())
    })?;

    ctx.document_repo
        .find_by_id(job.user_id, job.document_id)
        .await?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "Document",
                id: job.document_id.to_string(),
            })
        })?;

    // Caption URLs come from YouTube's player response (attacker-adjacent third-party data);
    // fetch them through the SSRF egress guard so a poisoned/redirected URL cannot reach
    // internal or metadata IPs.
    let http = ind_egress::build_guarded_client(
        ind_egress::GuardedClientOptions::new(
            ind_egress::UrlRules::ingest(),
            ctx.egress_policy.clone(),
        )
        .user_agent(IOS_USER_AGENT)
        .request_timeout(StdDuration::from_secs(30)),
    )
    .map_err(|e| AppError::Repository(Box::new(e)))?;

    let base_url = ctx
        .youtube_player_base_url
        .as_deref()
        .unwrap_or(DEFAULT_YOUTUBE_BASE);
    let player = fetch_player_response(&http, base_url, &video_id).await?;
    if player.video_details.is_none() && player.is_terminally_unavailable() {
        mark_unavailable_video(ctx, job.document_id).await?;
        return Err(AppError::Domain(DomainError::NotFound {
            entity: "YouTubeVideo",
            id: video_id,
        }));
    }
    let video_details = player.video_details.ok_or_else(|| {
        let (status, reason) = player
            .playability_status
            .as_ref()
            .map(|value| (value.status.as_deref(), value.reason.as_deref()))
            .unwrap_or((None, None));
        AppError::ExternalService {
            service: "youtube".into(),
            message: format!(
                "player response missing videoDetails for {video_id} (status: {}, reason: {})",
                status.unwrap_or("unknown"),
                reason.unwrap_or("unknown")
            ),
        }
    })?;

    let title = video_details.title.unwrap_or_default();
    let channel_name = video_details.author.unwrap_or_default();
    let description = video_details.short_description.unwrap_or_default();
    let view_count = video_details.view_count;
    let duration_seconds = video_details
        .length_seconds
        .as_deref()
        .and_then(|s| s.parse::<i32>().ok());
    let thumbnail_url = video_details
        .thumbnail
        .and_then(|t| t.thumbnails)
        .and_then(pick_largest_thumbnail);

    let caption_url = player
        .captions
        .and_then(|c| c.player_captions_tracklist_renderer)
        .and_then(|r| r.caption_tracks)
        .and_then(|tracks| pick_caption_track_url(&tracks))
        .unwrap_or_else(|| direct_timedtext_url(&video_id));

    let segments = fetch_transcript(&http, &caption_url).await;

    let html = build_reader_html(BuildReaderHtmlInput {
        video_id: &video_id,
        description: &description,
        channel_name: &channel_name,
        view_count: view_count.as_deref(),
        duration_seconds,
        segments: &segments,
    });

    let s3_key = format!(
        "documents/{}/{}/readable.html",
        job.user_id.into_uuid(),
        job.document_id.into_uuid()
    );
    let upload = storage
        .upload(&s3_key, "text/html", Bytes::from(html.into_bytes()))
        .await?;

    ctx.document_asset_repo
        .upsert_document_asset(NewDocumentAsset {
            document_id: job.document_id,
            asset_kind: ArchiveAssetKind::ReadableHtml,
            s3_key,
            s3_bucket: upload.bucket,
            content_type: "text/html".into(),
            size_bytes: upload.size_bytes,
            status: ArchiveAssetStatus::Completed,
            failed_reason: None,
        })
        .await?;

    let transcript_text = segments
        .iter()
        .map(|segment| segment.text.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");
    if transcript_text.trim().is_empty() {
        ctx.document_asset_repo
            .upsert_document_asset(NewDocumentAsset {
                document_id: job.document_id,
                asset_kind: ArchiveAssetKind::ExtractedText,
                s3_key: String::new(),
                s3_bucket: String::new(),
                content_type: "text/plain".into(),
                size_bytes: 0,
                status: ArchiveAssetStatus::Failed,
                failed_reason: Some("YouTube transcript unavailable or empty".into()),
            })
            .await?;
    } else {
        let transcript_key = format!(
            "documents/{}/{}/extracted.txt",
            job.user_id.into_uuid(),
            job.document_id.into_uuid()
        );
        let transcript_upload = storage
            .upload(
                &transcript_key,
                "text/plain",
                Bytes::from(transcript_text.into_bytes()),
            )
            .await?;
        ctx.document_asset_repo
            .upsert_document_asset(NewDocumentAsset {
                document_id: job.document_id,
                asset_kind: ArchiveAssetKind::ExtractedText,
                s3_key: transcript_key,
                s3_bucket: transcript_upload.bucket,
                content_type: "text/plain".into(),
                size_bytes: transcript_upload.size_bytes,
                status: ArchiveAssetStatus::Completed,
                failed_reason: None,
            })
            .await?;
    }

    let excerpt = (!description.is_empty()).then(|| truncate_chars(&description, 200));
    let title_value = (!title.is_empty()).then_some(title);

    ctx.document_repo
        .apply_youtube_enrichment(
            job.user_id,
            job.document_id,
            DocumentYoutubeEnrichment {
                title: title_value,
                excerpt,
                lead_image_url: thumbnail_url,
                duration_seconds,
                youtube_channel_name: (!channel_name.is_empty()).then(|| channel_name.clone()),
            },
        )
        .await?;

    enqueue_search_reindex_document(ctx, job.document_id).await?;
    enqueue_document_embed_if_engaged(ctx, job.user_id, job.document_id).await?;
    Ok(())
}

async fn mark_unavailable_video(
    ctx: &CaptureJobDeps,
    document_id: ind_domain::DocumentId,
) -> Result<(), AppError> {
    ctx.document_asset_repo
        .upsert_document_asset(NewDocumentAsset {
            document_id,
            asset_kind: ArchiveAssetKind::ReadableHtml,
            s3_key: String::new(),
            s3_bucket: ctx.feed.s3_bucket.clone(),
            content_type: "text/html".into(),
            size_bytes: 0,
            status: ArchiveAssetStatus::Failed,
            failed_reason: Some("This YouTube video is unavailable, private, or deleted.".into()),
        })
        .await?;
    Ok(())
}
