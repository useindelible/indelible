use futures::future::BoxFuture;
use ind_application::AppError;
use ind_application::repos::epub_format::{
    EpubAssetFormat, EpubFormatIssue, EpubFormatRepository, OriginalUploadLocation,
    PendingEpubAsset,
};
use ind_domain::{ArchiveAssetId, DocumentId, DomainError};
use sqlx::PgPool;
use uuid::Uuid;

const BACKFILL_LOCK: &str = "epub-chapter-list-backfill";

pub struct PgEpubFormatRepository {
    pool: PgPool,
}

impl PgEpubFormatRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

struct PendingRow {
    id: Uuid,
    document_id: Uuid,
    toc_key: String,
    original_key: Option<String>,
    original_content_type: Option<String>,
}

impl From<PendingRow> for PendingEpubAsset {
    fn from(row: PendingRow) -> Self {
        let original =
            row.original_key
                .zip(row.original_content_type)
                .map(|(s3_key, content_type)| OriginalUploadLocation {
                    s3_key,
                    content_type,
                });
        PendingEpubAsset {
            asset_id: ArchiveAssetId::from_uuid(row.id),
            document_id: DocumentId::from_uuid(row.document_id),
            toc_key: row.toc_key,
            original,
        }
    }
}

fn repo_error(err: sqlx::Error) -> AppError {
    AppError::Repository(Box::new(err))
}

#[async_trait::async_trait]
impl EpubFormatRepository for PgEpubFormatRepository {
    async fn epub_assets_pending_after(
        &self,
        cursor: Option<ArchiveAssetId>,
        limit: i64,
    ) -> Result<Vec<PendingEpubAsset>, AppError> {
        let rows = sqlx::query_as!(
            PendingRow,
            r#"
            SELECT toc.id, toc.document_id, toc.s3_key AS toc_key,
                   original.s3_key AS "original_key?",
                   original.content_type AS "original_content_type?"
            FROM archive_assets toc
            LEFT JOIN archive_assets original
              ON original.document_id = toc.document_id
             AND original.asset_kind = 'original_upload'
             AND original.status = 'completed'
            WHERE toc.asset_kind = 'epub' AND toc.format_version < 2
              AND toc.status = 'completed' AND toc.format_issue IS NULL
              AND toc.id > $1
            ORDER BY toc.id
            LIMIT $2
            "#,
            cursor.map_or(Uuid::nil(), ArchiveAssetId::into_uuid),
            limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(repo_error)?;
        Ok(rows.into_iter().map(PendingEpubAsset::from).collect())
    }

    async fn pending_epub_asset(
        &self,
        document_id: DocumentId,
    ) -> Result<Option<PendingEpubAsset>, AppError> {
        let row = sqlx::query_as!(
            PendingRow,
            r#"
            SELECT toc.id, toc.document_id, toc.s3_key AS toc_key,
                   original.s3_key AS "original_key?",
                   original.content_type AS "original_content_type?"
            FROM archive_assets toc
            LEFT JOIN archive_assets original
              ON original.document_id = toc.document_id
             AND original.asset_kind = 'original_upload'
             AND original.status = 'completed'
            WHERE toc.asset_kind = 'epub' AND toc.format_version < 2
              AND toc.status = 'completed' AND toc.format_issue IS NULL
              AND toc.document_id = $1
            "#,
            document_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(repo_error)?;
        Ok(row.map(PendingEpubAsset::from))
    }

    async fn find_epub_format(
        &self,
        document_id: DocumentId,
    ) -> Result<Option<EpubAssetFormat>, AppError> {
        let row = sqlx::query!(
            "SELECT format_version, format_issue FROM archive_assets \
             WHERE document_id = $1 AND asset_kind = 'epub'",
            document_id.into_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(repo_error)?;
        row.map(|row| {
            let issue = row
                .format_issue
                .map(|issue| {
                    EpubFormatIssue::parse(&issue).ok_or_else(|| {
                        AppError::Domain(DomainError::InvariantViolation {
                            message: format!("unknown EPUB format issue '{issue}'"),
                        })
                    })
                })
                .transpose()?;
            Ok(EpubAssetFormat {
                version: row.format_version,
                issue,
            })
        })
        .transpose()
    }

    async fn mark_format(
        &self,
        asset_id: ArchiveAssetId,
        version: i32,
        size_bytes: i64,
    ) -> Result<(), AppError> {
        let done = sqlx::query!(
            "UPDATE archive_assets \
             SET format_version = $2, format_issue = NULL, size_bytes = $3 \
             WHERE id = $1",
            asset_id.into_uuid(),
            version,
            size_bytes,
        )
        .execute(&self.pool)
        .await
        .map_err(repo_error)?;
        exactly_one_row(done.rows_affected(), asset_id)
    }

    async fn mark_format_issue(
        &self,
        asset_id: ArchiveAssetId,
        issue: EpubFormatIssue,
    ) -> Result<(), AppError> {
        let done = sqlx::query!(
            "UPDATE archive_assets SET format_issue = $2 WHERE id = $1",
            asset_id.into_uuid(),
            issue.as_str(),
        )
        .execute(&self.pool)
        .await
        .map_err(repo_error)?;
        exactly_one_row(done.rows_affected(), asset_id)
    }

    async fn with_document_lock<'a>(
        &'a self,
        document_id: DocumentId,
        work: BoxFuture<'a, Result<(), AppError>>,
    ) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await.map_err(repo_error)?;
        sqlx::query!(
            "SELECT pg_advisory_xact_lock(hashtextextended($1, 0))",
            format!("epub-toc:{document_id}"),
        )
        .execute(&mut *tx)
        .await
        .map_err(repo_error)?;
        work.await?;
        tx.commit().await.map_err(repo_error)
    }

    async fn try_global_backfill_lock<'a>(
        &'a self,
        work: BoxFuture<'a, Result<(), AppError>>,
    ) -> Result<bool, AppError> {
        // Detached so the session, and with it the lock, ends when this connection closes or
        // drops, instead of returning to the pool still holding the lock.
        let mut conn = self.pool.acquire().await.map_err(repo_error)?.detach();
        let acquired = sqlx::query_scalar!(
            r#"SELECT pg_try_advisory_lock(hashtextextended($1, 0)) AS "acquired!""#,
            BACKFILL_LOCK,
        )
        .fetch_one(&mut conn)
        .await
        .map_err(repo_error)?;
        let result = if acquired { work.await } else { Ok(()) };
        let closed = sqlx::Connection::close(conn).await.map_err(repo_error);
        result.and(closed).map(|()| acquired)
    }
}

fn exactly_one_row(rows_affected: u64, asset_id: ArchiveAssetId) -> Result<(), AppError> {
    if rows_affected == 1 {
        return Ok(());
    }
    Err(AppError::Domain(DomainError::NotFound {
        entity: "ArchiveAsset",
        id: asset_id.to_string(),
    }))
}
