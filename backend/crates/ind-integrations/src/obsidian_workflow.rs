use std::sync::Arc;

use ind_application::AppError;
use ind_application::export_summary::ExportSummaryProvider;
use ind_application::outputs::export::{
    ObsidianArtifactDownload, ObsidianExportPreview, ObsidianRefreshResult, ObsidianRunStatus,
};
use ind_application::ports::{ObsidianAckSubject, ObsidianRunAck, ObsidianRunCreate};
use ind_application::repos::export_cursor::ExportCursorRepository;
use ind_application::repos::integration_connection::IntegrationConnectionRepository;
use ind_application::repos::obsidian_export::{
    AckObsidianRunInput, CreateObsidianRunInput, ObsidianAckSubjectRecord,
    ObsidianArtifactDownloadRecord, ObsidianExportRepository, ObsidianRunStatusRecord,
};
use ind_application::repos::obsidian_preview::ObsidianPreviewRepository;
use ind_application::repos::outbox::JobOutboxRepository;
use ind_application::repos::prepared_content::PreparedContentProvider;
use ind_domain::{
    DomainError, IntegrationConnectionId, IntegrationProvider, LibraryEntryId,
    ObsidianExportSettings, ObsidianSyncConnectionJob, UserId, job_types,
};

pub struct ObsidianPreviewRenderer {
    preview_repo: Arc<dyn ObsidianPreviewRepository>,
    export_summary_provider: Arc<dyn ExportSummaryProvider>,
    prepared_content_provider: Arc<dyn PreparedContentProvider>,
}

impl ObsidianPreviewRenderer {
    pub fn new(
        preview_repo: Arc<dyn ObsidianPreviewRepository>,
        export_summary_provider: Arc<dyn ExportSummaryProvider>,
        prepared_content_provider: Arc<dyn PreparedContentProvider>,
    ) -> Self {
        Self {
            preview_repo,
            export_summary_provider,
            prepared_content_provider,
        }
    }

    pub async fn preview(
        &self,
        user_id: UserId,
        library_entry_id: Option<LibraryEntryId>,
        settings: ObsidianExportSettings,
    ) -> Result<ObsidianExportPreview, AppError> {
        let document = if let Some(library_entry_id) = library_entry_id {
            self.load_document(user_id, library_entry_id).await?
        } else {
            sample_preview_document()
        };

        let rendered = crate::obsidian::render_document(
            &settings,
            &document,
            &crate::obsidian::ObsidianRenderCursor::default(),
            chrono::Utc::now(),
        )
        .map_err(|e| {
            AppError::Domain(DomainError::Validation {
                field: "template".into(),
                message: e.to_string(),
            })
        })?
        .ok_or_else(|| {
            AppError::Domain(DomainError::Validation {
                field: "library_entry".into(),
                message: "preview document produced no export output".into(),
            })
        })?;

        Ok(ObsidianExportPreview {
            file_path: rendered.entry.file_path,
            full_content: rendered.entry.full_content.unwrap_or_default(),
            append_only_content: rendered.entry.append_only_content,
            full_document_text_path: rendered.entry.full_document_text_path,
            full_document_text: rendered.entry.full_document_text,
        })
    }

    async fn load_document(
        &self,
        user_id: UserId,
        library_entry_id: LibraryEntryId,
    ) -> Result<crate::obsidian::ObsidianRenderDocument, AppError> {
        let Some(document) = self
            .preview_repo
            .load_document(user_id, library_entry_id)
            .await?
        else {
            return Err(AppError::Domain(DomainError::NotFound {
                entity: "library_entry",
                id: library_entry_id.to_string(),
            }));
        };
        let summary = self
            .export_summary_provider
            .summary_for_document(document.document_id, document.excerpt.as_deref())
            .await?;
        let full_document_text = self
            .prepared_content_provider
            .load_for_document(document.document_id)
            .await?
            .map(|content| content.root_text)
            .filter(|text| !text.trim().is_empty());

        Ok(crate::obsidian::ObsidianRenderDocument {
            subject_id: document.library_entry_id.to_string(),
            subject_kind: ind_application::repos::export_subject::ExportSubjectKind::LibraryEntry
                .as_str()
                .to_string(),
            title: document.title.clone(),
            full_title: document.title,
            url: document.url,
            author: document.author,
            item_type: document.item_type,
            image_url: document.lead_image_url,
            summary,
            full_document_text,
            document_tags: document.tags,
            highlights: document
                .highlights
                .into_iter()
                .map(|h| crate::obsidian::ObsidianRenderHighlight {
                    id: h.id.to_string(),
                    text: h.text,
                    note: h.note,
                    color: h.color,
                    tags: h.tags,
                    location: None,
                    location_url: None,
                    created_at: h.created_at,
                })
                .collect(),
        })
    }
}

pub struct ObsidianRunWorkflow {
    connection_repo: Arc<dyn IntegrationConnectionRepository>,
    outbox_repo: Arc<dyn JobOutboxRepository>,
    export_cursor_repo: Arc<dyn ExportCursorRepository>,
    obsidian_export_repo: Arc<dyn ObsidianExportRepository>,
}

impl ObsidianRunWorkflow {
    pub fn new(
        connection_repo: Arc<dyn IntegrationConnectionRepository>,
        outbox_repo: Arc<dyn JobOutboxRepository>,
        export_cursor_repo: Arc<dyn ExportCursorRepository>,
        obsidian_export_repo: Arc<dyn ObsidianExportRepository>,
    ) -> Self {
        Self {
            connection_repo,
            outbox_repo,
            export_cursor_repo,
            obsidian_export_repo,
        }
    }

    pub async fn create_run(
        &self,
        user_id: UserId,
        input: ObsidianRunCreate,
    ) -> Result<ObsidianRunStatus, AppError> {
        let connection_id = self.active_connection_id(user_id).await?.ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "integration_connection",
                id: "obsidian".to_string(),
            })
        })?;
        let run_id = uuid::Uuid::now_v7();
        let requested_by_user = !input.auto;
        self.obsidian_export_repo
            .create_run(CreateObsidianRunInput {
                run_id,
                connection_id,
                user_id,
                requested_by_user,
                auto: input.auto,
                parent_folder_deleted: input.parent_folder_deleted,
                force_library_entry_ids: input.force_subject_ids,
            })
            .await?;

        #[expect(
            clippy::expect_used,
            reason = "ObsidianSyncConnectionJob is a plain owned struct; serde_json::to_value is infallible for it"
        )]
        let payload = serde_json::to_value(ObsidianSyncConnectionJob {
            connection_id,
            user_id,
            requested_by_user,
            run_id: Some(run_id),
        })
        .expect("ObsidianSyncConnectionJob serializes");
        self.outbox_repo
            .enqueue(
                job_types::INTEGRATION_OBSIDIAN_SYNC_CONNECTION,
                payload,
                Some(format!("obsidian_run:{run_id}")),
                chrono::Utc::now(),
            )
            .await?;

        self.get_run(user_id, run_id).await
    }

    pub async fn get_run(
        &self,
        user_id: UserId,
        run_id: uuid::Uuid,
    ) -> Result<ObsidianRunStatus, AppError> {
        let status = self
            .obsidian_export_repo
            .run_status(user_id, run_id)
            .await?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "obsidian_export_run",
                    id: run_id.to_string(),
                })
            })?;
        Ok(obsidian_run_status_view(status))
    }

    pub async fn get_artifact(
        &self,
        user_id: UserId,
        artifact_id: uuid::Uuid,
    ) -> Result<ObsidianArtifactDownload, AppError> {
        let record = self
            .obsidian_export_repo
            .artifact_download(user_id, artifact_id)
            .await?
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "obsidian_export_artifact",
                    id: artifact_id.to_string(),
                })
            })?;
        Ok(obsidian_artifact_download_view(record))
    }

    pub async fn ack_run(
        &self,
        user_id: UserId,
        run_id: uuid::Uuid,
        input: ObsidianRunAck,
    ) -> Result<ObsidianRunStatus, AppError> {
        let status = self
            .obsidian_export_repo
            .ack_run(user_id, run_id, obsidian_ack_input(input))
            .await?;
        Ok(obsidian_run_status_view(status))
    }

    pub async fn refresh_subjects(
        &self,
        user_id: UserId,
        subject_ids: &[LibraryEntryId],
        reason: &str,
    ) -> Result<ObsidianRefreshResult, AppError> {
        let connection_id = self.active_connection_id(user_id).await?.ok_or_else(|| {
            AppError::Domain(DomainError::NotFound {
                entity: "integration_connection",
                id: "obsidian".to_string(),
            })
        })?;
        let queued = self
            .obsidian_export_repo
            .queue_refresh_subjects(user_id, connection_id, subject_ids, reason)
            .await?;
        Ok(ObsidianRefreshResult { queued })
    }

    pub async fn record_path_rename(
        &self,
        user_id: UserId,
        subject_id: LibraryEntryId,
        new_path: String,
    ) -> Result<(), AppError> {
        let connections = self.connection_repo.list_by_user(user_id).await?;
        let connection = connections
            .into_iter()
            .find(|c| c.provider == IntegrationProvider::Obsidian)
            .ok_or_else(|| {
                AppError::Domain(DomainError::NotFound {
                    entity: "obsidian_connection",
                    id: user_id.to_string(),
                })
            })?;
        let updated = self
            .export_cursor_repo
            .record_generated_path(
                connection.id,
                subject_id,
                new_path.clone(),
                crate::obsidian::full_document_path_for_note_path(&new_path),
            )
            .await?;
        if !updated {
            return Err(AppError::Domain(DomainError::NotFound {
                entity: "obsidian_export_cursor",
                id: format!("{}/{}", connection.id, subject_id),
            }));
        }
        Ok(())
    }

    async fn active_connection_id(
        &self,
        user_id: UserId,
    ) -> Result<Option<IntegrationConnectionId>, AppError> {
        let connections = self.connection_repo.list_by_user(user_id).await?;
        Ok(connections
            .into_iter()
            .find(|c| {
                c.provider == IntegrationProvider::Obsidian
                    && matches!(c.status.as_str(), "pending" | "active")
            })
            .map(|c| c.id))
    }
}

fn obsidian_run_status_view(record: ObsidianRunStatusRecord) -> ObsidianRunStatus {
    ObsidianRunStatus {
        run_id: record.run_id,
        is_finished: record.is_finished(),
        task_status: record.status,
        total_documents: record.total_documents,
        documents_exported: record.documents_exported,
        artifact_ids: record.artifact_ids,
        error: record.error,
    }
}

fn obsidian_artifact_download_view(
    record: ObsidianArtifactDownloadRecord,
) -> ObsidianArtifactDownload {
    ObsidianArtifactDownload {
        artifact_id: record.artifact_id,
        content_type: record.content_type,
        bytes: record.bytes,
    }
}

fn obsidian_ack_input(input: ObsidianRunAck) -> AckObsidianRunInput {
    AckObsidianRunInput {
        artifact_ids: input.artifact_ids,
        subjects: input
            .subjects
            .into_iter()
            .map(obsidian_ack_subject_record)
            .collect(),
    }
}

fn obsidian_ack_subject_record(subject: ObsidianAckSubject) -> ObsidianAckSubjectRecord {
    ObsidianAckSubjectRecord {
        library_entry_id: subject.subject_id,
        status: subject.status,
        error: subject.error,
        last_content_hash: subject.last_content_hash,
        last_full_document_hash: subject.last_full_document_hash,
    }
}

fn sample_preview_document() -> crate::obsidian::ObsidianRenderDocument {
    crate::obsidian::ObsidianRenderDocument {
        subject_id: "lib_00000000-0000-0000-0000-000000000000".to_string(),
        subject_kind: "library_entry".to_string(),
        title: "Area Cat Maintains Plausible Deniability After Dog Takes Blame For Knocked-Over Vase".to_string(),
        full_title: "Area Cat Maintains Plausible Deniability After Dog Takes Blame For Knocked-Over Vase".to_string(),
        url: Some("https://theonion.com/area-cat-maintains-plausible-deniability".to_string()),
        author: Some("Staff Reporter".to_string()),
        item_type: ind_domain::ItemType::Article,
        image_url: Some("https://upload.wikimedia.org/wikipedia/commons/thumb/5/5e/Domestic_Cat_Face_Shot.jpg/960px-Domestic_Cat_Face_Shot.jpg".to_string()),
        summary: Some("A Siamese cat named Chairman Meow has successfully avoided accountability for a shattered Ming vase after the family golden retriever, Biscuit, confessed to the crime out of sheer emotional fragility.".to_string()),
        full_document_text: Some(
            "COLUMBUS, OH — Citing an airtight alibi consisting entirely of sleeping on the radiator, \
             local Siamese cat Chairman Meow, 7, emerged from Tuesday's vase incident with his reputation \
             fully intact after the family's golden retriever, Biscuit, broke down under questioning and \
             admitted guilt despite having been outside at the time of the incident.\n\n\
             \"I was nowhere near the living room,\" said Chairman Meow, who sources confirm had been \
             \"nowhere near\" every breakable object in the house for the past four years. \"Biscuit has \
             always struggled with impulse control. It's actually quite sad.\"\n\n\
             At press time, Chairman Meow had been spotted sitting directly in front of a second vase, \
             maintaining eye contact with the homeowner while slowly moving his tail toward it.".to_string()
        ),
        document_tags: vec!["animals".to_string(), "accountability".to_string(), "breaking-news".to_string()],
        highlights: vec![
            crate::obsidian::ObsidianRenderHighlight {
                id: "hlt_00000000-0000-0000-0000-000000000001".to_string(),
                text: "Biscuit broke down under questioning and admitted guilt despite having been outside at the time of the incident.".to_string(),
                note: Some("Classic. The dog confessed to a crime he didn't commit because someone raised their voice.".to_string()),
                color: "yellow".to_string(),
                tags: vec!["key-finding".to_string(), "dogs".to_string()],
                location: Some("¶ 1".to_string()),
                location_url: Some("https://theonion.com/area-cat-maintains-plausible-deniability#p1".to_string()),
                created_at: chrono::Utc::now(),
            },
            crate::obsidian::ObsidianRenderHighlight {
                id: "hlt_00000000-0000-0000-0000-000000000002".to_string(),
                text: "Biscuit has always struggled with impulse control. It's actually quite sad.".to_string(),
                note: Some("The audacity. He is projecting.".to_string()),
                color: "blue".to_string(),
                tags: vec!["quote".to_string(), "gaslighting".to_string()],
                location: Some("¶ 2".to_string()),
                location_url: Some("https://theonion.com/area-cat-maintains-plausible-deniability#p2".to_string()),
                created_at: chrono::Utc::now(),
            },
            crate::obsidian::ObsidianRenderHighlight {
                id: "hlt_00000000-0000-0000-0000-000000000003".to_string(),
                text: "Chairman Meow had been spotted sitting directly in front of a second vase, maintaining eye contact with the homeowner while slowly moving his tail toward it.".to_string(),
                note: None,
                color: "red".to_string(),
                tags: vec!["escalation".to_string()],
                location: Some("¶ 3".to_string()),
                location_url: Some("https://theonion.com/area-cat-maintains-plausible-deniability#p3".to_string()),
                created_at: chrono::Utc::now(),
            },
        ],
    }
}
