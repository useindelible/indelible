//! Library-entry-keyed tag membership.

use chrono::{DateTime, Utc};
use sqlx::{Postgres, QueryBuilder};

use ind_application::AppError;
use ind_application::repos::event::MutationSideEffects;
use ind_application::repos::{Cursor, Page};
use ind_domain::{LibraryEntryId, LibraryEntryWithDocument, Tag, TagId, TagSource, UserId};

use crate::cursor::{clamp_limit, decode_cursor_ts, encode_cursor_ts};
use crate::repos::library_repo::rows::{LIBRARY_DOC_COLUMNS, LibraryEntryLinkRow};
use crate::repos::write_helpers::apply_mutation_side_effects_tx;

use super::rows::TagRow;
use super::{PgTagRepository, map_sqlx_error};

impl PgTagRepository {
    pub(super) async fn list_by_library_entry_impl(
        &self,
        user_id: UserId,
        library_entry_id: LibraryEntryId,
    ) -> Result<Vec<Tag>, AppError> {
        let rows = sqlx::query_as!(
            TagRow,
            "SELECT t.id, t.user_id, t.name, t.color, t.parent_id, t.created_at \
             FROM tags t \
             JOIN library_entry_tags let ON let.tag_id = t.id \
             WHERE let.library_entry_id = $1 AND t.user_id = $2 \
             ORDER BY t.name ASC",
            library_entry_id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(Tag::from).collect())
    }

    pub(super) async fn replace_for_library_entry_impl(
        &self,
        user_id: UserId,
        library_entry_id: LibraryEntryId,
        tag_ids: &[TagId],
        effects: MutationSideEffects,
    ) -> Result<(), AppError> {
        self.replace_for_library_entry_with_source_impl(
            user_id,
            library_entry_id,
            tag_ids,
            TagSource::Manual,
            effects,
        )
        .await
    }

    pub(super) async fn replace_for_library_entry_with_source_impl(
        &self,
        user_id: UserId,
        library_entry_id: LibraryEntryId,
        tag_ids: &[TagId],
        source: TagSource,
        effects: MutationSideEffects,
    ) -> Result<(), AppError> {
        let source_str = match source {
            TagSource::Manual => "manual",
            TagSource::Ai => "ai",
            TagSource::Import => "import",
        };

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Repository(Box::new(e)))?;

        sqlx::query!(
            "DELETE FROM library_entry_tags \
             WHERE library_entry_id = $1 \
               AND EXISTS (SELECT 1 FROM library_entries WHERE id = $1 AND user_id = $2)",
            library_entry_id.into_uuid(),
            user_id.into_uuid(),
        )
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        let now = Utc::now();
        for tag_id in tag_ids {
            sqlx::query!(
                "INSERT INTO library_entry_tags \
                    (user_id, library_entry_id, tag_id, source, added_at) \
                 VALUES ($1, $2, $3, $4, $5) \
                 ON CONFLICT (library_entry_id, tag_id) DO NOTHING",
                user_id.into_uuid(),
                library_entry_id.into_uuid(),
                tag_id.into_uuid(),
                source_str,
                now,
            )
            .execute(&mut *tx)
            .await
            .map_err(map_sqlx_error)?;
        }

        apply_mutation_side_effects_tx(&mut tx, effects).await?;

        tx.commit()
            .await
            .map_err(|e| AppError::Repository(Box::new(e)))?;

        Ok(())
    }

    pub(super) async fn list_tag_library_entries_impl(
        &self,
        tag_id: TagId,
        user_id: UserId,
        cursor: Option<Cursor>,
        limit: u32,
    ) -> Result<Page<LibraryEntryWithDocument>, AppError> {
        let limit = clamp_limit(limit);
        let fetch_limit = limit + 1;

        let mut builder = QueryBuilder::<Postgres>::new("SELECT ");
        builder.push(LIBRARY_DOC_COLUMNS);
        builder.push(
            ", let.added_at AS link_added_at \
             FROM library_entry_tags let \
             JOIN library_entries le ON le.id = let.library_entry_id \
             JOIN documents d ON d.id = le.document_id AND d.user_id = le.user_id \
             WHERE let.tag_id = ",
        );
        builder.push_bind(tag_id.into_uuid());
        builder.push(" AND le.user_id = ");
        builder.push_bind(user_id.into_uuid());
        builder.push(" AND le.deleted_at IS NULL");

        if let Some(ref cursor) = cursor {
            let (cursor_ts, cursor_id) = decode_cursor_ts(cursor)?;
            builder.push(" AND (let.added_at, le.id) < (");
            builder.push_bind(cursor_ts);
            builder.push(", ");
            builder.push_bind(cursor_id);
            builder.push(")");
        }

        builder.push(" ORDER BY let.added_at DESC, le.id DESC LIMIT ");
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
}
