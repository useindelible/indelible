//! List/filter engine for saved Library content keyed on `library_entries` and `documents`.

use sqlx::{PgPool, Postgres, QueryBuilder};

use ind_application::AppError;
use ind_application::repos::{Cursor, Page};
use ind_domain::{FilterNode, LibraryEntryWithDocument, UserId};

use super::library_repo::rows::{LIBRARY_DOC_COLUMNS, LibraryWithDocRow};
use crate::cursor::{clamp_limit, decode_cursor_ts, encode_cursor_ts};

mod filters;

use filters::push_filter_node;

/// Inputs for a library-entry list query. `trashed_only` flips the soft-delete predicate to read
/// the Trash view instead of the active Library.
pub(crate) struct LibraryListFilter {
    pub filter_expression: Option<FilterNode>,
    pub trashed_only: bool,
}

pub(crate) async fn query_library_entries_page(
    pool: &PgPool,
    user_id: UserId,
    filter: &LibraryListFilter,
    cursor: Option<Cursor>,
    limit: u32,
) -> Result<Page<LibraryEntryWithDocument>, AppError> {
    let limit = clamp_limit(limit);
    let fetch_limit = limit + 1;

    let mut builder = QueryBuilder::<Postgres>::new("SELECT ");
    builder.push(LIBRARY_DOC_COLUMNS);
    builder.push(
        " FROM library_entries le \
         JOIN documents d ON d.id = le.document_id AND d.user_id = le.user_id \
         WHERE le.user_id = ",
    );
    builder.push_bind(user_id.into_uuid());

    if filter.trashed_only {
        builder.push(" AND le.deleted_at IS NOT NULL");
    } else {
        builder.push(" AND le.deleted_at IS NULL");
    }

    if let Some(filter_expression) = filter.filter_expression.as_ref() {
        builder.push(" AND ");
        push_filter_node(&mut builder, filter_expression)?;
    }

    if let Some(cursor) = cursor.as_ref() {
        let (cursor_ts, cursor_id) = decode_cursor_ts(cursor)?;
        builder.push(" AND (le.saved_at, le.id) < (");
        builder.push_bind(cursor_ts);
        builder.push(", ");
        builder.push_bind(cursor_id);
        builder.push(")");
    }

    builder.push(" ORDER BY le.saved_at DESC, le.id DESC LIMIT ");
    builder.push_bind(fetch_limit);

    let rows: Vec<LibraryWithDocRow> = builder
        .build_query_as()
        .fetch_all(pool)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;

    let has_more = rows.len() as i64 > limit;
    let items: Vec<LibraryEntryWithDocument> = rows
        .into_iter()
        .take(limit as usize)
        .map(LibraryWithDocRow::into_with_document)
        .collect::<Result<_, _>>()?;

    let next_cursor = if has_more {
        items
            .last()
            .map(|e| encode_cursor_ts(e.entry.saved_at, e.entry.id.into_uuid()))
    } else {
        None
    };

    Ok(Page { items, next_cursor })
}
