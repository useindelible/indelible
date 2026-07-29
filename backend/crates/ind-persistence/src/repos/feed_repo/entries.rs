use chrono::{DateTime, Utc};
use ind_application::{AppError, normalize_language_tag};
use ind_domain::*;
use uuid::Uuid;

use super::PgFeedRepository;
use super::types::*;

struct SourceEntryRow {
    id: Uuid,
    source_id: Uuid,
    guid: String,
    title: String,
    url: Option<String>,
    canonical_url: Option<String>,
    author: Option<String>,
    excerpt: Option<String>,
    content_html: Option<String>,
    language: Option<String>,
    lead_image_url: Option<String>,
    published_at: Option<DateTime<Utc>>,
    discovered_at: DateTime<Utc>,
}

impl From<SourceEntryRow> for FeedSourceEntry {
    fn from(row: SourceEntryRow) -> Self {
        FeedSourceEntry {
            id: FeedSourceEntryId::from_uuid(row.id),
            source_id: FeedSourceId::from_uuid(row.source_id),
            guid: row.guid,
            title: row.title,
            url: row.url,
            canonical_url: row.canonical_url,
            author: row.author,
            excerpt: row.excerpt,
            content_html: row.content_html,
            language: row.language,
            lead_image_url: row.lead_image_url,
            published_at: row.published_at,
            discovered_at: row.discovered_at,
        }
    }
}

impl PgFeedRepository {
    pub(super) async fn set_source_entry_language_if_missing_impl(
        &self,
        entry_id: FeedSourceEntryId,
        language: &str,
    ) -> Result<bool, AppError> {
        let Some(language) = normalize_language_tag(Some(language)) else {
            return Ok(false);
        };
        let result = sqlx::query!(
            "UPDATE feed_source_entries \
             SET language = $2 \
             WHERE id = $1 AND language IS NULL",
            entry_id.into_uuid(),
            language,
        )
        .execute(&self.pool)
        .await
        .map_err(map_entry_error)?;

        Ok(result.rows_affected() == 1)
    }

    pub(super) async fn find_source_entry_by_source_guid_impl(
        &self,
        source_id: FeedSourceId,
        guid: &str,
    ) -> Result<Option<FeedSourceEntry>, AppError> {
        let row = sqlx::query_as!(
            SourceEntryRow,
            "SELECT id, source_id, guid, title, url, canonical_url, author, excerpt, content_html, language, \
                    lead_image_url, published_at, discovered_at \
             FROM feed_source_entries \
             WHERE source_id = $1 AND guid = $2",
            source_id.into_uuid(),
            guid,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_entry_error)?;

        Ok(row.map(FeedSourceEntry::from))
    }

    pub(super) async fn find_source_entry_by_id_impl(
        &self,
        id: FeedSourceEntryId,
    ) -> Result<Option<FeedSourceEntry>, AppError> {
        let row = sqlx::query_as!(
            SourceEntryRow,
            "SELECT id, source_id, guid, title, url, canonical_url, author, excerpt, content_html, language, \
                    lead_image_url, published_at, discovered_at \
             FROM feed_source_entries \
             WHERE id = $1",
            id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_entry_error)?;

        Ok(row.map(FeedSourceEntry::from))
    }

    pub(super) async fn create_source_entry_impl(
        &self,
        mut entry: FeedSourceEntry,
    ) -> Result<FeedSourceEntry, AppError> {
        entry.language = normalize_language_tag(entry.language.as_deref());
        let row = sqlx::query_as!(
            SourceEntryRow,
            "INSERT INTO feed_source_entries \
                (id, source_id, guid, title, url, canonical_url, author, excerpt, content_html, language, lead_image_url, published_at, discovered_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) \
             RETURNING id, source_id, guid, title, url, canonical_url, author, excerpt, content_html, language, \
                       lead_image_url, published_at, discovered_at",
            entry.id.into_uuid(),
            entry.source_id.into_uuid(),
            entry.guid,
            entry.title,
            entry.url,
            entry.canonical_url,
            entry.author,
            entry.excerpt,
            entry.content_html,
            entry.language,
            entry.lead_image_url,
            entry.published_at,
            entry.discovered_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_entry_error)?;

        Ok(FeedSourceEntry::from(row))
    }

    pub(super) async fn create_or_adopt_polled_source_entry_impl(
        &self,
        mut entry: FeedSourceEntry,
    ) -> Result<FeedSourceEntry, AppError> {
        entry.language = normalize_language_tag(entry.language.as_deref());
        let mut tx = self.pool.begin().await.map_err(map_entry_error)?;
        let source_lock = entry.source_id.to_string();
        sqlx::query!(
            "SELECT pg_advisory_xact_lock(hashtextextended($1, 0))",
            source_lock,
        )
        .execute(&mut *tx)
        .await
        .map_err(map_entry_error)?;

        let exact = sqlx::query_as!(
            SourceEntryRow,
            "SELECT id, source_id, guid, title, url, canonical_url, author, excerpt, content_html, language, \
                    lead_image_url, published_at, discovered_at \
             FROM feed_source_entries WHERE source_id = $1 AND guid = $2 FOR UPDATE",
            entry.source_id.into_uuid(),
            entry.guid,
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_entry_error)?;
        if let Some(row) = exact {
            tx.commit().await.map_err(map_entry_error)?;
            return Ok(row.into());
        }

        if entry.guid.starts_with("entry-content-") {
            let adopted = sqlx::query_as!(
                SourceEntryRow,
                r#"
                SELECT id, source_id, guid, title, url, canonical_url, author, excerpt,
                       content_html, language, lead_image_url, published_at, discovered_at
                FROM feed_source_entries
                WHERE source_id = $1
                  AND guid ~* '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
                  AND title = $2
                  AND url IS NOT DISTINCT FROM $3
                  AND author IS NOT DISTINCT FROM $4
                  AND excerpt IS NOT DISTINCT FROM $5
                  AND content_html IS NOT DISTINCT FROM $6
                  AND ($7::timestamptz IS NULL OR published_at IS NOT DISTINCT FROM $7)
                ORDER BY discovered_at DESC, id DESC
                LIMIT 1
                FOR UPDATE
                "#,
                entry.source_id.into_uuid(),
                entry.title,
                entry.url,
                entry.author,
                entry.excerpt,
                entry.content_html,
                entry.published_at,
            )
            .fetch_optional(&mut *tx)
            .await
            .map_err(map_entry_error)?;

            if let Some(adopted) = adopted {
                let row = sqlx::query_as!(
                    SourceEntryRow,
                    r#"
                    UPDATE feed_source_entries
                    SET guid = $2,
                        canonical_url = COALESCE(canonical_url, $3),
                        lead_image_url = COALESCE($4, lead_image_url),
                        language = COALESCE(language, $5),
                        published_at = $6
                    WHERE id = $1
                    RETURNING id, source_id, guid, title, url, canonical_url, author, excerpt,
                              content_html, language, lead_image_url, published_at, discovered_at
                    "#,
                    adopted.id,
                    entry.guid,
                    entry.canonical_url,
                    entry.lead_image_url,
                    entry.language,
                    entry.published_at,
                )
                .fetch_one(&mut *tx)
                .await
                .map_err(map_entry_error)?;
                tx.commit().await.map_err(map_entry_error)?;
                return Ok(row.into());
            }
        }

        let row = sqlx::query_as!(
            SourceEntryRow,
            "INSERT INTO feed_source_entries \
                (id, source_id, guid, title, url, canonical_url, author, excerpt, content_html, language, lead_image_url, published_at, discovered_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) \
             RETURNING id, source_id, guid, title, url, canonical_url, author, excerpt, content_html, language, \
                       lead_image_url, published_at, discovered_at",
            entry.id.into_uuid(),
            entry.source_id.into_uuid(),
            entry.guid,
            entry.title,
            entry.url,
            entry.canonical_url,
            entry.author,
            entry.excerpt,
            entry.content_html,
            entry.language,
            entry.lead_image_url,
            entry.published_at,
            entry.discovered_at,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(map_entry_error)?;
        tx.commit().await.map_err(map_entry_error)?;
        Ok(row.into())
    }

    pub(super) async fn set_source_entry_canonical_url_impl(
        &self,
        entry_id: FeedSourceEntryId,
        canonical_url: &str,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE feed_source_entries SET canonical_url = $1 WHERE id = $2",
            canonical_url,
            entry_id.into_uuid(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_entry_error)?;

        Ok(())
    }

    pub(super) async fn source_entries_missing_canonical_url_after_impl(
        &self,
        after_id: Uuid,
        limit: i64,
    ) -> Result<Vec<(FeedSourceEntryId, String)>, AppError> {
        let rows = sqlx::query!(
            "SELECT id, url FROM feed_source_entries \
             WHERE id > $1 AND canonical_url IS NULL AND url IS NOT NULL \
             ORDER BY id LIMIT $2",
            after_id,
            limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_entry_error)?;

        Ok(rows
            .into_iter()
            .filter_map(|r| r.url.map(|url| (FeedSourceEntryId::from_uuid(r.id), url)))
            .collect())
    }
}
