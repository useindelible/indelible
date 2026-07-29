use anyhow::{Context, Result, bail};
use chrono::Utc;
use ind_application::repos::{
    background_job_recovery::{ActiveRecoveryFilter, BackgroundJobRecoveryRepository},
    dead_letter::DeadLetterRepository,
    embedding_backfill::EmbeddingBackfillRepository,
    integrity::IntegrityStatsRepository,
    search_reindex::SearchReindexRepository,
};
use ind_domain::{BackgroundJobRecoveryStatus, DeadLetterJobId};

use crate::{
    cli::Command,
    output::{
        EmbeddingsRepairView, SearchReindexView, dlq_replay_view, format_dlq_list,
        format_dlq_replay, format_dlq_show, format_dlq_stats, format_embeddings_repair,
        format_integrity_stats, format_recovery_list, format_search_reindex,
    },
};

pub struct CommandContext<'a> {
    pub dead_letters: &'a dyn DeadLetterRepository,
    pub recoveries: &'a dyn BackgroundJobRecoveryRepository,
    pub search_reindex: &'a dyn SearchReindexRepository,
    pub embeddings: &'a dyn EmbeddingBackfillRepository,
    pub integrity: &'a dyn IntegrityStatsRepository,
}

pub async fn execute(command: Command, context: CommandContext<'_>) -> Result<String> {
    match command {
        Command::Help => Ok(crate::USAGE.to_string()),
        Command::JobsDlqList { limit, json } => {
            let jobs = context.dead_letters.list(limit).await?;
            format_dlq_list(&jobs, json)
        }
        Command::JobsDlqShow {
            dead_letter_id,
            json,
        } => {
            let job = context
                .dead_letters
                .get(parse_dead_letter_id(&dead_letter_id)?)
                .await?;
            format_dlq_show(&job, json)
        }
        Command::JobsDlqReplay {
            dead_letter_id,
            json,
        } => {
            let replay = context
                .dead_letters
                .replay(parse_dead_letter_id(&dead_letter_id)?, Utc::now())
                .await?;
            format_dlq_replay(&dlq_replay_view(&replay), json)
        }
        Command::JobsDlqStats { json } => {
            let stats = context.dead_letters.stats().await?;
            format_dlq_stats(&stats, json)
        }
        Command::JobsRecoveryList {
            status,
            job_type,
            limit,
            json,
        } => {
            let filter = ActiveRecoveryFilter {
                status: status.as_deref().map(parse_recovery_status).transpose()?,
                job_type,
                subject_kind: None,
            };
            let rows = context.recoveries.list_active(filter, limit).await?;
            format_recovery_list(&rows, json)
        }
        Command::SearchReindex { page_size, json } => {
            let admission = context
                .search_reindex
                .enqueue_full_reindex(page_size, None, Utc::now())
                .await?;
            let outbox = admission
                .outbox
                .context("manual search reindex admission did not return an outbox row")?;
            format_search_reindex(
                &SearchReindexView {
                    outbox_id: outbox.id.to_string(),
                    job_type: outbox.job_type,
                    page_size,
                    dedupe_key: outbox.dedupe_key,
                    queued: admission.queued,
                },
                json,
            )
        }
        Command::EmbeddingsRepair { limit, json } => {
            let queued_count = context
                .embeddings
                .enqueue_missing_vector_repairs(limit)
                .await?;
            format_embeddings_repair(
                &EmbeddingsRepairView {
                    queued_count,
                    limit,
                },
                json,
            )
        }
        Command::IntegrityStats { json } => {
            let stats = context.integrity.stats().await?;
            format_integrity_stats(&stats, json)
        }
    }
}

fn parse_dead_letter_id(raw: &str) -> Result<DeadLetterJobId> {
    raw.parse()
        .with_context(|| format!("invalid dead_letter_id `{raw}`"))
}

fn parse_recovery_status(raw: &str) -> Result<BackgroundJobRecoveryStatus> {
    match raw {
        "waiting" => Ok(BackgroundJobRecoveryStatus::Waiting),
        "leased" => Ok(BackgroundJobRecoveryStatus::Leased),
        "terminal" => Ok(BackgroundJobRecoveryStatus::Terminal),
        "resolved" => Ok(BackgroundJobRecoveryStatus::Resolved),
        other => bail!("invalid status `{other}` (waiting|leased|terminal|resolved)"),
    }
}
