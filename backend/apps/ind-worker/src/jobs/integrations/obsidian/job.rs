use std::sync::Arc;

use chrono::Utc;
use ind_application::AppError;
use ind_domain::{DocumentId, DomainError, HighlightId, ObsidianSyncConnectionJob};
use ind_ingest::AssetBackedPreparedContentProvider;
use ind_integrations::obsidian::{
    ObsidianRenderCursor, SERVER_BASE_FOLDER, render_document, render_sync_notification,
    settings_from_config,
};
use ind_persistence::repos::PgObsidianExportRepository;

use super::artifact::{
    ObsidianArtifactManifest, ObsidianSyncNotificationArtifact, artifact_item_inserts_from_entries,
    build_zip_artifact,
};
use super::document::build_render_document;
use super::paths::{PendingObsidianEntry, resolve_artifact_paths};
use crate::context::IntegrationJobDeps;

pub async fn handle_sync_connection(
    ctx: &IntegrationJobDeps,
    job: ObsidianSyncConnectionJob,
) -> Result<(), AppError> {
    let obsidian_export_repo = PgObsidianExportRepository::new(ctx.pool.clone());
    let run_id = obsidian_export_repo
        .ensure_sync_run(
            job.user_id,
            job.connection_id,
            job.requested_by_user,
            job.run_id,
        )
        .await?;
    let result = build_run(ctx, &obsidian_export_repo, &job, run_id).await;
    if let Err(err) = result {
        // Only flip to failed if this worker is still the active owner
        // of the run. A concurrent dispatch that already marked it
        // terminal (success / failed / cancelled) wins; don't overwrite.
        obsidian_export_repo
            .mark_sync_run_failed(run_id, &err.to_string())
            .await?;
        return Err(err);
    }
    Ok(())
}

async fn build_run(
    ctx: &IntegrationJobDeps,
    obsidian_export_repo: &PgObsidianExportRepository,
    job: &ObsidianSyncConnectionJob,
    run_id: uuid::Uuid,
) -> Result<(), AppError> {
    let connection = obsidian_export_repo
        .load_sync_connection(job.user_id, job.connection_id)
        .await?;
    if connection.provider != "obsidian" {
        return Err(AppError::Domain(DomainError::Validation {
            field: "provider".to_string(),
            message: "connection is not an Obsidian integration".to_string(),
        }));
    }
    if !matches!(connection.status.as_str(), "pending" | "active") {
        return Err(AppError::Domain(DomainError::Validation {
            field: "status".to_string(),
            message: "Obsidian integration is inactive".to_string(),
        }));
    }
    let settings = settings_from_config(&connection.config);
    let run = obsidian_export_repo.load_sync_run(run_id).await?;

    // Claim the run only if it is still claimable (pending or already
    // running). If a parallel dispatch marked the run terminal between
    // job.dispatched_at and here, bail without rebuilding; re-running
    // the export would corrupt the existing artifact and double-bump
    // cursors.
    if !obsidian_export_repo.claim_sync_run(run_id).await? {
        tracing::warn!(
            run_id = %run_id,
            connection_id = %job.connection_id,
            "obsidian export run is already terminal; skipping rebuild"
        );
        return Ok(());
    }

    let candidates = obsidian_export_repo
        .list_sync_candidates(
            job.user_id,
            job.connection_id,
            &run,
            settings.export_all_reader_documents,
        )
        .await?;

    let full_content_provider = settings.export_all_reader_documents.then(|| {
        Arc::new(AssetBackedPreparedContentProvider::new(
            ctx.document_repo.clone(),
            ctx.document_asset_repo.clone(),
            ctx.mila_config_repo.clone(),
            ctx.object_storage.clone(),
        ))
    });

    let highlight_repo = ctx
        .highlight_repo
        .as_ref()
        .ok_or_else(|| AppError::ExternalService {
            service: "obsidian".to_string(),
            message: "highlight repository is not configured for Obsidian export".to_string(),
        })?;

    let now = Utc::now();
    let mut pending_entries = Vec::new();
    for candidate in &candidates {
        let document_id = DocumentId::from_uuid(candidate.document_id);
        let Some(document) = ctx
            .document_repo
            .find_by_id(job.user_id, document_id)
            .await?
        else {
            continue;
        };
        let Some(entry) = ctx
            .library_repo
            .find_active_by_document(job.user_id, document_id)
            .await?
        else {
            continue;
        };
        let doc = build_render_document(
            ctx,
            highlight_repo.as_ref(),
            full_content_provider.as_deref(),
            job.user_id,
            &document,
            &entry,
            &settings,
        )
        .await?;
        if settings.export_all_reader_documents
            && doc.highlights.is_empty()
            && doc.full_document_text.is_none()
        {
            continue;
        }
        let cursor = ObsidianRenderCursor {
            has_delivered: candidate.last_delivered_at.is_some(),
            last_highlight_created_at: candidate.last_exported_highlight_created_at,
            last_highlight_id: candidate
                .last_exported_highlight_id
                .map(|id| HighlightId::from_uuid(id).to_string()),
            force_full: candidate.force_full,
            last_content_hash: candidate.last_exported_file_hash.clone(),
            last_full_document_hash: candidate.last_exported_full_document_hash.clone(),
            generated_path: candidate.generated_path.clone(),
            generated_full_document_path: candidate.generated_full_document_path.clone(),
        };
        if let Some(rendered) = render_document(&settings, &doc, &cursor, now).map_err(|e| {
            AppError::Domain(DomainError::Validation {
                field: "template".to_string(),
                message: e.to_string(),
            })
        })? {
            pending_entries.push(PendingObsidianEntry {
                entry: rendered.entry,
                generated_path_locked: candidate.generated_path.is_some(),
                generated_full_document_path_locked: candidate
                    .generated_full_document_path
                    .is_some(),
            });
        }
    }

    let entries = resolve_artifact_paths(pending_entries)?;

    let sync_notification =
        render_sync_notification(&settings, entries.len(), now).map_err(|e| {
            AppError::Domain(DomainError::Validation {
                field: "sync_notification_template".to_string(),
                message: e.to_string(),
            })
        })?;
    let sync_notification =
        (!sync_notification.trim().is_empty()).then(|| ObsidianSyncNotificationArtifact {
            file_path: format!("{SERVER_BASE_FOLDER}/Indelible Syncs.md"),
            append_content: sync_notification,
        });

    if entries.is_empty() && sync_notification.is_none() {
        obsidian_export_repo
            .mark_sync_run_succeeded(run_id, candidates.len() as i32, 0)
            .await?;
        return Ok(());
    }

    let manifest = ObsidianArtifactManifest {
        version: 1,
        run_id,
        generated_at: now,
        entries,
        sync_notification,
    };
    let zip_bytes = build_zip_artifact(&manifest)?;
    let artifact_items = artifact_item_inserts_from_entries(&manifest.entries)?;
    obsidian_export_repo
        .store_sync_artifact(
            run_id,
            job.connection_id,
            job.user_id,
            &zip_bytes,
            &artifact_items,
        )
        .await?;

    obsidian_export_repo
        .mark_sync_run_artifact_ready(run_id, candidates.len() as i32)
        .await?;
    Ok(())
}
