use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::content_vector::ContentVectorSourceRef;
use ind_domain::DocumentId;

use super::PgContentVectorRepository;
use super::types::*;

impl PgContentVectorRepository {
    pub(super) async fn source_refs_for_chunks_impl(
        &self,
        chunk_ids: &[Uuid],
    ) -> Result<Vec<ContentVectorSourceRef>, AppError> {
        if chunk_ids.is_empty() {
            return Ok(Vec::new());
        }

        let rows = sqlx::query_as!(
            SourceRefRow,
            r#"
            SELECT cv.id AS chunk_id, cv.document_id AS "document_id!", d.title
            FROM content_vectors cv
            JOIN documents d ON d.id = cv.document_id
            WHERE cv.id = ANY($1)
            "#,
            chunk_ids,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows
            .into_iter()
            .map(|row| ContentVectorSourceRef {
                chunk_id: row.chunk_id,
                document_id: DocumentId::from_uuid(row.document_id),
                title: row.title,
            })
            .collect())
    }
}
