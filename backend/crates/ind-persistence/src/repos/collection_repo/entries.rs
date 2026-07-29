//! Library-entry-keyed collection membership (TASK-235). Collections organize saved Library
//! content, so membership rows live in `collection_entries(collection_id, library_entry_id)` and
//! contents are returned as `LibraryEntryWithDocument`. Prepared-but-unsaved feed documents have no
//! library entry and so never appear here.

use chrono::{DateTime, Utc};
use sqlx::{Postgres, QueryBuilder};

use ind_application::AppError;
use ind_application::repos::{Cursor, Page};
use ind_domain::{CollectionId, DomainError, LibraryEntryId, LibraryEntryWithDocument, UserId};

use super::{PgCollectionRepository, map_sqlx_error};
use crate::cursor::{clamp_limit, decode_cursor_ts, encode_cursor_ts};
use crate::repos::library_repo::rows::{LIBRARY_DOC_COLUMNS, LibraryEntryLinkRow};

impl PgCollectionRepository {
    pub(super) async fn add_library_entry_link(
        &self,
        user_id: UserId,
        collection_id: CollectionId,
        library_entry_id: LibraryEntryId,
    ) -> Result<(), AppError> {
        // The (collection_id, user_id) and (library_entry_id, user_id) composite FKs reject the
        // insert unless both parents belong to this user, so cross-tenant links cannot be forged.
        sqlx::query!(
            "INSERT INTO collection_entries (user_id, collection_id, library_entry_id, added_at) \
             VALUES ($1, $2, $3, now()) \
             ON CONFLICT (collection_id, library_entry_id) DO NOTHING",
            user_id.into_uuid(),
            collection_id.into_uuid(),
            library_entry_id.into_uuid(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    pub(super) async fn remove_library_entry_link(
        &self,
        user_id: UserId,
        collection_id: CollectionId,
        library_entry_id: LibraryEntryId,
    ) -> Result<(), AppError> {
        let result = sqlx::query!(
            "DELETE FROM collection_entries \
             WHERE collection_id = $1 AND library_entry_id = $2 AND user_id = $3",
            collection_id.into_uuid(),
            library_entry_id.into_uuid(),
            user_id.into_uuid(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(AppError::Domain(DomainError::NotFound {
                entity: "collection_entry",
                id: format!("collection={collection_id}, library_entry={library_entry_id}"),
            }));
        }

        Ok(())
    }

    pub(super) async fn list_collection_entries_query(
        &self,
        collection_id: CollectionId,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<LibraryEntryWithDocument>, AppError> {
        let limit = clamp_limit(limit);
        let fetch_limit = limit + 1;

        // QueryBuilder keeps the optional `added_at` cursor clause dynamic; the row is the shared
        // LibraryWithDocRow flattened plus the membership `added_at` used for keyset pagination.
        let mut builder = QueryBuilder::<Postgres>::new("SELECT ");
        builder.push(LIBRARY_DOC_COLUMNS);
        builder.push(
            ", ce.added_at AS link_added_at \
             FROM collection_entries ce \
             JOIN library_entries le ON le.id = ce.library_entry_id \
             JOIN documents d ON d.id = le.document_id AND d.user_id = le.user_id \
             WHERE ce.collection_id = ",
        );
        builder.push_bind(collection_id.into_uuid());
        builder.push(" AND le.user_id = ");
        builder.push_bind(user_id.into_uuid());
        builder.push(" AND le.deleted_at IS NULL");

        if let Some(ref cursor) = cursor {
            let (cursor_ts, cursor_id) = decode_cursor_ts(cursor)?;
            builder.push(" AND (ce.added_at, le.id) < (");
            builder.push_bind(cursor_ts);
            builder.push(", ");
            builder.push_bind(cursor_id);
            builder.push(")");
        }

        builder.push(" ORDER BY ce.added_at DESC, le.id DESC LIMIT ");
        builder.push_bind(fetch_limit);

        let rows: Vec<LibraryEntryLinkRow> = builder
            .build_query_as()
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

        let has_more = rows.len() as i64 > limit;
        let take = if has_more { limit as usize } else { rows.len() };
        let cursor_seed: Option<(DateTime<Utc>, uuid::Uuid)> = if has_more {
            rows.get(take - 1).map(|r| (r.link_added_at, r.entry.id))
        } else {
            None
        };

        let items: Vec<LibraryEntryWithDocument> = rows
            .into_iter()
            .take(take)
            .map(|r| r.entry.into_with_document())
            .collect::<Result<_, _>>()?;

        let next_cursor = cursor_seed.map(|(added_at, id)| encode_cursor_ts(added_at, id));

        Ok(Page { items, next_cursor })
    }

    pub(super) async fn count_library_entries_query(
        &self,
        collection_id: CollectionId,
    ) -> Result<i64, AppError> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM collection_entries ce \
             JOIN library_entries le ON le.id = ce.library_entry_id AND le.deleted_at IS NULL \
             WHERE ce.collection_id = $1",
            collection_id.into_uuid()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(count.unwrap_or(0))
    }
}
