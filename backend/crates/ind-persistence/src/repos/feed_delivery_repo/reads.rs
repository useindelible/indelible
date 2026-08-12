use chrono::{DateTime, Utc};
use ind_application::AppError;
use ind_application::repos::feed_delivery::FeedDeliveryUpsert;
use ind_application::repos::lifecycle_outbox::OutboxEntry;
use ind_application::repos::{Cursor, Page};
use ind_domain::{
    DomainError, FeedAutosaveJob, FeedDelivery, FeedDeliveryDisplay, FeedDeliveryId,
    FeedDeliveryState, FeedSubscriptionId, UserId,
};
use sqlx::{Postgres, Transaction};

use super::PgFeedDeliveryRepository;
use super::rows::{DeliveryDisplayRow, DeliveryRow, map_delivery_error};
use crate::cursor::{clamp_limit, decode_cursor_ts, encode_cursor_ts};
use crate::repos::write_helpers::enqueue_outbox_tx;

impl PgFeedDeliveryRepository {
    pub(super) async fn upsert_delivery_impl(
        &self,
        delivery: FeedDelivery,
    ) -> Result<FeedDeliveryUpsert, AppError> {
        let mut tx = self.pool.begin().await.map_err(map_delivery_error)?;
        let upsert = upsert_delivery_tx(&mut tx, delivery).await?;
        tx.commit().await.map_err(map_delivery_error)?;
        Ok(upsert)
    }

    pub(super) async fn upsert_delivery_with_autosave_impl(
        &self,
        delivery: FeedDelivery,
        autosave: Option<FeedAutosaveJob>,
        available_at: DateTime<Utc>,
    ) -> Result<FeedDeliveryUpsert, AppError> {
        let mut tx = self.pool.begin().await.map_err(map_delivery_error)?;
        let upsert = upsert_delivery_tx(&mut tx, delivery).await?;

        if upsert.newly_inserted
            && let Some(job) = autosave
        {
            let payload =
                serde_json::to_value(job).map_err(|e| AppError::Repository(Box::new(e)))?;
            enqueue_outbox_tx(
                &mut tx,
                &OutboxEntry {
                    job_type: "feed.autosave".into(),
                    payload,
                    dedupe_key: Some(format!("feed.autosave:{}", upsert.delivery.id)),
                    available_at,
                },
            )
            .await?;
        }

        tx.commit().await.map_err(map_delivery_error)?;
        Ok(upsert)
    }
}

async fn upsert_delivery_tx(
    tx: &mut Transaction<'_, Postgres>,
    delivery: FeedDelivery,
) -> Result<FeedDeliveryUpsert, AppError> {
    // Guard tenant ownership and referential consistency before writing: the
    // subscription must belong to this user and point at the given source, and the
    // source entry must belong to that source. The plain FKs only prove the rows
    // exist, not that they belong together or to the same user.
    let consistent = sqlx::query_scalar!(
        "SELECT EXISTS ( \
                 SELECT 1 FROM feed_subscriptions s \
                 JOIN feed_source_entries fse ON fse.source_id = s.source_id \
                 WHERE s.id = $1 AND s.user_id = $2 AND s.source_id = $3 AND fse.id = $4 \
             )",
        delivery.subscription_id.into_uuid(),
        delivery.user_id.into_uuid(),
        delivery.source_id.into_uuid(),
        delivery.source_entry_id.into_uuid(),
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(map_delivery_error)?;

    if consistent != Some(true) {
        return Err(AppError::Domain(DomainError::Validation {
            field: "subscription_id".into(),
            message: "feed delivery references a subscription/source/entry that is not \
                          consistent or not owned by the user"
                .into(),
        }));
    }

    let inserted = sqlx::query_as!(
        DeliveryRow,
        "INSERT INTO feed_deliveries \
                (id, user_id, subscription_id, source_id, source_entry_id, document_id, \
                 delivered_at, seen_at, dismissed_at, hidden_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
             ON CONFLICT (user_id, subscription_id, source_entry_id) DO NOTHING \
             RETURNING id, user_id, subscription_id, source_id, source_entry_id, document_id, \
                       delivered_at, seen_at, dismissed_at, hidden_at, created_at, updated_at",
        delivery.id.into_uuid(),
        delivery.user_id.into_uuid(),
        delivery.subscription_id.into_uuid(),
        delivery.source_id.into_uuid(),
        delivery.source_entry_id.into_uuid(),
        delivery.document_id.map(|id| id.into_uuid()),
        delivery.delivered_at,
        delivery.seen_at,
        delivery.dismissed_at,
        delivery.hidden_at,
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_delivery_error)?;

    if let Some(row) = inserted {
        return Ok(FeedDeliveryUpsert {
            delivery: FeedDelivery::from(row),
            newly_inserted: true,
        });
    }

    // Convergent link-on-discovery: for an existing delivery, set document_id only when the
    // row is unlinked and the incoming row carries one. User state and delivered_at are never
    // touched. If no link update is needed, the select below returns the existing delivery.
    if let Some(document_id) = delivery.document_id {
        let linked = sqlx::query_as!(
            DeliveryRow,
            "UPDATE feed_deliveries \
                 SET document_id = $4, updated_at = now() \
                 WHERE user_id = $1 AND subscription_id = $2 AND source_entry_id = $3 \
                   AND document_id IS NULL \
                 RETURNING id, user_id, subscription_id, source_id, source_entry_id, document_id, \
                           delivered_at, seen_at, dismissed_at, hidden_at, created_at, updated_at",
            delivery.user_id.into_uuid(),
            delivery.subscription_id.into_uuid(),
            delivery.source_entry_id.into_uuid(),
            document_id.into_uuid(),
        )
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_delivery_error)?;

        if let Some(row) = linked {
            return Ok(FeedDeliveryUpsert {
                delivery: FeedDelivery::from(row),
                newly_inserted: false,
            });
        }
    }

    let existing = sqlx::query_as!(
        DeliveryRow,
        "SELECT id, user_id, subscription_id, source_id, source_entry_id, document_id, \
                    delivered_at, seen_at, dismissed_at, hidden_at, created_at, updated_at \
             FROM feed_deliveries \
             WHERE user_id = $1 AND subscription_id = $2 AND source_entry_id = $3",
        delivery.user_id.into_uuid(),
        delivery.subscription_id.into_uuid(),
        delivery.source_entry_id.into_uuid(),
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(map_delivery_error)?;

    Ok(FeedDeliveryUpsert {
        delivery: FeedDelivery::from(existing),
        newly_inserted: false,
    })
}

impl PgFeedDeliveryRepository {
    pub(super) async fn find_by_id_impl(
        &self,
        id: FeedDeliveryId,
        user_id: UserId,
    ) -> Result<Option<FeedDelivery>, AppError> {
        let row = sqlx::query_as!(
            DeliveryRow,
            "SELECT id, user_id, subscription_id, source_id, source_entry_id, document_id, \
                    delivered_at, seen_at, dismissed_at, hidden_at, created_at, updated_at \
             FROM feed_deliveries WHERE id = $1 AND user_id = $2",
            id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_delivery_error)?;

        Ok(row.map(FeedDelivery::from))
    }

    /// Unseen/Seen Feed list (docs/document-feed-library-architecture.md, Query Surfaces ->
    /// Feed). Source-entry first via the `JOIN feed_source_entries`; documents are a left join
    /// because most deliveries are never prepared. The `library_entries` left join plus
    /// `le.id IS NULL` hides saved documents (AC #5). Subscription filter and keyset cursor are
    /// folded into SQL predicates so each state needs a single query. Unseen sorts by the source
    /// entry's publication time, falling back to `delivered_at`; Seen sorts by `seen_at`.
    pub(super) async fn list_deliveries_impl(
        &self,
        user_id: UserId,
        state: FeedDeliveryState,
        subscription_id: Option<FeedSubscriptionId>,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<FeedDeliveryDisplay>, AppError> {
        let limit = clamp_limit(limit);
        let fetch_limit = limit + 1;
        let subscription = subscription_id.map(|id| id.into_uuid());
        let (cursor_ts, cursor_id) = match &cursor {
            Some(cursor) => {
                let (ts, id) = decode_cursor_ts(cursor)?;
                (Some(ts), Some(id))
            }
            None => (None, None),
        };

        let rows = match state {
            FeedDeliveryState::Unseen => sqlx::query_as!(
                DeliveryDisplayRow,
                "SELECT fd.id, fd.user_id, fd.subscription_id, fd.source_id, fd.source_entry_id, \
                        fd.document_id, fd.delivered_at, fd.seen_at, fd.dismissed_at, fd.hidden_at, \
                        fd.created_at, fd.updated_at, \
                        fse.title AS entry_title, fse.url AS entry_url, fse.author AS entry_author, \
                        fse.excerpt AS entry_excerpt, fse.published_at AS entry_published_at, \
                        fse.lead_image_url AS entry_lead_image_url, \
                        d.document_type AS \"doc_document_type?\", d.title AS \"doc_title?\", \
                        d.canonical_url AS \"doc_canonical_url?\", d.author AS \"doc_author?\", \
                        d.excerpt AS \"doc_excerpt?\", d.lead_image_url AS \"doc_lead_image_url?\", \
                        d.thumbnail_url AS \"doc_thumbnail_url?\", (le.id IS NOT NULL) AS \"saved!\" \
                 FROM feed_deliveries fd \
                 JOIN feed_source_entries fse ON fse.id = fd.source_entry_id \
                 LEFT JOIN documents d ON d.id = fd.document_id AND d.user_id = fd.user_id \
                 LEFT JOIN library_entries le \
                   ON le.user_id = fd.user_id AND le.document_id = d.id AND le.deleted_at IS NULL \
                 WHERE fd.user_id = $1 AND fd.seen_at IS NULL AND fd.dismissed_at IS NULL \
                   AND fd.hidden_at IS NULL AND le.id IS NULL \
                   AND ($2::uuid IS NULL OR fd.subscription_id = $2) \
                   AND ($3::timestamptz IS NULL OR \
                        (COALESCE(fse.published_at, fd.delivered_at), fd.id) < ($3, $4)) \
                 ORDER BY COALESCE(fse.published_at, fd.delivered_at) DESC, fd.id DESC \
                 LIMIT $5",
                user_id.into_uuid(),
                subscription,
                cursor_ts,
                cursor_id,
                fetch_limit,
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_delivery_error)?,
            FeedDeliveryState::Seen => sqlx::query_as!(
                DeliveryDisplayRow,
                "SELECT fd.id, fd.user_id, fd.subscription_id, fd.source_id, fd.source_entry_id, \
                        fd.document_id, fd.delivered_at, fd.seen_at, fd.dismissed_at, fd.hidden_at, \
                        fd.created_at, fd.updated_at, \
                        fse.title AS entry_title, fse.url AS entry_url, fse.author AS entry_author, \
                        fse.excerpt AS entry_excerpt, fse.published_at AS entry_published_at, \
                        fse.lead_image_url AS entry_lead_image_url, \
                        d.document_type AS \"doc_document_type?\", d.title AS \"doc_title?\", \
                        d.canonical_url AS \"doc_canonical_url?\", d.author AS \"doc_author?\", \
                        d.excerpt AS \"doc_excerpt?\", d.lead_image_url AS \"doc_lead_image_url?\", \
                        d.thumbnail_url AS \"doc_thumbnail_url?\", (le.id IS NOT NULL) AS \"saved!\" \
                 FROM feed_deliveries fd \
                 JOIN feed_source_entries fse ON fse.id = fd.source_entry_id \
                 LEFT JOIN documents d ON d.id = fd.document_id AND d.user_id = fd.user_id \
                 LEFT JOIN library_entries le \
                   ON le.user_id = fd.user_id AND le.document_id = d.id AND le.deleted_at IS NULL \
                 WHERE fd.user_id = $1 AND fd.seen_at IS NOT NULL AND fd.dismissed_at IS NULL \
                   AND fd.hidden_at IS NULL AND le.id IS NULL \
                   AND ($2::uuid IS NULL OR fd.subscription_id = $2) \
                   AND ($3::timestamptz IS NULL OR (fd.seen_at, fd.id) < ($3, $4)) \
                 ORDER BY fd.seen_at DESC, fd.id DESC \
                 LIMIT $5",
                user_id.into_uuid(),
                subscription,
                cursor_ts,
                cursor_id,
                fetch_limit,
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_delivery_error)?,
        };

        let has_more = rows.len() as i64 > limit;
        let items: Vec<FeedDeliveryDisplay> = rows
            .into_iter()
            .take(limit as usize)
            .map(DeliveryDisplayRow::into_display)
            .collect::<Result<_, _>>()?;
        let next_cursor = if has_more {
            items.last().map(|d| {
                // Cursor column matches the ORDER BY column for the requested state.
                let ts = match state {
                    FeedDeliveryState::Unseen => {
                        d.entry_published_at.unwrap_or(d.delivery.delivered_at)
                    }
                    FeedDeliveryState::Seen => {
                        d.delivery.seen_at.unwrap_or(d.delivery.delivered_at)
                    }
                };
                encode_cursor_ts(ts, d.delivery.id.into_uuid())
            })
        } else {
            None
        };

        Ok(Page { items, next_cursor })
    }

    /// Single delivery with document overlay and true `saved` flag. Unlike the list reads it
    /// does not exclude seen/saved/dismissed/hidden rows, so it reports the real state for a
    /// known delivery id.
    pub(super) async fn find_display_by_id_impl(
        &self,
        id: FeedDeliveryId,
        user_id: UserId,
    ) -> Result<Option<FeedDeliveryDisplay>, AppError> {
        let row = sqlx::query_as!(
            DeliveryDisplayRow,
            "SELECT fd.id, fd.user_id, fd.subscription_id, fd.source_id, fd.source_entry_id, \
                    fd.document_id, fd.delivered_at, fd.seen_at, fd.dismissed_at, fd.hidden_at, \
                    fd.created_at, fd.updated_at, \
                    fse.title AS entry_title, fse.url AS entry_url, fse.author AS entry_author, \
                    fse.excerpt AS entry_excerpt, fse.published_at AS entry_published_at, \
                        fse.lead_image_url AS entry_lead_image_url, \
                    d.document_type AS \"doc_document_type?\", d.title AS \"doc_title?\", \
                    d.canonical_url AS \"doc_canonical_url?\", d.author AS \"doc_author?\", \
                    d.excerpt AS \"doc_excerpt?\", d.lead_image_url AS \"doc_lead_image_url?\", \
                    d.thumbnail_url AS \"doc_thumbnail_url?\", (le.id IS NOT NULL) AS \"saved!\" \
             FROM feed_deliveries fd \
             JOIN feed_source_entries fse ON fse.id = fd.source_entry_id \
             LEFT JOIN documents d ON d.id = fd.document_id AND d.user_id = fd.user_id \
             LEFT JOIN library_entries le \
               ON le.user_id = fd.user_id AND le.document_id = d.id AND le.deleted_at IS NULL \
             WHERE fd.id = $1 AND fd.user_id = $2",
            id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_delivery_error)?;

        row.map(DeliveryDisplayRow::into_display).transpose()
    }

    pub(super) async fn count_unseen_impl(&self, user_id: UserId) -> Result<i64, AppError> {
        // Same saved-exclusion join shape as list_deliveries (through documents) so the badge
        // count can never drift from the Unseen list.
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) AS \"count!\" \
             FROM feed_deliveries fd \
             LEFT JOIN documents d ON d.id = fd.document_id AND d.user_id = fd.user_id \
             LEFT JOIN library_entries le \
               ON le.user_id = fd.user_id AND le.document_id = d.id AND le.deleted_at IS NULL \
             WHERE fd.user_id = $1 AND fd.seen_at IS NULL AND fd.dismissed_at IS NULL \
               AND fd.hidden_at IS NULL AND le.id IS NULL",
            user_id.into_uuid(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_delivery_error)?;

        Ok(count)
    }

    /// Read-ahead eligibility (docs/document-feed-library-architecture.md, Readable Content
    /// Preparation Policy). Selects the newest unread, URL-backed deliveries from active
    /// subscriptions whose linked/materialized document has no completed readable asset.
    /// "Active subscription" = `status='active'` AND either the caller targets this
    /// `subscription_id` directly (bypasses the activity check) or a delivery from it was seen
    /// within `active_within_days`. There is intentionally no `content_html` truncation gate.
    pub(super) async fn list_prefetch_candidates_impl(
        &self,
        user_id: UserId,
        subscription_id: Option<FeedSubscriptionId>,
        active_within_days: i64,
        limit: u32,
    ) -> Result<Vec<FeedDelivery>, AppError> {
        let limit = clamp_limit(limit);
        let subscription = subscription_id.map(|id| id.into_uuid());

        let rows = sqlx::query_as!(
            DeliveryRow,
            "SELECT fd.id, fd.user_id, fd.subscription_id, fd.source_id, fd.source_entry_id, \
                    fd.document_id, fd.delivered_at, fd.seen_at, fd.dismissed_at, fd.hidden_at, \
                    fd.created_at, fd.updated_at \
             FROM feed_deliveries fd \
             JOIN feed_source_entries fse ON fse.id = fd.source_entry_id \
             JOIN feed_subscriptions s ON s.id = fd.subscription_id \
             LEFT JOIN archive_assets aa \
               ON aa.document_id = fd.document_id AND aa.asset_kind = 'readable_html' \
               AND aa.status = 'completed' \
             WHERE fd.user_id = $1 \
               AND fd.seen_at IS NULL AND fd.dismissed_at IS NULL AND fd.hidden_at IS NULL \
               AND fse.canonical_url IS NOT NULL \
               AND s.status = 'active' \
               AND aa.id IS NULL \
               AND ($2::uuid IS NULL OR fd.subscription_id = $2) \
               AND ($2::uuid IS NOT NULL OR EXISTS ( \
                       SELECT 1 FROM feed_deliveries fd2 \
                       WHERE fd2.user_id = fd.user_id \
                         AND fd2.subscription_id = fd.subscription_id \
                         AND fd2.seen_at >= now() - make_interval(days => $3) \
                   )) \
             ORDER BY COALESCE(fse.published_at, fd.delivered_at) DESC, fd.id DESC \
             LIMIT $4",
            user_id.into_uuid(),
            subscription,
            active_within_days as i32,
            limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_delivery_error)?;

        Ok(rows.into_iter().map(FeedDelivery::from).collect())
    }
}
