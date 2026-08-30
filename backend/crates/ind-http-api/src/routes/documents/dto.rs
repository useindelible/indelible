use chrono::{DateTime, Utc};
use ind_application::ports::DocumentReaderView;
use ind_domain::{ArchiveAssetKind, ArchiveAssetStatus, DocumentId};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::ApiError;
use crate::extract::Validate;

pub(crate) fn parse_document_id(raw: &str) -> Result<DocumentId, ApiError> {
    raw.parse().map_err(|_| ApiError::NotFound {
        entity: "Document",
        id: raw.to_string(),
    })
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct DocumentUpsertNoteBody {
    pub body: String,
}

impl Validate for DocumentUpsertNoteBody {
    fn validate(&self) -> Result<(), Vec<crate::error::FieldError>> {
        Ok(())
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DocumentNoteResponse {
    pub id: String,
    pub body: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, validator::Validate, ToSchema)]
/// `chapter_locator` and `chapter_offset` map onto `ReadingPositionSchema::anchor` / `offset`.
pub struct UpdateDocumentProgressBody {
    #[validate(range(min = 0.0, max = 100.0))]
    pub progress_percent: f32,
    pub chapter_locator: Option<String>,
    pub chapter_offset: Option<i32>,
}

impl UpdateDocumentProgressBody {
    pub(crate) fn position(&self) -> Option<ind_domain::ReadingPosition> {
        let anchor = self
            .chapter_locator
            .as_deref()
            .map(ind_domain::ReadingAnchor::from_locator);
        let offset = self.chapter_offset;
        (anchor.is_some() || offset.is_some()).then(|| ind_domain::ReadingPosition {
            anchor,
            offset,
            ..Default::default()
        })
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DocumentReprocessResponse {
    pub queued: bool,
    pub job_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after_seconds: Option<u64>,
}

/// Reader read-model. `library_entry_id`/`saved` distinguish a prepared-but-unsaved document
/// from a saved Library entry; `readable_ready` tells the client whether the readable asset has
/// landed (the on-tap render is async, so a freshly opened document polls until ready).
#[derive(Debug, Serialize, ToSchema)]
pub struct DocumentReaderResponse {
    pub document_id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub document_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub library_entry_id: Option<String>,
    pub saved: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_percent: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_progress_percent: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapter_locator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapter_offset: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<ReadingPositionSchema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub last_read_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub finished_at: Option<DateTime<Utc>>,
    pub available_assets: Vec<String>,
    pub assets: Vec<DocumentReaderAssetResponse>,
    pub readable_ready: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,
    /// Resolved short summary. Returns the stored AI summary when present,
    /// falling back to `excerpt`. `null` when neither is available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub published_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead_image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub word_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reading_time_minutes: Option<i32>,
}

impl DocumentReaderResponse {
    pub fn from_view(
        view: DocumentReaderView,
        asset_base_url: &str,
        summary: Option<String>,
    ) -> Self {
        let readable_ready = view.assets.iter().any(|asset| {
            asset.asset_kind == ArchiveAssetKind::ReadableHtml
                && asset.status == ArchiveAssetStatus::Completed
        });
        let available_assets = view
            .assets
            .iter()
            .filter(|asset| {
                asset.status == ArchiveAssetStatus::Completed && !asset.s3_key.trim().is_empty()
            })
            .map(|asset| asset.asset_kind.to_string())
            .collect();
        let assets = view
            .assets
            .iter()
            .map(DocumentReaderAssetResponse::from)
            .collect();
        let thumbnail_url = view.document.thumbnail_url.or_else(|| {
            view.assets
                .iter()
                .any(|asset| {
                    asset.asset_kind == ArchiveAssetKind::Thumbnail
                        && asset.status == ArchiveAssetStatus::Completed
                })
                .then(|| {
                    crate::routes::asset_urls::document_asset_url(
                        asset_base_url,
                        view.document.id,
                        ArchiveAssetKind::Thumbnail,
                    )
                })
        });
        let state = view.state;
        Self {
            document_id: view.document.id.to_string(),
            title: view.document.title,
            url: view.document.canonical_url.or(view.document.original_url),
            document_type: view.document.document_type.as_str().to_string(),
            library_entry_id: view.library_entry_id.map(|id| id.to_string()),
            saved: view.library_entry_id.is_some(),
            progress_percent: state.as_ref().and_then(|s| s.progress_percent),
            max_progress_percent: state.as_ref().and_then(|s| s.max_progress_percent),
            chapter_locator: state.as_ref().and_then(|s| s.chapter_locator.clone()),
            chapter_offset: state.as_ref().and_then(|s| s.chapter_offset),
            position: state
                .as_ref()
                .and_then(|s| s.scroll_position.clone())
                .and_then(|v| serde_json::from_value::<ind_domain::ReadingPosition>(v).ok())
                .map(Into::into),
            last_read_at: state.as_ref().and_then(|s| s.last_read_at),
            finished_at: state.as_ref().and_then(|s| s.finished_at),
            available_assets,
            assets,
            readable_ready,
            domain: view.document.domain,
            author: view.document.author,
            excerpt: view.document.excerpt,
            summary,
            published_at: view.document.published_at,
            language: view.document.language,
            lead_image_url: view.document.lead_image_url,
            thumbnail_url,
            word_count: view.document.word_count,
            reading_time_minutes: view.document.reading_time_minutes,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DocumentReaderAssetResponse {
    pub id: String,
    pub asset_kind: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_reason: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
}

impl From<&ind_domain::DocumentAsset> for DocumentReaderAssetResponse {
    fn from(asset: &ind_domain::DocumentAsset) -> Self {
        Self {
            id: asset.id.to_string(),
            asset_kind: asset.asset_kind.to_string(),
            content_type: asset.content_type.clone(),
            size_bytes: asset.size_bytes,
            status: status_str(asset.status).to_string(),
            failed_reason: asset.failed_reason.clone(),
            created_at: asset.created_at,
        }
    }
}

/// Document asset metadata plus an API-origin download URL for its bytes (the
/// reader fetches the readable HTML from this URL; the asset proxy behind it
/// streams or redirects depending on `asset_serving_mode`).
#[derive(Debug, Serialize, ToSchema)]
pub struct DocumentAssetResponse {
    pub id: String,
    pub object: &'static str,
    pub document_id: String,
    pub asset_kind: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub status: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub download_url: String,
}

impl DocumentAssetResponse {
    pub fn from_asset(asset: ind_domain::DocumentAsset, base_url: &str) -> Self {
        let download_url = crate::routes::asset_urls::document_asset_url(
            base_url,
            asset.document_id,
            asset.asset_kind,
        );
        Self {
            id: asset.id.to_string(),
            object: "document_asset",
            document_id: asset.document_id.to_string(),
            asset_kind: asset.asset_kind.to_string(),
            content_type: asset.content_type,
            size_bytes: asset.size_bytes,
            status: status_str(asset.status).to_string(),
            created_at: asset.created_at,
            download_url,
        }
    }
}

fn status_str(status: ArchiveAssetStatus) -> &'static str {
    match status {
        ArchiveAssetStatus::Pending => "pending",
        ArchiveAssetStatus::Completed => "completed",
        ArchiveAssetStatus::Degraded => "degraded",
        ArchiveAssetStatus::Failed => "failed",
        ArchiveAssetStatus::Unsupported => "unsupported",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ArticleTocResponseStatus {
    Ready,
    None,
    Pending,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ArticleTocEntryResponse {
    /// Document-order heading ordinal (pre-dedupe): the positional fallback
    /// when a cached article body predates anchor ids.
    pub source_heading_index: u32,
    /// Anchor id present on the heading element in the stored readable HTML.
    pub id: String,
    pub title: String,
    /// Relative outline depth (0 = top level), normalized from tag levels.
    pub depth: u8,
    /// Words in this entry's own section; clients derive minutes at 238 WPM.
    pub word_count: u32,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ArticleTocResponse {
    pub status: ArticleTocResponseStatus,
    /// True when the outline was capped at the entry limit.
    pub truncated: bool,
    /// Empty unless `status` is `ready`.
    pub entries: Vec<ArticleTocEntryResponse>,
}

/// The structural landmark a position sits on. A closed set: an unrecognised `type` is a
/// malformed body, not a validation error.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ReadingAnchorSchema {
    Page { page: i32 },
    Spine { chapter: String },
    Cue { cue: String },
    Section { section: String },
}

/// Schema-only flat representation of `ReadingAnchorSchema` for OpenAPI code generators, which
/// map an undiscriminated `oneOf` to `Any`. Mirrors the `LocatorSchemaFlat` treatment.
#[derive(Serialize, ToSchema)]
pub struct ReadingAnchorSchemaFlat {
    /// Discriminator: "page", "spine", "cue" or "section".
    #[serde(rename = "type")]
    pub anchor_type: String,
    /// page: 1-based page number.
    pub page: Option<i32>,
    /// spine: EPUB spine key.
    pub chapter: Option<String>,
    /// cue: transcript cue id.
    pub cue: Option<String>,
    /// section: article, email or tweet section id.
    pub section: Option<String>,
}

impl From<ReadingAnchorSchema> for ind_domain::ReadingAnchor {
    fn from(v: ReadingAnchorSchema) -> Self {
        match v {
            ReadingAnchorSchema::Page { page } => Self::Page { page },
            ReadingAnchorSchema::Spine { chapter } => Self::Spine { chapter },
            ReadingAnchorSchema::Cue { cue } => Self::Cue { cue },
            ReadingAnchorSchema::Section { section } => Self::Section { section },
        }
    }
}

impl From<ind_domain::ReadingAnchor> for ReadingAnchorSchema {
    fn from(v: ind_domain::ReadingAnchor) -> Self {
        use ind_domain::ReadingAnchor as A;
        match v {
            A::Page { page } => Self::Page { page },
            A::Spine { chapter } => Self::Spine { chapter },
            A::Cue { cue } => Self::Cue { cue },
            A::Section { section } => Self::Section { section },
        }
    }
}

/// Where the reader stopped, in coordinates every artifact type can speak. The fields are
/// independent, not a union keyed on document type: an article sends `anchor` + `offset`, a
/// PDF `anchor` + `fraction`, a video or podcast adds `seconds`.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default)]
pub struct ReadingPositionSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<ReadingAnchorSchemaFlat>)]
    pub anchor: Option<ReadingAnchorSchema>,
    /// Units into `anchor` — characters for text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<i32>,
    /// `0..=1` through the whole artifact.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fraction: Option<f64>,
    /// Media playhead in seconds, for video and podcast.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seconds: Option<f64>,
}

impl From<ReadingPositionSchema> for ind_domain::ReadingPosition {
    fn from(v: ReadingPositionSchema) -> Self {
        Self {
            anchor: v.anchor.map(Into::into),
            offset: v.offset,
            fraction: v.fraction,
            seconds: v.seconds,
        }
    }
}

impl From<ind_domain::ReadingPosition> for ReadingPositionSchema {
    fn from(v: ind_domain::ReadingPosition) -> Self {
        Self {
            anchor: v.anchor.map(Into::into),
            offset: v.offset,
            fraction: v.fraction,
            seconds: v.seconds,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ReadingEventBody {
    /// `rev_` prefixed, generated by the client so retries are idempotent.
    #[schema(value_type = String)]
    pub id: ind_domain::ReadingEventId,
    /// Required when the batch carries a `client_id`; the server allocates it otherwise.
    #[serde(default)]
    pub origin_seq: Option<i64>,
    #[schema(value_type = String, example = "progress")]
    pub kind: ind_domain::ReadingEventKind,
    /// Hundredths of a percent: 42.37% is 4237.
    #[serde(default)]
    pub progress_basis_points: Option<i32>,
    /// Why the event happened. Defaults to `reader` telemetry.
    #[serde(default)]
    #[schema(value_type = Option<String>, example = "reader")]
    pub cause: Option<ind_domain::ReadingCause>,
    /// Groups one continuous sitting; omit if the client does not track sessions.
    #[serde(default)]
    #[schema(value_type = Option<String>)]
    pub session_id: Option<uuid::Uuid>,
    /// Which pass through the document. Increment to start a reread; a higher attempt
    /// outranks a lower one regardless of arrival time.
    #[serde(default)]
    pub attempt: Option<i16>,
    #[serde(default)]
    pub position: Option<ReadingPositionSchema>,
    /// Which readable representation `position` refers to.
    #[serde(default)]
    #[schema(value_type = Option<String>, example = "epub")]
    pub asset_kind: Option<ind_domain::ArchiveAssetKind>,
    #[serde(default)]
    pub position_version: Option<i16>,
    #[serde(default)]
    pub active_ms: Option<i32>,
    #[schema(value_type = String, format = DateTime)]
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AppendReadingEventsBody {
    /// Stable per-install id, `cli_` prefixed. Omitted by callers with no device
    /// identity, who are attributed from their credential instead.
    #[serde(default)]
    #[schema(value_type = Option<String>)]
    pub client_id: Option<ind_domain::ClientId>,
    pub events: Vec<ReadingEventBody>,
}

pub const MAX_READING_EVENTS_PER_BATCH: usize = 200;

impl Validate for AppendReadingEventsBody {
    fn validate(&self) -> Result<(), Vec<crate::error::FieldError>> {
        use ind_domain::ReadingEventKind as Kind;
        let mut errors = Vec::new();
        let mut push = |field: String, message: &str| {
            errors.push(crate::error::FieldError {
                field,
                message: message.into(),
            });
        };
        if self.events.is_empty() || self.events.len() > MAX_READING_EVENTS_PER_BATCH {
            push("events".into(), "must contain between 1 and 200 events");
        }
        let device_batch = self.client_id.is_some();
        let mut last_seq: Option<i64> = None;
        for (i, e) in self.events.iter().enumerate() {
            let f = |name: &str| format!("events[{i}].{name}");
            match (device_batch, e.origin_seq) {
                (true, None) => push(f("origin_seq"), "is required when client_id is set"),
                (false, Some(_)) => push(
                    f("origin_seq"),
                    "is server-assigned when client_id is absent, so it must be omitted",
                ),
                (true, Some(seq)) => {
                    if seq < 0 {
                        push(f("origin_seq"), "must not be negative");
                    }
                    if last_seq.is_some_and(|prev| seq <= prev) {
                        push(f("origin_seq"), "must strictly increase within a batch");
                    }
                    last_seq = Some(seq);
                }
                (false, None) => {}
            }
            let has_payload =
                e.progress_basis_points.is_some() || e.position.is_some() || e.active_ms.is_some();
            match e.kind {
                Kind::Opened if has_payload => {
                    push(f("kind"), "opened events carry no progress fields")
                }
                // `finished` deliberately carries no progress requirement: a reader who stops
                // at 94% because the rest is appendices has finished the book, and forcing a
                // fake 100 would write that falsehood into the log permanently.
                Kind::Progress if e.progress_basis_points.is_none() => push(
                    f("progress_basis_points"),
                    "is required for progress events",
                ),
                _ => {}
            }
            if e.progress_basis_points
                .is_some_and(|bp| !(0..=10_000).contains(&bp))
            {
                push(f("progress_basis_points"), "must be within 0..=10000");
            }
            if e.attempt.is_some_and(|a| a < 1) {
                push(f("attempt"), "must be 1 or greater");
            }
            if e.position_version.is_some_and(|v| v < 1) {
                push(f("position_version"), "must be 1 or greater");
            }
            if e.active_ms.is_some_and(|v| v < 0) {
                push(f("active_ms"), "must not be negative");
            }
            if let Some(p) = &e.position {
                match &p.anchor {
                    Some(ReadingAnchorSchema::Page { page }) if *page < 1 => {
                        push(f("position.anchor.page"), "must be 1 or greater")
                    }
                    Some(ReadingAnchorSchema::Spine { chapter }) if chapter.is_empty() => {
                        push(f("position.anchor.chapter"), "must not be empty")
                    }
                    Some(ReadingAnchorSchema::Cue { cue }) if cue.is_empty() => {
                        push(f("position.anchor.cue"), "must not be empty")
                    }
                    Some(ReadingAnchorSchema::Section { section }) if section.is_empty() => {
                        push(f("position.anchor.section"), "must not be empty")
                    }
                    _ => {}
                }
                if p.offset.is_some_and(|v| v < 0) {
                    push(f("position.offset"), "must not be negative");
                }
                if p.fraction.is_some_and(|v| !(0.0..=1.0).contains(&v)) {
                    push(f("position.fraction"), "must be within 0..=1");
                }
                if p.seconds.is_some_and(|v| !v.is_finite() || v < 0.0) {
                    push(f("position.seconds"), "must be a finite value of 0 or more");
                }
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl ReadingEventBody {
    /// Validation has already bounded `progress_basis_points`, so the conversion cannot fail.
    pub(crate) fn into_domain(
        self,
        origin: &ind_domain::EventOrigin,
    ) -> ind_domain::NewReadingEvent {
        let progress = self
            .progress_basis_points
            .and_then(|bp| ind_domain::BasisPoints::new(bp).ok());
        ind_domain::NewReadingEvent {
            id: self.id,
            origin: origin.clone(),
            origin_seq: self.origin_seq,
            kind: self.kind,
            cause: self.cause.unwrap_or_default(),
            session_id: self.session_id,
            attempt: self.attempt.unwrap_or(1),
            progress,
            position: self.position.map(Into::into),
            asset_kind: self.asset_kind,
            position_version: self
                .position_version
                .unwrap_or(ind_domain::NewReadingEvent::CURRENT_POSITION_VERSION),
            active_ms: self.active_ms,
            recorded_at: self.recorded_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AppendReadingEventsResponse {
    pub accepted: usize,
    pub replayed: usize,
}
