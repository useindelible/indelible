use ind_application::error::AppError;
use serde::Deserialize;

pub(super) const IOS_USER_AGENT: &str =
    "com.google.ios.youtube/20.10.4 (iPhone16,2; U; CPU iOS 18_3_2 like Mac OS X;)";

pub(super) const DEFAULT_YOUTUBE_BASE: &str = "https://www.youtube.com";

#[derive(Deserialize)]
pub(super) struct PlayerResponse {
    #[serde(rename = "videoDetails")]
    pub(super) video_details: Option<VideoDetails>,
    #[serde(rename = "playabilityStatus")]
    pub(super) playability_status: Option<PlayabilityStatus>,
    pub(super) captions: Option<Captions>,
}

impl PlayerResponse {
    pub(super) fn is_terminally_unavailable(&self) -> bool {
        self.playability_status
            .as_ref()
            .and_then(|value| value.status.as_deref())
            .is_some_and(|status| matches!(status, "ERROR" | "UNPLAYABLE" | "LOGIN_REQUIRED"))
    }
}

#[derive(Deserialize)]
pub(super) struct PlayabilityStatus {
    pub(super) status: Option<String>,
    pub(super) reason: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct VideoDetails {
    pub(super) title: Option<String>,
    pub(super) author: Option<String>,
    #[serde(rename = "shortDescription")]
    pub(super) short_description: Option<String>,
    #[serde(rename = "lengthSeconds")]
    pub(super) length_seconds: Option<String>,
    #[serde(rename = "viewCount")]
    pub(super) view_count: Option<String>,
    pub(super) thumbnail: Option<Thumbnail>,
}

#[derive(Deserialize)]
pub(super) struct Thumbnail {
    pub(super) thumbnails: Option<Vec<ThumbnailEntry>>,
}

#[derive(Deserialize)]
pub(super) struct ThumbnailEntry {
    pub(super) url: Option<String>,
    pub(super) width: Option<u32>,
}

#[derive(Deserialize)]
pub(super) struct Captions {
    #[serde(rename = "playerCaptionsTracklistRenderer")]
    pub(super) player_captions_tracklist_renderer: Option<CaptionTracklistRenderer>,
}

#[derive(Deserialize)]
pub(super) struct CaptionTracklistRenderer {
    #[serde(rename = "captionTracks")]
    pub(super) caption_tracks: Option<Vec<CaptionTrack>>,
}

#[derive(Deserialize)]
pub(super) struct CaptionTrack {
    #[serde(rename = "baseUrl")]
    pub(super) base_url: Option<String>,
    #[serde(rename = "vssId")]
    pub(super) vss_id: Option<String>,
}

pub(super) async fn fetch_player_response(
    http: &ind_egress::GuardedHttpClient,
    base_url: &str,
    video_id: &str,
) -> Result<PlayerResponse, AppError> {
    let body = serde_json::json!({
        "context": {
            "client": {
                "clientName": "IOS",
                "clientVersion": "20.10.4",
                "deviceMake": "Apple",
                "deviceModel": "iPhone16,2",
                "userAgent": IOS_USER_AGENT,
                "osName": "iPhone",
                "osVersion": "18.3.2.22D82",
                "hl": "en"
            }
        },
        "videoId": video_id,
        "contentCheckOk": true,
        "racyCheckOk": true,
    });

    let resp = http
        .post(&format!(
            "{}/youtubei/v1/player?prettyPrint=false",
            base_url.trim_end_matches('/')
        ))
        .map_err(|error| youtube_error(error.to_string()))?
        .header("Content-Type", "application/json")
        .header("X-Youtube-Client-Name", "5")
        .header("X-Youtube-Client-Version", "20.10.4")
        .header("Origin", "https://www.youtube.com")
        .json(&body)
        .send()
        .await
        .map_err(|error| youtube_error(error.to_string()))?;

    if !resp.status().is_success() {
        return Err(youtube_error(format!(
            "youtube player API returned {} for {}",
            resp.status(),
            video_id
        )));
    }

    resp.json::<PlayerResponse>()
        .await
        .map_err(|error| youtube_error(error.to_string()))
}

fn youtube_error(message: String) -> AppError {
    AppError::ExternalService {
        service: "youtube".into(),
        message,
    }
}

pub(super) fn pick_largest_thumbnail(mut thumbnails: Vec<ThumbnailEntry>) -> Option<String> {
    thumbnails.sort_by(|a, b| b.width.unwrap_or(0).cmp(&a.width.unwrap_or(0)));
    thumbnails.into_iter().find_map(|t| t.url)
}

pub(super) fn pick_caption_track_url(tracks: &[CaptionTrack]) -> Option<String> {
    let english = tracks.iter().find(|t| {
        t.vss_id
            .as_deref()
            .is_some_and(|v| v.contains(".en") || v.contains("a.en"))
    });
    english
        .or_else(|| tracks.first())
        .and_then(|t| t.base_url.clone())
}

pub(super) fn direct_timedtext_url(video_id: &str) -> String {
    format!(
        "https://www.youtube.com/api/timedtext?v={}&lang=en&fmt=xml",
        urlencoding(video_id)
    )
}

fn urlencoding(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}
