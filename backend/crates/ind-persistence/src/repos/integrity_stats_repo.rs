use async_trait::async_trait;
use sqlx::PgPool;

use ind_application::error::AppError;
use ind_application::repos::integrity::{IntegrityStats, IntegrityStatsRepository};

pub struct PgIntegrityStatsRepository {
    pool: PgPool,
}

impl PgIntegrityStatsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IntegrityStatsRepository for PgIntegrityStatsRepository {
    async fn stats(&self) -> Result<IntegrityStats, AppError> {
        let row = sqlx::query_as!(
            IntegrityStatsRow,
            r#"
            SELECT
                (
                    SELECT COUNT(DISTINCT d.id)::BIGINT
                    FROM documents d
                    WHERE (
                            EXISTS (
                                SELECT 1
                                FROM library_entries le
                                WHERE le.document_id = d.id
                                  AND le.deleted_at IS NULL
                            )
                            OR EXISTS (
                                SELECT 1
                                FROM feed_deliveries fd
                                WHERE fd.document_id = d.id
                                  AND fd.dismissed_at IS NULL
                                  AND fd.hidden_at IS NULL
                            )
                        )
                      AND NOT EXISTS (
                            SELECT 1
                            FROM search_documents sd
                            WHERE sd.document_id = d.id
                        )
                ) AS documents_missing_search_rows,
                (
                    SELECT COUNT(DISTINCT d.id)::BIGINT
                    FROM documents d
                    WHERE d.user_id IS NOT NULL
                      AND EXISTS (
                            SELECT 1
                            FROM archive_assets aa
                            WHERE aa.document_id = d.id
                              AND aa.status = 'completed'
                              AND aa.s3_key <> ''
                              AND (
                                    (aa.asset_kind = 'readable_html' AND aa.content_type = 'text/html')
                                    OR (aa.asset_kind = 'epub' AND aa.content_type = 'application/json')
                                    OR (aa.asset_kind = 'original_upload' AND aa.content_type = 'application/pdf')
                              )
                        )
                      AND NOT EXISTS (
                            SELECT 1
                            FROM content_vectors cv
                            WHERE cv.document_id = d.id
                        )
                      AND NOT EXISTS (
                            SELECT 1
                            FROM job_outbox jo
                            WHERE jo.dedupe_key = ('document.ai.embed:doc_' || d.id::text)
                              AND jo.dispatched_at IS NULL
                        )
                ) AS documents_missing_vectors,
                (
                    SELECT COUNT(*)::BIGINT
                    FROM archive_assets aa
                    WHERE aa.document_id IS NOT NULL
                      AND aa.status IN ('failed', 'degraded')
                      AND aa.asset_kind IN (
                            'readable_html',
                            'monolith',
                            'pdf',
                            'screenshot',
                            'thumbnail',
                            'warc',
                            'epub',
                            'extracted_text'
                      )
                ) AS failed_derived_assets,
                (
                    SELECT COUNT(*)::BIGINT
                    FROM dead_letter_jobs
                    WHERE replayed_at IS NULL
                ) AS dead_letter_jobs
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| AppError::Repository(Box::new(err)))?;

        Ok(IntegrityStats {
            documents_missing_search_rows: row.documents_missing_search_rows.unwrap_or(0),
            documents_missing_vectors: row.documents_missing_vectors.unwrap_or(0),
            failed_derived_assets: row.failed_derived_assets.unwrap_or(0),
            dead_letter_jobs: row.dead_letter_jobs.unwrap_or(0),
        })
    }
}

struct IntegrityStatsRow {
    documents_missing_search_rows: Option<i64>,
    documents_missing_vectors: Option<i64>,
    failed_derived_assets: Option<i64>,
    dead_letter_jobs: Option<i64>,
}
