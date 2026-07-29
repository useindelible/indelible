use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HomeWidgetKind {
    ContinueReading,
    DailyReview,
    RecentlyAdded,
    QuickReads,
    PinnedCollections,
    ReadingStats,
    FeedDigest,
}

impl HomeWidgetKind {
    pub const ALL: &'static [HomeWidgetKind] = &[
        HomeWidgetKind::ContinueReading,
        HomeWidgetKind::DailyReview,
        HomeWidgetKind::RecentlyAdded,
        HomeWidgetKind::QuickReads,
        HomeWidgetKind::PinnedCollections,
        HomeWidgetKind::ReadingStats,
        HomeWidgetKind::FeedDigest,
    ];
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyReviewSummary {
    pub due_count: i64,
    pub streak_days: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingStatsSummary {
    /// Distinct documents opened in the past 7 days (counted from `last_read_at`).
    pub documents_read: i64,
    pub items_completed: i64,
    pub highlights_made: i64,
    pub streak_days: i32,
}
