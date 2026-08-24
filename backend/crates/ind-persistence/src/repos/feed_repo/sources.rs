use chrono::{DateTime, Duration, Utc};
use ind_application::{AppError, text::strip_nul};
use ind_domain::*;
use uuid::Uuid;

use crate::cursor::clamp_limit;

use super::PgFeedRepository;
use super::types::*;

struct SourceRow {
    id: Uuid,
    canonical_key: String,
    source_url: String,
    poll_url: String,
    title: String,
    description: Option<String>,
    site_url: Option<String>,
    image_url: Option<String>,
    domain: Option<String>,
    feed_type: String,
    visibility: String,
    provider: Option<String>,
    is_resolvable: bool,
    popularity: i32,
    last_entry_added_at: Option<DateTime<Utc>>,
    last_polled_at: Option<DateTime<Utc>>,
    next_poll_at: Option<DateTime<Utc>>,
    last_etag: Option<String>,
    last_modified: Option<String>,
    consecutive_failures: i32,
    last_error: Option<String>,
    lease_owner: Option<String>,
    lease_expires_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<SourceRow> for FeedSource {
    type Error = AppError;

    fn try_from(row: SourceRow) -> Result<Self, Self::Error> {
        Ok(FeedSource {
            id: FeedSourceId::from_uuid(row.id),
            canonical_key: row.canonical_key,
            source_url: row.source_url,
            poll_url: row.poll_url,
            title: row.title,
            description: row.description,
            site_url: row.site_url,
            image_url: row.image_url,
            domain: row.domain,
            feed_type: parse_feed_type(&row.feed_type)?,
            visibility: parse_visibility(&row.visibility)?,
            provider: row.provider,
            is_resolvable: row.is_resolvable,
            popularity: row.popularity,
            last_entry_added_at: row.last_entry_added_at,
            last_polled_at: row.last_polled_at,
            next_poll_at: row.next_poll_at,
            last_etag: row.last_etag,
            last_modified: row.last_modified,
            consecutive_failures: row.consecutive_failures,
            last_error: row.last_error,
            lease_owner: row.lease_owner,
            lease_expires_at: row.lease_expires_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

/// A feed source is a mirror of a remote document: its title, description, URLs and the
/// HTTP validators all arrive verbatim from the network, and Postgres `text` rejects the NUL
/// (`0x00`) a mis-decoded feed can carry. `canonical_key` is also the lookup key, so
/// `find_source_by_canonical_key_impl` strips it the same way.
fn strip_nul_from_source_text(source: &mut FeedSource) {
    // Destructured field by field on purpose: a new column breaks this function until
    // someone decides whether it needs sanitizing.
    let FeedSource {
        canonical_key,
        source_url,
        poll_url,
        title,
        description,
        site_url,
        image_url,
        domain,
        last_etag,
        last_modified,
        last_error,
        // Server-controlled: identifiers, enums, counters, timestamps, the worker lease, and
        // the provider slug we assign ourselves.
        id: _,
        feed_type: _,
        visibility: _,
        provider: _,
        is_resolvable: _,
        popularity: _,
        last_entry_added_at: _,
        last_polled_at: _,
        next_poll_at: _,
        consecutive_failures: _,
        lease_owner: _,
        lease_expires_at: _,
        created_at: _,
        updated_at: _,
    } = source;

    *canonical_key = strip_nul(canonical_key);
    *source_url = strip_nul(source_url);
    *poll_url = strip_nul(poll_url);
    *title = strip_nul(title);
    *description = description.as_deref().map(strip_nul);
    *site_url = site_url.as_deref().map(strip_nul);
    *image_url = image_url.as_deref().map(strip_nul);
    *domain = domain.as_deref().map(strip_nul);
    *last_etag = last_etag.as_deref().map(strip_nul);
    *last_modified = last_modified.as_deref().map(strip_nul);
    *last_error = last_error.as_deref().map(strip_nul);
}

/// The poll writers update the HTTP validators and the error text on their own, so they carry
/// remote bytes into the same columns [`strip_nul_from_source_text`] guards at create time.
fn strip_nul_from_poll_outcome(state: &mut PollOutcome) {
    // Destructured field by field on purpose: a new column breaks this function until
    // someone decides whether it needs sanitizing.
    let PollOutcome {
        last_etag,
        last_modified,
        last_error,
        // Server-controlled: the source identifier, a counter, and timestamps.
        source_id: _,
        last_polled_at: _,
        next_poll_at: _,
        consecutive_failures: _,
    } = state;

    *last_etag = last_etag.as_deref().map(strip_nul);
    *last_modified = last_modified.as_deref().map(strip_nul);
    *last_error = last_error.as_deref().map(strip_nul);
}

/// Same remote metadata as [`strip_nul_from_source_text`], re-read on every refresh, so NUL
/// can reappear long after the source was first created.
fn strip_nul_from_source_details(details: &mut SourceDetailsUpdate) {
    // Destructured field by field on purpose: a new column breaks this function until
    // someone decides whether it needs sanitizing.
    let SourceDetailsUpdate {
        poll_url,
        title,
        description,
        site_url,
        image_url,
        domain,
        // Server-controlled: enums, the provider slug, and a bool.
        feed_type: _,
        visibility: _,
        provider: _,
        is_resolvable: _,
    } = details;

    *poll_url = strip_nul(poll_url);
    *title = strip_nul(title);
    *description = description.as_deref().map(strip_nul);
    *site_url = site_url.as_deref().map(strip_nul);
    *image_url = image_url.as_deref().map(strip_nul);
    *domain = domain.as_deref().map(strip_nul);
}

impl PgFeedRepository {
    pub(super) async fn find_source_by_id_impl(
        &self,
        id: FeedSourceId,
    ) -> Result<Option<FeedSource>, AppError> {
        let row = sqlx::query_as!(
            SourceRow,
            "SELECT id, canonical_key, source_url, poll_url, title, description, site_url, \
                    image_url, domain, feed_type, visibility, provider, is_resolvable, \
                    popularity, last_entry_added_at, last_polled_at, next_poll_at, last_etag, \
                    last_modified, consecutive_failures, last_error, lease_owner, \
                    lease_expires_at, created_at, updated_at \
             FROM feed_sources WHERE id = $1",
            id.into_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_source_error)?;

        row.map(FeedSource::try_from).transpose()
    }

    pub(super) async fn find_source_by_canonical_key_impl(
        &self,
        canonical_key: &str,
    ) -> Result<Option<FeedSource>, AppError> {
        // The write strips NUL, so a stored key is always NUL-free; the lookup has to strip
        // the same way or a caller's raw key would never match the row it stored.
        let canonical_key = strip_nul(canonical_key);
        let row = sqlx::query_as!(
            SourceRow,
            "SELECT id, canonical_key, source_url, poll_url, title, description, site_url, \
                    image_url, domain, feed_type, visibility, provider, is_resolvable, \
                    popularity, last_entry_added_at, last_polled_at, next_poll_at, last_etag, \
                    last_modified, consecutive_failures, last_error, lease_owner, \
                    lease_expires_at, created_at, updated_at \
             FROM feed_sources WHERE canonical_key = $1",
            canonical_key
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_source_error)?;

        row.map(FeedSource::try_from).transpose()
    }

    pub(super) async fn create_source_impl(
        &self,
        mut source: FeedSource,
    ) -> Result<FeedSource, AppError> {
        strip_nul_from_source_text(&mut source);
        let row = sqlx::query_as!(
            SourceRow,
            "INSERT INTO feed_sources \
                (id, canonical_key, source_url, poll_url, title, description, site_url, \
                 image_url, domain, feed_type, visibility, provider, is_resolvable, \
                 popularity, last_entry_added_at, last_polled_at, next_poll_at, last_etag, \
                 last_modified, consecutive_failures, last_error, lease_owner, lease_expires_at, \
                 created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, \
                     $16, $17, $18, $19, $20, $21, $22, $23, $24, $25) \
             RETURNING id, canonical_key, source_url, poll_url, title, description, site_url, \
                       image_url, domain, feed_type, visibility, provider, is_resolvable, \
                       popularity, last_entry_added_at, last_polled_at, next_poll_at, last_etag, \
                       last_modified, consecutive_failures, last_error, lease_owner, \
                       lease_expires_at, created_at, updated_at",
            source.id.into_uuid(),
            source.canonical_key,
            source.source_url,
            source.poll_url,
            source.title,
            source.description,
            source.site_url,
            source.image_url,
            source.domain,
            feed_type_to_str(source.feed_type),
            visibility_to_str(source.visibility),
            source.provider,
            source.is_resolvable,
            source.popularity,
            source.last_entry_added_at,
            source.last_polled_at,
            source.next_poll_at,
            source.last_etag,
            source.last_modified,
            source.consecutive_failures,
            source.last_error,
            source.lease_owner,
            source.lease_expires_at,
            source.created_at,
            source.updated_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_source_error)?;

        FeedSource::try_from(row)
    }

    pub(super) async fn update_source_details_impl(
        &self,
        id: FeedSourceId,
        mut details: ind_domain::SourceDetailsUpdate,
    ) -> Result<FeedSource, AppError> {
        strip_nul_from_source_details(&mut details);
        let row = sqlx::query_as!(
            SourceRow,
            "UPDATE feed_sources \
             SET poll_url = $2, title = $3, description = $4, site_url = $5, image_url = $6, \
                 domain = $7, feed_type = $8, visibility = $9, provider = $10, \
                 is_resolvable = $11, \
                 updated_at = now() \
             WHERE id = $1 \
             RETURNING id, canonical_key, source_url, poll_url, title, description, site_url, \
                       image_url, domain, feed_type, visibility, provider, is_resolvable, \
                       popularity, last_entry_added_at, last_polled_at, next_poll_at, last_etag, \
                       last_modified, consecutive_failures, last_error, lease_owner, \
                       lease_expires_at, created_at, updated_at",
            id.into_uuid(),
            details.poll_url,
            details.title,
            details.description,
            details.site_url,
            details.image_url,
            details.domain,
            feed_type_to_str(details.feed_type),
            visibility_to_str(details.visibility),
            details.provider,
            details.is_resolvable,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_source_error)?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "feed_source",
                id: id.to_string(),
            })
        })?;

        FeedSource::try_from(row)
    }

    pub(super) async fn bump_source_popularity_impl(
        &self,
        id: FeedSourceId,
        delta: i32,
    ) -> Result<FeedSource, AppError> {
        let row = sqlx::query_as!(
            SourceRow,
            "UPDATE feed_sources \
             SET popularity = GREATEST(0, popularity + $2), updated_at = now() \
             WHERE id = $1 \
             RETURNING id, canonical_key, source_url, poll_url, title, description, site_url, \
                       image_url, domain, feed_type, visibility, provider, is_resolvable, \
                       popularity, last_entry_added_at, last_polled_at, next_poll_at, last_etag, \
                       last_modified, consecutive_failures, last_error, lease_owner, \
                       lease_expires_at, created_at, updated_at",
            id.into_uuid(),
            delta,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_source_error)?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "feed_source",
                id: id.to_string(),
            })
        })?;

        FeedSource::try_from(row)
    }

    pub(super) async fn mark_source_poll_requested_impl(
        &self,
        id: FeedSourceId,
        next_poll_at: DateTime<Utc>,
    ) -> Result<FeedSource, AppError> {
        let row = sqlx::query_as!(
            SourceRow,
            "UPDATE feed_sources \
             SET next_poll_at = $2, updated_at = now() \
             WHERE id = $1 \
             RETURNING id, canonical_key, source_url, poll_url, title, description, site_url, \
                       image_url, domain, feed_type, visibility, provider, is_resolvable, \
                       popularity, last_entry_added_at, last_polled_at, next_poll_at, last_etag, \
                       last_modified, consecutive_failures, last_error, lease_owner, \
                       lease_expires_at, created_at, updated_at",
            id.into_uuid(),
            next_poll_at,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_source_error)?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "feed_source",
                id: id.to_string(),
            })
        })?;

        FeedSource::try_from(row)
    }

    pub(super) async fn mark_source_poll_success_impl(
        &self,
        id: FeedSourceId,
        mut state: PollOutcome,
        last_entry_added_at: Option<DateTime<Utc>>,
    ) -> Result<FeedSource, AppError> {
        strip_nul_from_poll_outcome(&mut state);
        let row = sqlx::query_as!(
            SourceRow,
            "UPDATE feed_sources \
             SET last_polled_at = $2, next_poll_at = $3, last_etag = $4, last_modified = $5, \
                 consecutive_failures = $6, last_error = $7, last_entry_added_at = COALESCE($8, last_entry_added_at), \
                 lease_owner = NULL, lease_expires_at = NULL, updated_at = now() \
             WHERE id = $1 \
             RETURNING id, canonical_key, source_url, poll_url, title, description, site_url, \
                       image_url, domain, feed_type, visibility, provider, is_resolvable, \
                       popularity, last_entry_added_at, last_polled_at, next_poll_at, last_etag, \
                       last_modified, consecutive_failures, last_error, lease_owner, \
                       lease_expires_at, created_at, updated_at",
            id.into_uuid(),
            state.last_polled_at,
            state.next_poll_at,
            state.last_etag.as_deref(),
            state.last_modified.as_deref(),
            state.consecutive_failures,
            state.last_error.as_deref(),
            last_entry_added_at,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_source_error)?
        .ok_or_else(|| AppError::Domain(DomainError::NotFound {
            entity: "feed_source",
            id: id.to_string(),
        }))?;

        FeedSource::try_from(row)
    }

    pub(super) async fn mark_source_poll_failure_impl(
        &self,
        id: FeedSourceId,
        next_poll_at: DateTime<Utc>,
        error: String,
        consecutive_failures: i32,
    ) -> Result<FeedSource, AppError> {
        // Poll failures quote what the remote sent — a URL, a parser message, a response
        // fragment — so a NUL from the network can reach this column.
        let error = strip_nul(&error);
        let row = sqlx::query_as!(
            SourceRow,
            "UPDATE feed_sources \
             SET next_poll_at = $2, last_error = $3, consecutive_failures = $4, \
                 lease_owner = NULL, lease_expires_at = NULL, updated_at = now() \
             WHERE id = $1 \
             RETURNING id, canonical_key, source_url, poll_url, title, description, site_url, \
                       image_url, domain, feed_type, visibility, provider, is_resolvable, \
                       popularity, last_entry_added_at, last_polled_at, next_poll_at, last_etag, \
                       last_modified, consecutive_failures, last_error, lease_owner, \
                       lease_expires_at, created_at, updated_at",
            id.into_uuid(),
            next_poll_at,
            error,
            consecutive_failures,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_source_error)?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "feed_source",
                id: id.to_string(),
            })
        })?;

        FeedSource::try_from(row)
    }

    pub(super) async fn clear_source_lease_impl(&self, id: FeedSourceId) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE feed_sources \
             SET lease_owner = NULL, lease_expires_at = NULL, updated_at = now() \
             WHERE id = $1",
            id.into_uuid()
        )
        .execute(&self.pool)
        .await
        .map_err(map_source_error)?;

        Ok(())
    }

    pub(super) async fn claim_due_sources_impl(
        &self,
        now: DateTime<Utc>,
        worker_id: &str,
        limit: i64,
        lease_duration: Duration,
    ) -> Result<Vec<FeedSource>, AppError> {
        let lease_expires_at = now + lease_duration;
        let rows = sqlx::query_as!(
            SourceRow,
            "WITH due AS ( \
                SELECT fs.id \
                FROM feed_sources fs \
                WHERE EXISTS ( \
                        SELECT 1 \
                        FROM feed_subscriptions sub \
                        WHERE sub.source_id = fs.id \
                          AND sub.status = 'active' \
                    ) \
                  AND COALESCE(next_poll_at, $1) <= $1 \
                  AND (lease_expires_at IS NULL OR lease_expires_at < $1) \
                ORDER BY next_poll_at NULLS FIRST, updated_at ASC \
                LIMIT $2 \
                FOR UPDATE SKIP LOCKED \
             ) \
             UPDATE feed_sources fs \
             SET lease_owner = $3, lease_expires_at = $4, updated_at = now() \
             FROM due \
             WHERE fs.id = due.id \
             RETURNING fs.id, fs.canonical_key, fs.source_url, fs.poll_url, fs.title, \
                       fs.description, fs.site_url, fs.image_url, fs.domain, fs.feed_type, \
                       fs.visibility, fs.provider, fs.is_resolvable, fs.popularity, \
                       fs.last_entry_added_at, fs.last_polled_at, fs.next_poll_at, fs.last_etag, \
                       fs.last_modified, fs.consecutive_failures, fs.last_error, \
                       fs.lease_owner, fs.lease_expires_at, fs.created_at, fs.updated_at",
            now,
            limit,
            worker_id,
            lease_expires_at,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_source_error)?;

        rows.into_iter()
            .map(FeedSource::try_from)
            .collect::<Result<Vec<_>, _>>()
    }

    pub(super) async fn search_public_sources_impl(
        &self,
        query: &str,
        surface: FeedSearchSurface,
        limit: u32,
    ) -> Result<Vec<FeedSource>, AppError> {
        let limit = clamp_limit(limit);
        let query_trimmed = query.trim();
        let escaped = escape_like(query_trimmed);
        let like = format!("%{escaped}%");
        let prefix = format!("{escaped}%");
        let surface = surface_to_str(surface);
        let rows = sqlx::query_as!(
            SourceRow,
            "SELECT id, canonical_key, source_url, poll_url, title, description, site_url, \
                    image_url, domain, feed_type, visibility, provider, is_resolvable, \
                    popularity, last_entry_added_at, last_polled_at, next_poll_at, last_etag, \
                    last_modified, consecutive_failures, last_error, lease_owner, \
                    lease_expires_at, created_at, updated_at \
             FROM feed_sources \
             WHERE visibility = 'public' \
               AND (title ILIKE $1 ESCAPE '\\' OR COALESCE(description, '') ILIKE $1 ESCAPE '\\' \
                    OR COALESCE(domain, '') ILIKE $1 ESCAPE '\\' OR source_url ILIKE $1 ESCAPE '\\' \
                    OR poll_url ILIKE $1 ESCAPE '\\') \
               AND ($2 = 'all' \
                    OR ($2 = 'youtube' AND feed_type = 'youtube') \
                    OR ($2 = 'twitter' AND feed_type = 'twitter') \
                    OR ($2 = 'rss' AND feed_type <> 'youtube' AND feed_type <> 'twitter')) \
             ORDER BY \
               CASE \
                   WHEN lower(title) = lower($3) THEN 300 \
                   WHEN title ILIKE $4 ESCAPE '\\' THEN 220 \
                   WHEN domain ILIKE $4 ESCAPE '\\' THEN 180 \
                   ELSE 0 \
               END DESC, \
               CASE \
                   WHEN $2 = 'youtube' AND feed_type = 'youtube' THEN 50 \
                   WHEN $2 = 'twitter' AND feed_type = 'twitter' THEN 50 \
                   WHEN $2 = 'rss' AND feed_type <> 'youtube' AND feed_type <> 'twitter' THEN 25 \
                   ELSE 0 \
               END DESC, \
               popularity DESC, \
               last_entry_added_at DESC NULLS LAST, \
               updated_at DESC \
             LIMIT $5",
            like,
            surface,
            query_trimmed,
            prefix,
            limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_source_error)?;

        rows.into_iter()
            .map(FeedSource::try_from)
            .collect::<Result<Vec<_>, _>>()
    }
}
