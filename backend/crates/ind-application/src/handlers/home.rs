use std::sync::Arc;

use futures::future::BoxFuture;
use ind_domain::{Collection, DailyReviewSummary, HomeWidgetKind, ReadingStatsSummary, UserId};

use crate::AppError;
use crate::ports::content::HomeOperations;
use crate::repos::home::{HomeDocument, HomeFeedEntry, HomeRepository};

pub struct HomeDashboardData {
    pub continue_reading: Option<Vec<HomeDocument>>,
    pub daily_review: Option<DailyReviewSummary>,
    pub recently_added: Option<Vec<HomeDocument>>,
    pub quick_reads: Option<Vec<HomeDocument>>,
    pub pinned_collections: Option<Vec<(Collection, Vec<HomeDocument>)>>,
    pub reading_stats: Option<ReadingStatsSummary>,
    pub feed_digest: Option<Vec<HomeFeedEntry>>,
}

pub struct HomeService {
    home_repo: Arc<dyn HomeRepository>,
}

impl HomeService {
    pub fn new(home_repo: Arc<dyn HomeRepository>) -> Self {
        Self { home_repo }
    }

    pub async fn get_dashboard(
        &self,
        user_id: UserId,
        widgets: Option<Vec<HomeWidgetKind>>,
    ) -> Result<HomeDashboardData, AppError> {
        let active_widgets = widgets.as_deref().unwrap_or(HomeWidgetKind::ALL);

        let want = |kind: HomeWidgetKind| active_widgets.contains(&kind);

        let (
            continue_reading,
            daily_review,
            recently_added,
            quick_reads,
            pinned_collections,
            reading_stats,
            feed_digest,
        ) = tokio::join!(
            self.fetch_if(want(HomeWidgetKind::ContinueReading), || {
                let repo = self.home_repo.clone();
                async move { repo.continue_reading(user_id, 10).await }
            }),
            self.fetch_if(want(HomeWidgetKind::DailyReview), || {
                let repo = self.home_repo.clone();
                async move { repo.daily_review_summary(user_id).await }
            }),
            self.fetch_if(want(HomeWidgetKind::RecentlyAdded), || {
                let repo = self.home_repo.clone();
                async move { repo.recently_added(user_id, 10).await }
            }),
            self.fetch_if(want(HomeWidgetKind::QuickReads), || {
                let repo = self.home_repo.clone();
                async move { repo.quick_reads(user_id, 10).await }
            }),
            self.fetch_if(want(HomeWidgetKind::PinnedCollections), || {
                let repo = self.home_repo.clone();
                async move { repo.pinned_collections(user_id, 5, 3).await }
            }),
            self.fetch_if(want(HomeWidgetKind::ReadingStats), || {
                let repo = self.home_repo.clone();
                async move { repo.reading_stats_weekly(user_id).await }
            }),
            self.fetch_if(want(HomeWidgetKind::FeedDigest), || {
                let repo = self.home_repo.clone();
                async move { repo.feed_digest(user_id, 5).await }
            }),
        );

        Ok(HomeDashboardData {
            continue_reading,
            daily_review,
            recently_added,
            quick_reads,
            pinned_collections,
            reading_stats,
            feed_digest,
        })
    }

    // Widget failures are intentionally isolated: a broken widget returns None (omitted from the
    // response) rather than failing the entire dashboard request. Clients treat absent widgets
    // the same as unrequested ones, which is acceptable for a best-effort dashboard.
    async fn fetch_if<T, F, Fut>(&self, enabled: bool, f: F) -> Option<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, AppError>>,
    {
        if !enabled {
            return None;
        }
        match f().await {
            Ok(data) => Some(data),
            Err(e) => {
                tracing::warn!(error = %e, "home widget query failed, returning null");
                None
            }
        }
    }

    pub async fn get_widget_config(
        &self,
        user_id: UserId,
    ) -> Result<Option<serde_json::Value>, AppError> {
        self.home_repo.get_widget_config(user_id).await
    }

    pub async fn set_widget_config(
        &self,
        user_id: UserId,
        config: serde_json::Value,
    ) -> Result<(), AppError> {
        self.home_repo.set_widget_config(user_id, config).await
    }
}

impl HomeOperations for HomeService {
    fn get_dashboard<'a>(
        &'a self,
        user_id: UserId,
        widgets: Option<Vec<HomeWidgetKind>>,
    ) -> BoxFuture<'a, Result<HomeDashboardData, AppError>> {
        Box::pin(self.get_dashboard(user_id, widgets))
    }

    fn get_widget_config<'a>(
        &'a self,
        user_id: UserId,
    ) -> BoxFuture<'a, Result<Option<serde_json::Value>, AppError>> {
        Box::pin(self.get_widget_config(user_id))
    }

    fn set_widget_config<'a>(
        &'a self,
        user_id: UserId,
        config: serde_json::Value,
    ) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(self.set_widget_config(user_id, config))
    }
}
