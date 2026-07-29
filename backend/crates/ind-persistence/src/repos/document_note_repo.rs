use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::document_note::DocumentNoteRepository;
use ind_application::repos::lifecycle_outbox::OutboxEntry;
use ind_domain::{DocumentId, DocumentNote, ItemNoteId, UserId};

use super::write_helpers::apply_outbox_tx;

pub struct PgDocumentNoteRepository {
    pool: PgPool,
}

impl PgDocumentNoteRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct DocumentNoteRow {
    id: Uuid,
    document_id: Uuid,
    user_id: Uuid,
    body: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<DocumentNoteRow> for DocumentNote {
    fn from(row: DocumentNoteRow) -> Self {
        DocumentNote {
            id: ItemNoteId::from_uuid(row.id),
            document_id: DocumentId::from_uuid(row.document_id),
            user_id: UserId::from_uuid(row.user_id),
            body: row.body,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

fn map_sqlx_error(err: sqlx::Error) -> AppError {
    super::map_sqlx_error("document_note", "duplicate note for document", err)
}

#[async_trait::async_trait]
impl DocumentNoteRepository for PgDocumentNoteRepository {
    async fn find_by_document(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<Option<DocumentNote>, AppError> {
        let row = sqlx::query_as!(
            DocumentNoteRow,
            "SELECT id, document_id AS \"document_id!\", user_id, body, created_at, updated_at \
             FROM item_notes \
             WHERE document_id = $1 AND user_id = $2",
            document_id.into_uuid(),
            user_id.into_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(DocumentNote::from))
    }

    async fn upsert_for_document(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        body: &str,
        outbox: Vec<OutboxEntry>,
    ) -> Result<DocumentNote, AppError> {
        let id = ItemNoteId::new();
        let now = Utc::now();
        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;
        let row = sqlx::query_as!(
            DocumentNoteRow,
            "INSERT INTO item_notes (id, document_id, user_id, body, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $5) \
             ON CONFLICT (user_id, document_id) WHERE document_id IS NOT NULL DO UPDATE SET \
             body = EXCLUDED.body, \
             updated_at = EXCLUDED.updated_at \
             RETURNING id, document_id AS \"document_id!\", user_id, body, created_at, updated_at",
            id.into_uuid(),
            document_id.into_uuid(),
            user_id.into_uuid(),
            body,
            now,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        apply_outbox_tx(&mut tx, &outbox).await?;

        tx.commit().await.map_err(map_sqlx_error)?;
        Ok(DocumentNote::from(row))
    }

    async fn delete_for_document(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "DELETE FROM item_notes WHERE document_id = $1 AND user_id = $2",
            document_id.into_uuid(),
            user_id.into_uuid()
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }
}
