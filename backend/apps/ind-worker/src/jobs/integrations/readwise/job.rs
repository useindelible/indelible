use std::collections::HashMap;

use ind_application::AppError;
use ind_application::handlers::feed::FeedService;
use ind_domain::{DomainError, ImportJobCountsDelta, ImportJobStatus};
use tracing::{info, warn};

use super::assets::asset_count_for_extension;
use super::download::{download_bytes, download_zip};
use super::parse::parse_csv;
use super::rows::{process_csv_row, process_zip_only_entry};
use super::types::{
    ArtifactKeys, ProcessCsvOutcome, ProcessRowResult, ReadwiseCsvRow, ReadwiseImportJob,
    ReadwiseReport, ZipEntry,
};
use crate::context::IntegrationJobDeps;

pub async fn handle_readwise_import(
    ctx: &IntegrationJobDeps,
    job: ReadwiseImportJob,
) -> Result<(), AppError> {
    let import_job = ctx
        .import_job_repo
        .find_by_id_unchecked(job.import_job_id)
        .await?
        .ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "ImportJob",
                id: job.import_job_id.to_string(),
            })
        })?;

    let user_id = import_job.user_id;
    let job_id = import_job.id;

    let artifact_keys: ArtifactKeys = import_job
        .raw_artifact_key
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or(ArtifactKeys {
            csv_key: None,
            zip_key: None,
            opml_key: None,
        });

    let Some(storage_arc) = ctx.object_storage.as_ref() else {
        let message = "object storage is not configured; Readwise import requires S3".to_string();
        ctx.import_job_repo.mark_started(job_id).await?;
        ctx.import_job_repo
            .mark_finished(job_id, ImportJobStatus::Failed, Some(message))
            .await?;
        return Ok(());
    };
    let storage: &dyn ind_application::storage::ObjectStorage = storage_arc.as_ref();

    ctx.import_job_repo.mark_started(job_id).await?;

    let mut report = ReadwiseReport {
        csv_rows: 0,
        highlight_rows: 0,
        reading_progress_rows: 0,
        zip_files_total: 0,
        zip_files_matched: 0,
        zip_files_unmatched: 0,
        unmatched_zip_assets: vec![],
        archive_assets_imported: 0,
        search_reindex_jobs_enqueued: 0,
        embedding_jobs_enqueued: 0,
        opml_feeds_created: 0,
        opml_feeds_skipped: 0,
        opml_errors: vec![],
    };

    let mut any_success = false;
    let mut any_failure = false;

    if artifact_keys.csv_key.is_none()
        && artifact_keys.zip_key.is_none()
        && artifact_keys.opml_key.is_none()
    {
        any_failure = true;
        warn!("Readwise import job has no artifact keys");
    }

    // Download ZIP and build ULID -> entry map before processing CSV rows.
    let zip_map: HashMap<String, ZipEntry> = if let Some(zip_key) = &artifact_keys.zip_key {
        match download_zip(ctx, storage, zip_key).await {
            Ok(m) => m,
            Err(e) => {
                any_failure = true;
                warn!(error = %e, "failed to download Readwise ZIP; continuing without it");
                HashMap::new()
            }
        }
    } else {
        HashMap::new()
    };
    report.zip_files_total = zip_map.len() as u32;

    let mut csv_rows: Vec<ReadwiseCsvRow> = Vec::new();
    if let Some(csv_key) = &artifact_keys.csv_key {
        match download_bytes(storage, csv_key).await {
            Ok(bytes) => match parse_csv(&bytes) {
                Ok(rows) => csv_rows = rows,
                Err(e) => {
                    any_failure = true;
                    warn!(error = %e, "failed to parse Readwise CSV");
                }
            },
            Err(e) => {
                any_failure = true;
                warn!(error = %e, "failed to download Readwise CSV");
            }
        }
    }

    report.csv_rows = csv_rows.len() as u32;
    report.reading_progress_rows = csv_rows
        .iter()
        .filter(|row| row.reading_progress > 0.0 || row.seen)
        .count() as u32;

    // Track which ZIP ULIDs are matched so we can handle unmatched entries later.
    let mut matched_zip_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

    for row in &csv_rows {
        match process_csv_row(ctx, storage, user_id, row, &zip_map, &mut matched_zip_ids).await {
            Ok(ProcessCsvOutcome::Imported {
                search_reindex_jobs,
                embedding_jobs,
                diagnostics,
            }) => {
                any_success = true;
                if let Some(entry) = zip_map.get(&row.id) {
                    report.archive_assets_imported += asset_count_for_extension(&entry.extension);
                }
                report.search_reindex_jobs_enqueued += search_reindex_jobs;
                report.embedding_jobs_enqueued += embedding_jobs;
                ctx.import_job_repo
                    .increment_counts(
                        job_id,
                        ImportJobCountsDelta {
                            imported: 1,
                            updated: 0,
                            duplicate: 0,
                            skipped_private: 0,
                            failed: 0,
                        },
                    )
                    .await?;
                ctx.import_job_repo
                    .append_item_outcome(
                        job_id,
                        &row.id,
                        ind_domain::ImportItemOutcome::Imported,
                        None,
                        diagnostics,
                    )
                    .await
                    .ok();
            }
            Ok(ProcessCsvOutcome::Duplicate {
                search_reindex_jobs,
                embedding_jobs,
                diagnostics,
            }) => {
                report.search_reindex_jobs_enqueued += search_reindex_jobs;
                report.embedding_jobs_enqueued += embedding_jobs;
                ctx.import_job_repo
                    .increment_counts(
                        job_id,
                        ImportJobCountsDelta {
                            imported: 0,
                            updated: 0,
                            duplicate: 1,
                            skipped_private: 0,
                            failed: 0,
                        },
                    )
                    .await?;
                ctx.import_job_repo
                    .append_item_outcome(
                        job_id,
                        &row.id,
                        ind_domain::ImportItemOutcome::Duplicate,
                        None,
                        diagnostics,
                    )
                    .await
                    .ok();
            }
            Ok(ProcessCsvOutcome::SkippedPrivate) => {
                ctx.import_job_repo
                    .increment_counts(
                        job_id,
                        ImportJobCountsDelta {
                            imported: 0,
                            updated: 0,
                            duplicate: 0,
                            skipped_private: 1,
                            failed: 0,
                        },
                    )
                    .await?;
                ctx.import_job_repo
                    .append_item_outcome(
                        job_id,
                        &row.id,
                        ind_domain::ImportItemOutcome::SkippedPrivate,
                        None,
                        None,
                    )
                    .await
                    .ok();
            }
            Err(ProcessRowResult::Failed(e)) => {
                any_failure = true;
                warn!(external_id = %row.id, error = %e, "Readwise CSV row failed");
                ctx.import_job_repo
                    .increment_counts(
                        job_id,
                        ImportJobCountsDelta {
                            imported: 0,
                            updated: 0,
                            duplicate: 0,
                            skipped_private: 0,
                            failed: 1,
                        },
                    )
                    .await?;
                ctx.import_job_repo
                    .append_item_outcome(
                        job_id,
                        &row.id,
                        ind_domain::ImportItemOutcome::Failed,
                        Some(e.to_string()),
                        None,
                    )
                    .await
                    .ok();
            }
        }
    }

    // Handle ZIP-only entries (no CSV match).
    for (ulid, entry) in &zip_map {
        if matched_zip_ids.contains(ulid) {
            continue;
        }
        report.zip_files_unmatched += 1;
        report.unmatched_zip_assets.push(entry.path.clone());
        let ext_key = format!("zip_only_{ulid}");
        match process_zip_only_entry(ctx, storage, user_id, ulid, entry).await {
            Ok(ProcessCsvOutcome::Imported {
                search_reindex_jobs,
                embedding_jobs,
                diagnostics,
            }) => {
                any_success = true;
                report.archive_assets_imported += asset_count_for_extension(&entry.extension);
                report.search_reindex_jobs_enqueued += search_reindex_jobs;
                report.embedding_jobs_enqueued += embedding_jobs;
                ctx.import_job_repo
                    .increment_counts(
                        job_id,
                        ImportJobCountsDelta {
                            imported: 1,
                            updated: 0,
                            duplicate: 0,
                            skipped_private: 0,
                            failed: 0,
                        },
                    )
                    .await?;
                ctx.import_job_repo
                    .append_item_outcome(
                        job_id,
                        &ext_key,
                        ind_domain::ImportItemOutcome::Imported,
                        None,
                        diagnostics,
                    )
                    .await
                    .ok();
            }
            Ok(ProcessCsvOutcome::Duplicate {
                search_reindex_jobs,
                embedding_jobs,
                diagnostics,
            }) => {
                report.search_reindex_jobs_enqueued += search_reindex_jobs;
                report.embedding_jobs_enqueued += embedding_jobs;
                ctx.import_job_repo
                    .increment_counts(
                        job_id,
                        ImportJobCountsDelta {
                            imported: 0,
                            updated: 0,
                            duplicate: 1,
                            skipped_private: 0,
                            failed: 0,
                        },
                    )
                    .await?;
                ctx.import_job_repo
                    .append_item_outcome(
                        job_id,
                        &ext_key,
                        ind_domain::ImportItemOutcome::Duplicate,
                        None,
                        diagnostics,
                    )
                    .await
                    .ok();
            }
            Ok(ProcessCsvOutcome::SkippedPrivate) => {}
            Err(e) => {
                any_failure = true;
                warn!(ulid = %ulid, error = %e, "Readwise ZIP-only entry failed");
                ctx.import_job_repo
                    .increment_counts(
                        job_id,
                        ImportJobCountsDelta {
                            imported: 0,
                            updated: 0,
                            duplicate: 0,
                            skipped_private: 0,
                            failed: 1,
                        },
                    )
                    .await?;
                ctx.import_job_repo
                    .append_item_outcome(
                        job_id,
                        &ext_key,
                        ind_domain::ImportItemOutcome::Failed,
                        Some(e.to_string()),
                        None,
                    )
                    .await
                    .ok();
            }
        }
    }

    // Update ZIP counts in report.
    report.zip_files_matched = matched_zip_ids.len() as u32;

    // OPML import.
    if let Some(opml_key) = &artifact_keys.opml_key {
        match download_bytes(storage, opml_key).await {
            Ok(bytes) => {
                let opml_str = String::from_utf8_lossy(&bytes);
                #[expect(
                    clippy::expect_used,
                    reason = "guarded HTTP fetcher builds from a valid static egress policy; construction is infallible"
                )]
                let feed_service = FeedService::new(
                    ctx.feed_repo.clone(),
                    ctx.outbox_repo.clone(),
                    std::sync::Arc::new(
                        ind_ingest::ReqwestHttpFetcher::with_policy(ctx.egress_policy.clone())
                            .expect("ingest guarded client builds"),
                    ),
                    std::sync::Arc::new(ind_ingest::FeedRsFeedParser::new()),
                    std::sync::Arc::new(ind_ingest::QuickXmlOpmlParser::new()),
                );
                match feed_service.import_opml(user_id, &opml_str).await {
                    Ok(result) => {
                        report.opml_feeds_created = result.created;
                        report.opml_feeds_skipped = result.skipped;
                        report.opml_errors = result.errors;
                        if !report.opml_errors.is_empty() {
                            any_failure = true;
                        }
                        if report.opml_feeds_created > 0 || report.opml_feeds_skipped > 0 {
                            any_success = true;
                        }
                    }
                    Err(e) => {
                        any_failure = true;
                        warn!(error = %e, "OPML import failed");
                        report.opml_errors.push(e.to_string());
                    }
                }
            }
            Err(e) => {
                any_failure = true;
                warn!(error = %e, "failed to download OPML");
                report.opml_errors.push(e.to_string());
            }
        }
    }

    let provider_report = serde_json::to_value(&report).unwrap_or(serde_json::Value::Null);
    ctx.import_job_repo
        .set_provider_report(job_id, provider_report)
        .await?;

    let final_status = match (any_success, any_failure) {
        (true, true) => ImportJobStatus::Partial,
        (true, false) => ImportJobStatus::Completed,
        (false, true) => ImportJobStatus::Failed,
        (false, false) => ImportJobStatus::Completed,
    };

    ctx.import_job_repo
        .mark_finished(job_id, final_status, None)
        .await?;

    info!(
        job_id = %job_id,
        user_id = %user_id,
        csv_rows = report.csv_rows,
        zip_matched = report.zip_files_matched,
        zip_unmatched = report.zip_files_unmatched,
        opml_created = report.opml_feeds_created,
        "Readwise import completed"
    );

    Ok(())
}
