use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::obsidian_preview::{
    ObsidianPreviewDocument, ObsidianPreviewHighlight, ObsidianPreviewRepository,
};
use ind_domain::{DocumentId, DomainError, HighlightId, ItemType, LibraryEntryId, UserId};

pub struct PgObsidianPreviewRepository {
    pool: PgPool,
}

impl PgObsidianPreviewRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

struct ObsidianPreviewItemRow {
    id: Uuid,
    library_entry_id: Uuid,
    title: String,
    url: Option<String>,
    author: Option<String>,
    item_type: String,
    lead_image_url: Option<String>,
    excerpt: Option<String>,
}

struct ObsidianPreviewHighlightRow {
    id: Uuid,
    text: String,
    color: String,
    created_at: DateTime<Utc>,
    note: Option<String>,
}

struct HighlightTagRow {
    highlight_id: Uuid,
    name: String,
}

fn parse_item_type(raw: &str) -> Result<ItemType, AppError> {
    raw.parse::<ItemType>().map_err(|_| {
        AppError::Domain(DomainError::InvariantViolation {
            message: format!("invalid item type: {raw}"),
        })
    })
}

fn repo_err(err: sqlx::Error) -> AppError {
    AppError::Repository(Box::new(err))
}

#[async_trait::async_trait]
impl ObsidianPreviewRepository for PgObsidianPreviewRepository {
    async fn load_document(
        &self,
        user_id: UserId,
        library_entry_id: LibraryEntryId,
    ) -> Result<Option<ObsidianPreviewDocument>, AppError> {
        let row = sqlx::query_as!(
            ObsidianPreviewItemRow,
            r#"SELECT d.id, le.id AS "library_entry_id!", d.title,
                      COALESCE(d.original_url, d.canonical_url) AS url,
                      d.author, d.document_type AS item_type, d.lead_image_url, d.excerpt
               FROM library_entries le
               JOIN documents d ON d.id = le.document_id AND d.user_id = le.user_id
               WHERE le.id = $1
                 AND le.user_id = $2
                 AND le.deleted_at IS NULL"#,
            library_entry_id.into_uuid(),
            user_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(repo_err)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let tag_rows = sqlx::query_scalar!(
            r#"SELECT t.name
               FROM library_entry_tags let
               JOIN tags t ON t.id = let.tag_id
               WHERE let.library_entry_id = $1
               ORDER BY t.name"#,
            row.library_entry_id,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(repo_err)?;

        let highlight_rows = sqlx::query_as!(
            ObsidianPreviewHighlightRow,
            r#"SELECT h.id, h.text_content AS text, h.color, h.created_at, hn.body AS "note?"
               FROM highlights h
               LEFT JOIN highlight_notes hn ON hn.highlight_id = h.id
               WHERE h.document_id = $1 AND h.user_id = $2
               ORDER BY h.created_at, h.id"#,
            row.id,
            user_id.into_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(repo_err)?;

        let highlight_ids: Vec<Uuid> = highlight_rows
            .iter()
            .map(|highlight| highlight.id)
            .collect();
        let highlight_tag_rows = if highlight_ids.is_empty() {
            Vec::new()
        } else {
            sqlx::query_as!(
                HighlightTagRow,
                r#"SELECT ht.highlight_id, t.name
                   FROM highlight_tags ht
                   JOIN tags t ON t.id = ht.tag_id
                   WHERE ht.highlight_id = ANY($1)
                   ORDER BY ht.highlight_id, t.name"#,
                &highlight_ids,
            )
            .fetch_all(&self.pool)
            .await
            .map_err(repo_err)?
        };

        let mut tags_by_highlight: HashMap<Uuid, Vec<String>> = HashMap::new();
        for tag_row in highlight_tag_rows {
            tags_by_highlight
                .entry(tag_row.highlight_id)
                .or_default()
                .push(tag_row.name);
        }

        Ok(Some(ObsidianPreviewDocument {
            document_id: DocumentId::from_uuid(row.id),
            library_entry_id: LibraryEntryId::from_uuid(row.library_entry_id),
            title: row.title,
            url: row.url,
            author: row.author,
            item_type: parse_item_type(&row.item_type)?,
            lead_image_url: row.lead_image_url,
            excerpt: row.excerpt,
            tags: tag_rows,
            highlights: highlight_rows
                .into_iter()
                .map(|highlight| ObsidianPreviewHighlight {
                    id: HighlightId::from_uuid(highlight.id),
                    text: highlight.text,
                    color: highlight.color,
                    created_at: highlight.created_at,
                    note: highlight.note,
                    tags: tags_by_highlight.remove(&highlight.id).unwrap_or_default(),
                })
                .collect(),
        }))
    }
}
