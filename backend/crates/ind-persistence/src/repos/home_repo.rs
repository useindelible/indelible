use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::home::{HomeDocument, HomeFeedEntry, HomeRepository};
use ind_domain::{
    Collection, CollectionId, DailyReviewSummary, DocumentId, FeedDeliveryId, FeedSubscriptionId,
    ItemType, ReadingStatsSummary, UserId,
};

pub struct PgHomeRepository {
    pool: PgPool,
}

impl PgHomeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn map_sqlx_error(err: sqlx::Error) -> AppError {
    AppError::Repository(Box::new(err))
}

fn parse_item_type(s: &str) -> Result<ItemType, AppError> {
    s.parse::<ItemType>().map_err(|_| {
        AppError::Domain(ind_domain::DomainError::InvariantViolation {
            message: format!("invalid item type: {s}"),
        })
    })
}

#[derive(sqlx::FromRow)]
struct HomeDocumentRow {
    id: Uuid,
    item_type: String,
    title: String,
    excerpt: Option<String>,
    url: Option<String>,
    domain: Option<String>,
    author: Option<String>,
    reading_time_minutes: Option<i32>,
    lead_image_url: Option<String>,
    progress_percent: Option<f32>,
    max_progress_percent: Option<f32>,
    last_read_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl TryFrom<HomeDocumentRow> for HomeDocument {
    type Error = AppError;

    fn try_from(row: HomeDocumentRow) -> Result<Self, Self::Error> {
        Ok(HomeDocument {
            document_id: DocumentId::from_uuid(row.id),
            item_type: parse_item_type(&row.item_type)?,
            title: row.title,
            excerpt: row.excerpt,
            url: row.url,
            domain: row.domain,
            author: row.author,
            reading_time_minutes: row.reading_time_minutes,
            lead_image_url: row.lead_image_url,
            progress_percent: row.progress_percent,
            max_progress_percent: row.max_progress_percent,
            last_read_at: row.last_read_at,
            created_at: row.created_at,
        })
    }
}

#[derive(sqlx::FromRow)]
struct CollectionRow {
    id: Uuid,
    user_id: Uuid,
    parent_id: Option<Uuid>,
    name: String,
    description: Option<String>,
    icon: Option<String>,
    color: Option<String>,
    sort_order: i32,
    is_pinned: bool,
    rss_token: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<CollectionRow> for Collection {
    fn from(row: CollectionRow) -> Self {
        Collection {
            id: CollectionId::from_uuid(row.id),
            user_id: UserId::from_uuid(row.user_id),
            parent_id: row.parent_id.map(CollectionId::from_uuid),
            name: row.name,
            description: row.description,
            icon: row.icon,
            color: row.color,
            sort_order: row.sort_order,
            is_pinned: row.is_pinned,
            rss_token: row.rss_token,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct HomeFeedEntryRow {
    id: Uuid,
    subscription_id: Uuid,
    title: String,
    url: Option<String>,
    author: Option<String>,
    excerpt: Option<String>,
    published_at: Option<DateTime<Utc>>,
}

impl From<HomeFeedEntryRow> for HomeFeedEntry {
    fn from(row: HomeFeedEntryRow) -> Self {
        HomeFeedEntry {
            delivery_id: FeedDeliveryId::from_uuid(row.id),
            subscription_id: FeedSubscriptionId::from_uuid(row.subscription_id),
            title: row.title,
            url: row.url,
            author: row.author,
            excerpt: row.excerpt,
            published_at: row.published_at,
        }
    }
}

#[async_trait::async_trait]
impl HomeRepository for PgHomeRepository {
    async fn continue_reading(
        &self,
        user_id: UserId,
        limit: i64,
    ) -> Result<Vec<HomeDocument>, AppError> {
        let rows = sqlx::query_as!(
            HomeDocumentRow,
            "SELECT le.document_id AS id, d.document_type AS item_type, \
             d.title, d.excerpt, d.original_url AS url, d.domain, d.author, \
             NULL::int AS reading_time_minutes, d.lead_image_url, \
             uds.progress_percent::real AS \"progress_percent?\", \
             uds.max_progress_percent::real AS \"max_progress_percent?\", uds.last_read_at, \
             le.created_at \
             FROM library_entries le \
             JOIN documents d ON d.id = le.document_id AND d.user_id = le.user_id \
             JOIN user_document_state uds \
                 ON uds.user_id = le.user_id AND uds.document_id = le.document_id \
             WHERE le.user_id = $1 \
             AND le.deleted_at IS NULL \
             AND uds.last_read_at IS NOT NULL \
             AND COALESCE(uds.max_progress_percent, 0) < 99.5 \
             AND uds.finished_at IS NULL \
             ORDER BY uds.last_read_at DESC NULLS LAST \
             LIMIT $2",
            user_id.into_uuid(),
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(HomeDocument::try_from).collect()
    }

    async fn daily_review_summary(&self, user_id: UserId) -> Result<DailyReviewSummary, AppError> {
        let due_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM review_cards \
             WHERE user_id = $1 AND status = 'active' AND next_due_at <= now()",
            user_id.into_uuid()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?
        .unwrap_or(0);

        let streak_days = sqlx::query_scalar!(
            "WITH daily AS ( \
                SELECT DISTINCT date_trunc('day', reviewed_at) AS day \
                FROM review_events re JOIN review_cards rc ON rc.id = re.card_id \
                WHERE rc.user_id = $1 \
                ORDER BY day DESC \
            ), numbered AS ( \
                SELECT day, row_number() OVER (ORDER BY day DESC) AS rn FROM daily \
            ) \
            SELECT COUNT(*)::int FROM numbered \
            WHERE day = current_date - make_interval(days => (rn - 1)::int)",
            user_id.into_uuid()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?
        .unwrap_or(0);

        Ok(DailyReviewSummary {
            due_count,
            streak_days,
        })
    }

    async fn recently_added(
        &self,
        user_id: UserId,
        limit: i64,
    ) -> Result<Vec<HomeDocument>, AppError> {
        let rows = sqlx::query_as!(
            HomeDocumentRow,
            "SELECT le.document_id AS id, d.document_type AS item_type, \
             d.title, d.excerpt, d.original_url AS url, d.domain, d.author, \
             NULL::int AS reading_time_minutes, d.lead_image_url, \
             uds.progress_percent::real AS \"progress_percent?\", \
             uds.max_progress_percent::real AS \"max_progress_percent?\", uds.last_read_at, \
             le.created_at \
             FROM library_entries le \
             JOIN documents d ON d.id = le.document_id AND d.user_id = le.user_id \
             LEFT JOIN user_document_state uds \
                 ON uds.user_id = le.user_id AND uds.document_id = le.document_id \
             WHERE le.user_id = $1 \
             AND le.deleted_at IS NULL \
             AND le.created_at >= now() - interval '7 days' \
             ORDER BY le.created_at DESC \
             LIMIT $2",
            user_id.into_uuid(),
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(HomeDocument::try_from).collect()
    }

    async fn quick_reads(
        &self,
        user_id: UserId,
        limit: i64,
    ) -> Result<Vec<HomeDocument>, AppError> {
        let rows = sqlx::query_as!(
            HomeDocumentRow,
            "SELECT le.document_id AS id, d.document_type AS item_type, \
             d.title, d.excerpt, d.original_url AS url, d.domain, d.author, \
             NULL::int AS reading_time_minutes, d.lead_image_url, \
             uds.progress_percent::real AS \"progress_percent?\", \
             uds.max_progress_percent::real AS \"max_progress_percent?\", uds.last_read_at, \
             le.created_at \
             FROM library_entries le \
             JOIN documents d ON d.id = le.document_id AND d.user_id = le.user_id \
             LEFT JOIN user_document_state uds \
                 ON uds.user_id = le.user_id AND uds.document_id = le.document_id \
             WHERE le.user_id = $1 \
             AND le.deleted_at IS NULL \
             AND d.document_type = 'article' \
             AND uds.last_read_at IS NULL \
             ORDER BY le.created_at DESC \
             LIMIT $2",
            user_id.into_uuid(),
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(HomeDocument::try_from).collect()
    }

    async fn pinned_collections(
        &self,
        user_id: UserId,
        collection_limit: i64,
        items_per: i64,
    ) -> Result<Vec<(Collection, Vec<HomeDocument>)>, AppError> {
        let collection_rows = sqlx::query_as!(
            CollectionRow,
            "SELECT id, user_id, parent_id, name, description, icon, color, \
             sort_order, is_pinned, rss_token, created_at, updated_at \
             FROM collections \
             WHERE user_id = $1 AND is_pinned = true \
             ORDER BY sort_order ASC, name ASC \
             LIMIT $2",
            user_id.into_uuid(),
            collection_limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        let mut result = Vec::with_capacity(collection_rows.len());

        for col_row in collection_rows {
            let col_id = col_row.id;
            let collection = Collection::from(col_row);

            let item_rows = sqlx::query_as!(
                HomeDocumentRow,
                "SELECT le.document_id AS id, d.document_type AS item_type, \
                 d.title, d.excerpt, d.original_url AS url, d.domain, d.author, \
                 NULL::int AS reading_time_minutes, d.lead_image_url, \
                 uds.progress_percent::real AS \"progress_percent?\", \
                 uds.max_progress_percent::real AS \"max_progress_percent?\", uds.last_read_at, \
                 le.created_at \
                 FROM collection_entries ce \
                 JOIN library_entries le ON le.id = ce.library_entry_id AND le.deleted_at IS NULL \
                 JOIN documents d ON d.id = le.document_id AND d.user_id = le.user_id \
                 LEFT JOIN user_document_state uds \
                     ON uds.user_id = le.user_id AND uds.document_id = le.document_id \
                 WHERE ce.collection_id = $1 \
                 ORDER BY ce.added_at DESC \
                 LIMIT $2",
                col_id,
                items_per
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

            let items: Vec<HomeDocument> = item_rows
                .into_iter()
                .map(HomeDocument::try_from)
                .collect::<Result<_, _>>()?;
            result.push((collection, items));
        }

        Ok(result)
    }

    async fn reading_stats_weekly(&self, user_id: UserId) -> Result<ReadingStatsSummary, AppError> {
        let highlights_made = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM highlights \
             WHERE user_id = $1 AND created_at >= now() - interval '7 days'",
            user_id.into_uuid()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?
        .unwrap_or(0);

        let items_completed = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM user_document_state \
             WHERE user_id = $1 \
             AND finished_at >= now() - interval '7 days' \
             AND finished_at IS NOT NULL",
            user_id.into_uuid()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?
        .unwrap_or(0);

        let documents_read = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM user_document_state \
             WHERE user_id = $1 \
             AND last_read_at >= now() - interval '7 days' \
             AND last_read_at IS NOT NULL",
            user_id.into_uuid()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?
        .unwrap_or(0);

        let streak_days = sqlx::query_scalar!(
            "WITH daily AS ( \
                SELECT DISTINCT date_trunc('day', last_read_at) AS day \
                FROM user_document_state WHERE user_id = $1 \
                AND last_read_at IS NOT NULL \
                ORDER BY day DESC \
            ), numbered AS ( \
                SELECT day, row_number() OVER (ORDER BY day DESC) AS rn FROM daily \
            ) \
            SELECT COUNT(*)::int FROM numbered \
            WHERE day = current_date - make_interval(days => (rn - 1)::int)",
            user_id.into_uuid()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?
        .unwrap_or(0);

        Ok(ReadingStatsSummary {
            documents_read,
            items_completed,
            highlights_made,
            streak_days,
        })
    }

    async fn feed_digest(
        &self,
        user_id: UserId,
        limit: i64,
    ) -> Result<Vec<HomeFeedEntry>, AppError> {
        let rows = sqlx::query_as!(
            HomeFeedEntryRow,
            "SELECT fd.id, fd.subscription_id, fse.title, fse.url, fse.author, fse.excerpt, \
             fse.published_at \
             FROM feed_deliveries fd \
             JOIN feed_source_entries fse ON fse.id = fd.source_entry_id \
             WHERE fd.user_id = $1 \
             AND fd.seen_at IS NULL \
             AND fd.dismissed_at IS NULL \
             AND fd.hidden_at IS NULL \
             ORDER BY fse.published_at DESC NULLS LAST, fd.delivered_at DESC \
             LIMIT $2",
            user_id.into_uuid(),
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(HomeFeedEntry::from).collect())
    }

    async fn get_widget_config(
        &self,
        user_id: UserId,
    ) -> Result<Option<serde_json::Value>, AppError> {
        let config = sqlx::query_scalar!(
            "SELECT home_widget_config FROM user_preferences WHERE user_id = $1",
            user_id.into_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(config.flatten())
    }

    async fn set_widget_config(
        &self,
        user_id: UserId,
        config: serde_json::Value,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "INSERT INTO user_preferences (user_id, home_widget_config, updated_at) \
             VALUES ($1, $2, now()) \
             ON CONFLICT (user_id) DO UPDATE \
             SET home_widget_config = EXCLUDED.home_widget_config, updated_at = now()",
            user_id.into_uuid(),
            config
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }
}
