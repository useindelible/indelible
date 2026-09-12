//! Append-only reading log. `user_document_state` is a projection of these events.

use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::string_enum::impl_string_enum;
use uuid::Uuid;

use crate::{ApiTokenId, ArchiveAssetKind, ClientId, ClientType, DomainError, ReadingEventId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadingEventKind {
    Opened,
    Progress,
    Finished,
}

impl ReadingEventKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Opened => "opened",
            Self::Progress => "progress",
            Self::Finished => "finished",
        }
    }
}

impl FromStr for ReadingEventKind {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "opened" => Ok(Self::Opened),
            "progress" => Ok(Self::Progress),
            "finished" => Ok(Self::Finished),
            other => Err(DomainError::Validation {
                field: "kind".into(),
                message: format!("unknown reading event kind `{other}`"),
            }),
        }
    }
}

/// The one structural landmark a position sits on. Every artifact has exactly one, so this is
/// a closed set rather than a coordinate: an unrecognised kind is refused, not stored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ReadingAnchor {
    Page { page: i32 },
    Spine { chapter: String },
    Cue { cue: String },
    Section { section: String },
}

impl ReadingAnchor {
    /// Rendering for the pre-existing `user_document_state.chapter_locator` column, which the
    /// web reader still reads. `from_locator` round-trips this exactly.
    pub fn locator(&self) -> String {
        match self {
            Self::Page { page } => format!("page:{page}"),
            Self::Spine { chapter } => chapter.clone(),
            Self::Cue { cue } => format!("cue:{cue}"),
            Self::Section { section } => section.clone(),
        }
    }

    /// Legacy `PATCH /progress` locators carry no kind. The deployed web reader sends only two
    /// shapes — `page:N` for PDF and an EPUB TOC entry id — so anything unprefixed is a spine key.
    pub fn from_locator(value: &str) -> Self {
        if let Some(page) = value
            .strip_prefix("page:")
            .and_then(|n| n.parse::<i32>().ok())
            .filter(|page| *page >= 1)
        {
            return Self::Page { page };
        }
        Self::Spine {
            chapter: value.to_owned(),
        }
    }
}

/// Where the reader stopped, in coordinates every artifact type can speak. Not a union keyed
/// on document type: the fields are independent and more than one can hold at once — a video
/// with a transcript has both a playhead and a cue. Stored verbatim in
/// `user_document_state.scroll_position`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ReadingPosition {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<ReadingAnchor>,
    /// Units into `anchor` — characters for text; unused where the anchor is atomic.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<i32>,
    /// `0..=1` through the whole artifact. The one coordinate every type can supply.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fraction: Option<f64>,
    /// Media playhead in seconds, for video and podcast.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seconds: Option<f64>,
}

impl ReadingPosition {
    pub fn is_empty(&self) -> bool {
        self.anchor.is_none()
            && self.offset.is_none()
            && self.fraction.is_none()
            && self.seconds.is_none()
    }
}

/// Third-party system a reading event was imported from. Distinct from `IntegrationProvider`
/// (long-lived, config-bearing sync connections) — this only names the source of a one-shot
/// background import that has no request context to attribute a device or session to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrationSource {
    Readwise,
}

impl_string_enum!(IntegrationSource, "integration source", {
    Readwise => "readwise",
});

/// Who wrote a reading event. A mobile install identifies itself and keeps its own
/// sequence; other callers are attributed from the credential their session carries;
/// a background import is attributed to the integration that produced it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventOrigin {
    Device(ClientId),
    Surface(ClientType),
    ApiToken(ApiTokenId),
    Integration(IntegrationSource),
}

impl fmt::Display for EventOrigin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Device(id) => write!(f, "{id}"),
            Self::Surface(client_type) => write!(f, "surface:{client_type}"),
            Self::ApiToken(id) => write!(f, "{id}"),
            Self::Integration(source) => write!(f, "integration:{source}"),
        }
    }
}

impl FromStr for EventOrigin {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if let Some(source) = value.strip_prefix("integration:") {
            return source
                .parse::<IntegrationSource>()
                .map(Self::Integration)
                .map_err(|_| DomainError::Validation {
                    field: "origin".into(),
                    message: format!("unrecognised event origin `{value}`"),
                });
        }
        if let Some(client_type) = value.strip_prefix("surface:") {
            return client_type
                .parse::<ClientType>()
                .map(Self::Surface)
                .map_err(|_| DomainError::Validation {
                    field: "origin".into(),
                    message: format!("unrecognised event origin `{value}`"),
                });
        }
        if let Ok(id) = value.parse::<ClientId>() {
            return Ok(Self::Device(id));
        }
        if let Ok(id) = value.parse::<ApiTokenId>() {
            return Ok(Self::ApiToken(id));
        }
        Err(DomainError::Validation {
            field: "origin".into(),
            message: format!("unrecognised event origin `{value}`"),
        })
    }
}

/// Why an event exists. Ordering is by time; this is intent, which time cannot express — a
/// deliberate correction should outrank a stale device batch that merely arrived later.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadingCause {
    #[default]
    Reader,
    Manual,
    Import,
    Sync,
    Repair,
}

impl ReadingCause {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reader => "reader",
            Self::Manual => "manual",
            Self::Import => "import",
            Self::Sync => "sync",
            Self::Repair => "repair",
        }
    }
}

impl fmt::Display for ReadingCause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ReadingCause {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "reader" => Ok(Self::Reader),
            "manual" => Ok(Self::Manual),
            "import" => Ok(Self::Import),
            "sync" => Ok(Self::Sync),
            "repair" => Ok(Self::Repair),
            other => Err(DomainError::Validation {
                field: "cause".into(),
                message: format!("unknown reading cause `{other}`"),
            }),
        }
    }
}

/// Progress in hundredths of a percent, so 42.37% is 4237. Whole percent loses eight pages of
/// an 800-page book before it moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BasisPoints(i32);

impl BasisPoints {
    pub const MAX: i32 = 10_000;

    pub fn new(value: i32) -> Result<Self, DomainError> {
        if (0..=Self::MAX).contains(&value) {
            Ok(Self(value))
        } else {
            Err(DomainError::Validation {
                field: "progress_basis_points".into(),
                message: format!("must be between 0 and {}", Self::MAX),
            })
        }
    }

    pub fn from_percent(percent: i32) -> Result<Self, DomainError> {
        Self::new(percent.saturating_mul(100))
    }

    pub fn get(self) -> i32 {
        self.0
    }

    /// Whole percent, truncated, for the `user_document_state` columns the web reader reads.
    pub fn to_percent(self) -> i32 {
        self.0 / 100
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewReadingEvent {
    pub id: ReadingEventId,
    pub origin: EventOrigin,
    /// `None` for a caller with no device counter; the server allocates from
    /// `reading_events_surface_seq` so PATCH and POST share one ordering authority.
    pub origin_seq: Option<i64>,
    pub kind: ReadingEventKind,
    pub cause: ReadingCause,
    /// Groups one continuous sitting. `None` from a client that does not track sessions.
    pub session_id: Option<Uuid>,
    /// Which pass through the document this is. A higher attempt always outranks a lower one,
    /// which is what separates a deliberate reread from a stale device replaying old progress.
    pub attempt: i16,
    pub progress: Option<BasisPoints>,
    pub position: Option<ReadingPosition>,
    /// Which readable representation `position` refers to. Captured for a later per-format
    /// projection; the current projection keeps one position per document.
    pub asset_kind: Option<ArchiveAssetKind>,
    pub position_version: i16,
    /// Active reading time accrued **since the previous event in this session**, never the
    /// session total, so `SUM(active_ms)` is meaningful. Paused on background and inactivity.
    pub active_ms: Option<i32>,
    pub recorded_at: DateTime<Utc>,
}

impl NewReadingEvent {
    pub const CURRENT_POSITION_VERSION: i16 = 1;
}

impl NewReadingEvent {
    pub fn anchor(&self) -> Option<&ReadingAnchor> {
        self.position.as_ref().and_then(|p| p.anchor.as_ref())
    }

    pub fn offset(&self) -> Option<i32> {
        self.position.as_ref().and_then(|p| p.offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_round_trips_through_its_wire_name() {
        for (kind, name) in [
            (ReadingEventKind::Opened, "opened"),
            (ReadingEventKind::Progress, "progress"),
            (ReadingEventKind::Finished, "finished"),
        ] {
            assert_eq!(kind.as_str(), name);
            assert_eq!(name.parse::<ReadingEventKind>().unwrap(), kind);
        }
        assert!("paused".parse::<ReadingEventKind>().is_err());
    }

    #[test]
    fn position_serializes_only_present_fields() {
        let position = ReadingPosition {
            anchor: Some(ReadingAnchor::Page { page: 12 }),
            fraction: Some(0.31),
            ..ReadingPosition::default()
        };
        assert_eq!(
            serde_json::to_value(&position).unwrap(),
            serde_json::json!({"anchor": {"type": "page", "page": 12}, "fraction": 0.31})
        );
        assert!(ReadingPosition::default().is_empty());
        assert!(!position.is_empty());
    }

    #[test]
    fn every_artifact_type_has_a_position_it_can_express() {
        let article = ReadingPosition {
            anchor: Some(ReadingAnchor::Section {
                section: "sec-3".into(),
            }),
            offset: Some(4210),
            fraction: Some(0.44),
            seconds: None,
        };
        let epub = ReadingPosition {
            anchor: Some(ReadingAnchor::Spine {
                chapter: "OEBPS/ch09.xhtml".into(),
            }),
            offset: Some(1180),
            fraction: Some(0.62),
            seconds: None,
        };
        let pdf = ReadingPosition {
            anchor: Some(ReadingAnchor::Page { page: 12 }),
            fraction: Some(0.31),
            ..ReadingPosition::default()
        };
        let video = ReadingPosition {
            anchor: Some(ReadingAnchor::Cue { cue: "0142".into() }),
            fraction: Some(0.63),
            seconds: Some(757.4),
            offset: None,
        };
        let podcast = ReadingPosition {
            fraction: Some(0.18),
            seconds: Some(1284.0),
            ..ReadingPosition::default()
        };
        for position in [&article, &epub, &pdf, &video, &podcast] {
            assert!(!position.is_empty());
            let json = serde_json::to_value(position).unwrap();
            assert_eq!(
                serde_json::from_value::<ReadingPosition>(json).unwrap(),
                *position
            );
        }
        assert_eq!(video.seconds, Some(757.4));
        assert_eq!(podcast.seconds, Some(1284.0));
    }

    #[test]
    fn anchor_round_trips_through_the_legacy_locator_string() {
        for anchor in [
            ReadingAnchor::Page { page: 12 },
            ReadingAnchor::Cue { cue: "0142".into() },
            ReadingAnchor::Spine {
                chapter: "OEBPS/ch09.xhtml".into(),
            },
            ReadingAnchor::Section {
                section: "sec-3".into(),
            },
        ] {
            let rendered = anchor.locator();
            assert_eq!(
                ReadingAnchor::from_locator(&rendered).locator(),
                rendered,
                "{anchor:?} must survive the legacy column unchanged"
            );
        }
        assert_eq!(
            ReadingAnchor::from_locator("page:12"),
            ReadingAnchor::Page { page: 12 }
        );
        for not_a_page in ["page:0", "page:-3", "page:abc", "pge:12"] {
            assert!(
                matches!(
                    ReadingAnchor::from_locator(not_a_page),
                    ReadingAnchor::Spine { .. }
                ),
                "{not_a_page} must not become a page"
            );
        }
    }

    #[test]
    fn an_unrecognised_anchor_kind_is_refused() {
        for wire in [
            serde_json::json!({"type": "frame", "frame": "94"}),
            serde_json::json!({"type": "other", "kind": "frame", "value": "94"}),
            serde_json::json!({"page": 12}),
        ] {
            assert!(serde_json::from_value::<ReadingAnchor>(wire).is_err());
        }
    }

    #[test]
    fn cause_wire_strings_match_serde() {
        for cause in [
            ReadingCause::Reader,
            ReadingCause::Manual,
            ReadingCause::Import,
            ReadingCause::Sync,
            ReadingCause::Repair,
        ] {
            let serde_name = serde_json::to_value(cause).unwrap();
            assert_eq!(serde_name, serde_json::json!(cause.as_str()));
            assert_eq!(cause.as_str().parse::<ReadingCause>().unwrap(), cause);
        }
        assert_eq!(ReadingCause::default(), ReadingCause::Reader);
        assert!("guess".parse::<ReadingCause>().is_err());
    }

    #[test]
    fn basis_points_bound_and_convert() {
        assert_eq!(BasisPoints::new(4237).unwrap().to_percent(), 42);
        assert_eq!(BasisPoints::from_percent(100).unwrap().get(), 10_000);
        assert!(BasisPoints::new(-1).is_err());
        assert!(BasisPoints::new(10_001).is_err());
    }

    #[test]
    fn ids_use_their_prefixes() {
        let event = ReadingEventId::new();
        assert!(event.to_string().starts_with("rev_"));
        assert_eq!(event.to_string().parse::<ReadingEventId>().unwrap(), event);
        let client = ClientId::new();
        assert!(client.to_string().starts_with("cli_"));
        assert!("legacy".parse::<ClientId>().is_err());
    }

    #[test]
    fn event_origin_round_trips_through_display_and_from_str() {
        let device = EventOrigin::Device(ClientId::new());
        assert_eq!(device.to_string().parse::<EventOrigin>().unwrap(), device);

        for client_type in [
            ClientType::Web,
            ClientType::Ios,
            ClientType::Android,
            ClientType::Desktop,
            ClientType::Extension,
            ClientType::Cli,
        ] {
            let surface = EventOrigin::Surface(client_type);
            assert_eq!(surface.to_string().parse::<EventOrigin>().unwrap(), surface);
        }

        let api_token = EventOrigin::ApiToken(ApiTokenId::new());
        assert_eq!(
            api_token.to_string().parse::<EventOrigin>().unwrap(),
            api_token
        );

        let integration = EventOrigin::Integration(IntegrationSource::Readwise);
        assert_eq!(
            integration.to_string().parse::<EventOrigin>().unwrap(),
            integration
        );
        assert_eq!(integration.to_string(), "integration:readwise");
    }

    #[test]
    fn event_origin_rejects_unrecognised_strings() {
        let err = "not-an-origin".parse::<EventOrigin>().unwrap_err();
        assert!(matches!(err, DomainError::Validation { field, .. } if field == "origin"));

        let err = "surface:pager".parse::<EventOrigin>().unwrap_err();
        assert!(matches!(err, DomainError::Validation { field, .. } if field == "origin"));
    }

    #[test]
    fn surface_and_device_origins_never_collide() {
        let surface: EventOrigin = "surface:cli".parse().unwrap();
        let device = EventOrigin::Device(ClientId::new());
        assert_ne!(surface.to_string(), device.to_string());
        assert!(device.to_string().starts_with("cli_"));
        assert_eq!(surface, EventOrigin::Surface(ClientType::Cli));
    }

    #[test]
    fn integration_origins_never_cross_parse_with_other_forms() {
        let integration: EventOrigin = "integration:readwise".parse().unwrap();
        let surface: EventOrigin = "surface:web".parse().unwrap();
        let device = EventOrigin::Device(ClientId::new());
        let api_token = EventOrigin::ApiToken(ApiTokenId::new());

        assert_eq!(
            integration,
            EventOrigin::Integration(IntegrationSource::Readwise)
        );
        assert_ne!(integration.to_string(), surface.to_string());
        assert_ne!(integration.to_string(), device.to_string());
        assert_ne!(integration.to_string(), api_token.to_string());

        assert!("integration:notion".parse::<EventOrigin>().is_err());
        assert!("readwise".parse::<EventOrigin>().is_err());
    }
}
