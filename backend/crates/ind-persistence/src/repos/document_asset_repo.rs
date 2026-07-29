use std::str::FromStr;

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::document_asset::{DocumentAssetRepository, PreparedReadableLocation};
use ind_domain::{
    ArchiveAssetId, ArchiveAssetKind, ArchiveAssetStatus, DocumentAsset, DocumentId, DomainError,
    NewDocumentAsset,
};

/// Document-keyed archive assets. Same `archive_assets` table as the legacy item-keyed
/// repository, addressed by `document_id` (item_id stays NULL for these rows). See
/// docs/document-feed-library-architecture.md (Archive assets and rendered content).
pub struct PgDocumentAssetRepository {
    pool: PgPool,
}

impl PgDocumentAssetRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct DocumentAssetRow {
    id: Uuid,
    document_id: Uuid,
    asset_kind: String,
    s3_key: String,
    s3_bucket: String,
    content_type: String,
    size_bytes: i64,
    status: String,
    failed_reason: Option<String>,
    created_at: DateTime<Utc>,
}

impl TryFrom<DocumentAssetRow> for DocumentAsset {
    type Error = AppError;

    fn try_from(row: DocumentAssetRow) -> Result<Self, Self::Error> {
        Ok(DocumentAsset {
            id: ArchiveAssetId::from_uuid(row.id),
            document_id: DocumentId::from_uuid(row.document_id),
            asset_kind: ArchiveAssetKind::from_str(&row.asset_kind)
                .map_err(|message| AppError::Domain(DomainError::InvariantViolation { message }))?,
            s3_key: row.s3_key,
            s3_bucket: row.s3_bucket,
            content_type: row.content_type,
            size_bytes: row.size_bytes,
            status: parse_asset_status(&row.status),
            failed_reason: row.failed_reason,
            created_at: row.created_at,
        })
    }
}

fn parse_asset_status(s: &str) -> ArchiveAssetStatus {
    match s {
        "pending" => ArchiveAssetStatus::Pending,
        "completed" => ArchiveAssetStatus::Completed,
        "degraded" => ArchiveAssetStatus::Degraded,
        "failed" => ArchiveAssetStatus::Failed,
        "unsupported" => ArchiveAssetStatus::Unsupported,
        _ => ArchiveAssetStatus::Completed,
    }
}

fn asset_status_to_str(status: ArchiveAssetStatus) -> &'static str {
    match status {
        ArchiveAssetStatus::Pending => "pending",
        ArchiveAssetStatus::Completed => "completed",
        ArchiveAssetStatus::Degraded => "degraded",
        ArchiveAssetStatus::Failed => "failed",
        ArchiveAssetStatus::Unsupported => "unsupported",
    }
}

fn map_error(err: sqlx::Error) -> AppError {
    super::map_sqlx_error("document_asset", "duplicate asset kind for document", err)
}

#[async_trait::async_trait]
impl DocumentAssetRepository for PgDocumentAssetRepository {
    async fn upsert_document_asset(
        &self,
        asset: NewDocumentAsset,
    ) -> Result<DocumentAsset, AppError> {
        // Conflict target carries the partial-index predicate (document_id IS NOT NULL); inserts
        // always set document_id, so a re-render converges onto the existing row for the kind.
        let row = sqlx::query_as!(
            DocumentAssetRow,
            "WITH upserted AS ( \
             INSERT INTO archive_assets \
                 (id, document_id, asset_kind, s3_key, s3_bucket, content_type, size_bytes, \
                  status, failed_reason, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, now()) \
             ON CONFLICT (document_id, asset_kind) WHERE document_id IS NOT NULL DO UPDATE SET \
                 s3_key = EXCLUDED.s3_key, s3_bucket = EXCLUDED.s3_bucket, \
                 content_type = EXCLUDED.content_type, size_bytes = EXCLUDED.size_bytes, \
                 status = EXCLUDED.status, failed_reason = EXCLUDED.failed_reason, \
                 created_at = now() \
             WHERE archive_assets.status <> 'completed' OR EXCLUDED.status = 'completed' \
             RETURNING id, document_id, asset_kind, s3_key, s3_bucket, \
                       content_type, size_bytes, status, failed_reason, created_at \
             ) \
             SELECT id AS \"id!\", document_id AS \"document_id!\", \
                    asset_kind AS \"asset_kind!\", s3_key AS \"s3_key!\", \
                    s3_bucket AS \"s3_bucket!\", content_type AS \"content_type!\", \
                    size_bytes AS \"size_bytes!\", status AS \"status!\", failed_reason, \
                    created_at AS \"created_at!\" \
             FROM upserted \
             UNION ALL \
             SELECT id, document_id AS \"document_id!\", asset_kind, s3_key, s3_bucket, \
                    content_type, size_bytes, status, failed_reason, created_at \
             FROM archive_assets \
             WHERE document_id = $2 AND asset_kind = $3 AND NOT EXISTS (SELECT 1 FROM upserted) \
             LIMIT 1",
            ArchiveAssetId::new().into_uuid(),
            asset.document_id.into_uuid(),
            asset.asset_kind.to_string(),
            &asset.s3_key,
            &asset.s3_bucket,
            &asset.content_type,
            asset.size_bytes,
            asset_status_to_str(asset.status),
            asset.failed_reason.as_deref(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_error)?;

        DocumentAsset::try_from(row)
    }

    async fn find_by_document(
        &self,
        document_id: DocumentId,
    ) -> Result<Vec<DocumentAsset>, AppError> {
        let rows = sqlx::query_as!(
            DocumentAssetRow,
            "SELECT id, document_id AS \"document_id!\", asset_kind, s3_key, s3_bucket, \
                    content_type, size_bytes, status, failed_reason, created_at \
             FROM archive_assets \
             WHERE document_id = $1 \
             ORDER BY created_at ASC",
            document_id.into_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_error)?;

        rows.into_iter()
            .map(DocumentAsset::try_from)
            .collect::<Result<Vec<_>, _>>()
    }

    async fn find_by_document_and_kind(
        &self,
        document_id: DocumentId,
        kind: ArchiveAssetKind,
    ) -> Result<Option<DocumentAsset>, AppError> {
        let row = sqlx::query_as!(
            DocumentAssetRow,
            "SELECT id, document_id AS \"document_id!\", asset_kind, s3_key, s3_bucket, \
                    content_type, size_bytes, status, failed_reason, created_at \
             FROM archive_assets \
             WHERE document_id = $1 AND asset_kind = $2",
            document_id.into_uuid(),
            kind.to_string(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_error)?;

        row.map(DocumentAsset::try_from).transpose()
    }

    async fn has_successful_asset(
        &self,
        document_id: DocumentId,
        kind: ArchiveAssetKind,
    ) -> Result<bool, AppError> {
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS ( \
                 SELECT 1 FROM archive_assets \
                 WHERE document_id = $1 AND asset_kind = $2 AND status = 'completed' \
             ) AS \"exists!\"",
            document_id.into_uuid(),
            kind.to_string(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_error)?;

        Ok(exists)
    }

    async fn commit_article_toc(
        &self,
        document_id: DocumentId,
        expected_readable_created_at: DateTime<Utc>,
        new_readable_location: Option<PreparedReadableLocation>,
        toc_asset: NewDocumentAsset,
    ) -> Result<bool, AppError> {
        let mut tx = self.pool.begin().await.map_err(map_error)?;

        // Version guard: the readable row must still carry the created_at this
        // ToC was derived against. The swap keeps created_at untouched — it
        // changes representation (anchored copy), not content version, and the
        // already-uploaded ToC payload records that stamp as its source.
        let guard_held = match new_readable_location {
            Some(location) => {
                let result = sqlx::query!(
                    "UPDATE archive_assets \
                     SET s3_key = $1, size_bytes = $2 \
                     WHERE document_id = $3 AND asset_kind = 'readable_html' \
                       AND created_at = $4",
                    location.s3_key,
                    location.size_bytes,
                    document_id.into_uuid(),
                    expected_readable_created_at,
                )
                .execute(&mut *tx)
                .await
                .map_err(map_error)?;
                result.rows_affected() == 1
            }
            None => sqlx::query_scalar!(
                "SELECT EXISTS ( \
                     SELECT 1 FROM archive_assets \
                     WHERE document_id = $1 AND asset_kind = 'readable_html' \
                       AND created_at = $2 \
                 ) AS \"exists!\"",
                document_id.into_uuid(),
                expected_readable_created_at,
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(map_error)?,
        };

        if !guard_held {
            // Dropping the transaction rolls back; a reprocess won the race and
            // its own ingest recomputes the ToC.
            return Ok(false);
        }

        sqlx::query!(
            "INSERT INTO archive_assets \
                 (id, document_id, asset_kind, s3_key, s3_bucket, content_type, size_bytes, \
                  status, failed_reason, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, now()) \
             ON CONFLICT (document_id, asset_kind) WHERE document_id IS NOT NULL DO UPDATE SET \
                 s3_key = EXCLUDED.s3_key, s3_bucket = EXCLUDED.s3_bucket, \
                 content_type = EXCLUDED.content_type, size_bytes = EXCLUDED.size_bytes, \
                 status = EXCLUDED.status, failed_reason = EXCLUDED.failed_reason, \
                 created_at = now()",
            ArchiveAssetId::new().into_uuid(),
            toc_asset.document_id.into_uuid(),
            toc_asset.asset_kind.to_string(),
            &toc_asset.s3_key,
            &toc_asset.s3_bucket,
            &toc_asset.content_type,
            toc_asset.size_bytes,
            asset_status_to_str(toc_asset.status),
            toc_asset.failed_reason.as_deref(),
        )
        .execute(&mut *tx)
        .await
        .map_err(map_error)?;

        tx.commit().await.map_err(map_error)?;
        Ok(true)
    }
}
