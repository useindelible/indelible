use anyhow::Result;
use chrono::{DateTime, Utc};
use ind_application::repos::dead_letter::{DeadLetterReplay, DeadLetterStats};
use ind_application::repos::integrity::IntegrityStats;
use ind_domain::{BackgroundJobRecovery, DeadLetterJob};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct RecoveryView {
    pub id: String,
    pub recovery_key: String,
    pub job_type: String,
    pub status: String,
    pub failure_class: String,
    pub failure_reason_code: String,
    pub recovery_attempts: i32,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub last_failed_at: DateTime<Utc>,
}

impl From<&BackgroundJobRecovery> for RecoveryView {
    fn from(row: &BackgroundJobRecovery) -> Self {
        Self {
            id: row.id.to_string(),
            recovery_key: row.recovery_key.clone(),
            job_type: row.job_type.clone(),
            status: format!("{:?}", row.status).to_ascii_lowercase(),
            failure_class: format!("{:?}", row.failure_class).to_ascii_lowercase(),
            failure_reason_code: row.failure_reason_code.clone(),
            recovery_attempts: row.recovery_attempts,
            next_retry_at: row.next_retry_at,
            last_failed_at: row.last_failed_at,
        }
    }
}

pub fn format_recovery_list(rows: &[BackgroundJobRecovery], json: bool) -> Result<String> {
    if json {
        return compact_json(&rows.iter().map(RecoveryView::from).collect::<Vec<_>>());
    }

    let mut lines = vec![format!(
        "{:<36}  {:<26}  {:<8}  {:<9}  {:<26}  {:>8}  NEXT_RETRY_AT",
        "ID", "JOB_TYPE", "STATUS", "CLASS", "REASON", "ATTEMPTS"
    )];
    for row in rows {
        let view = RecoveryView::from(row);
        lines.push(format!(
            "{:<36}  {:<26}  {:<8}  {:<9}  {:<26}  {:>8}  {}",
            view.id,
            view.job_type,
            view.status,
            view.failure_class,
            view.failure_reason_code,
            view.recovery_attempts,
            display_optional_datetime(view.next_retry_at),
        ));
    }
    Ok(with_trailing_newline(lines.join("\n")))
}

#[derive(Debug, Clone, Serialize)]
pub struct DlqReplayView {
    pub dead_letter_id: String,
    pub outbox_id: String,
    pub job_type: String,
    pub queued: bool,
    pub original_dedupe_key: Option<String>,
    pub dedupe_key: Option<String>,
    pub replayed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchReindexView {
    pub outbox_id: String,
    pub job_type: String,
    pub page_size: u32,
    pub dedupe_key: Option<String>,
    pub queued: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct EmbeddingsRepairView {
    pub queued_count: i64,
    pub limit: i64,
}

pub fn format_dlq_list(jobs: &[DeadLetterJob], json: bool) -> Result<String> {
    if json {
        return compact_json(&jobs.iter().map(DeadLetterJobView::from).collect::<Vec<_>>());
    }

    let mut lines = vec![format!(
        "{:<36}  {:<28}  {:<10}  {:>8}  FAILED_AT",
        "ID", "JOB_TYPE", "STATUS", "ATTEMPTS"
    )];
    for job in jobs {
        lines.push(format!(
            "{:<36}  {:<28}  {:<10}  {:>8}  {}",
            job.id,
            job.original_job_type,
            replay_status(job),
            job.attempts,
            job.failed_at
        ));
    }
    Ok(with_trailing_newline(lines.join("\n")))
}

pub fn format_dlq_show(job: &DeadLetterJob, json: bool) -> Result<String> {
    let view = DeadLetterJobView::from(job);
    if json {
        return compact_json(&view);
    }

    Ok(with_trailing_newline(format!(
        "id: {}\njob_type: {}\nattempts: {}\nfailed_at: {}\nstatus: {}\noriginal_dedupe_key: {}\nfailure_reason_code: {}\nreplayed_at: {}\nreplay_outbox_id: {}\nerror: {}\npayload: {}",
        view.id,
        view.original_job_type,
        view.attempts,
        view.failed_at,
        view.replay_status,
        display_optional_string(view.original_dedupe_key.as_deref()),
        display_optional_string(view.failure_reason_code.as_deref()),
        display_optional_datetime(view.replayed_at),
        display_optional_string(view.replay_outbox_id.as_deref()),
        view.error_message,
        view.original_payload
    )))
}

pub fn format_dlq_stats(stats: &DeadLetterStats, json: bool) -> Result<String> {
    let view = DeadLetterStatsView::from(stats);
    if json {
        return compact_json(&view);
    }

    Ok(with_trailing_newline(format!(
        "unresolved: {}\nreplayed: {}\ndistinct_unresolved_job_types: {}\noldest_unresolved_failed_at: {}\nnewest_unresolved_failed_at: {}",
        view.unresolved,
        view.replayed,
        view.distinct_unresolved_job_types,
        display_optional_datetime(view.oldest_unresolved_failed_at),
        display_optional_datetime(view.newest_unresolved_failed_at)
    )))
}

pub fn format_dlq_replay(view: &DlqReplayView, json: bool) -> Result<String> {
    if json {
        return compact_json(view);
    }

    Ok(with_trailing_newline(format!(
        "dead_letter_id: {}\noutbox_id: {}\njob_type: {}\nqueued: {}\noriginal_dedupe_key: {}\nreplay_dedupe_key: {}\nreplayed_at: {}",
        view.dead_letter_id,
        view.outbox_id,
        view.job_type,
        view.queued,
        display_optional_string(view.original_dedupe_key.as_deref()),
        display_optional_string(view.dedupe_key.as_deref()),
        display_optional_datetime(view.replayed_at)
    )))
}

pub fn format_search_reindex(view: &SearchReindexView, json: bool) -> Result<String> {
    if json {
        return compact_json(view);
    }

    Ok(with_trailing_newline(format!(
        "search reindex\nqueued: {}\noutbox_id: {}\njob_type: {}\npage_size: {}\ndedupe_key: {}",
        view.queued,
        view.outbox_id,
        view.job_type,
        view.page_size,
        display_optional_string(view.dedupe_key.as_deref())
    )))
}

pub fn format_embeddings_repair(view: &EmbeddingsRepairView, json: bool) -> Result<String> {
    if json {
        return compact_json(view);
    }

    Ok(with_trailing_newline(format!(
        "queued embedding repairs: {}\nlimit: {}",
        view.queued_count, view.limit
    )))
}

pub fn format_integrity_stats(stats: &IntegrityStats, json: bool) -> Result<String> {
    let view = IntegrityStatsView::from(stats);
    if json {
        return compact_json(&view);
    }

    Ok(with_trailing_newline(format!(
        "documents_missing_search_rows: {}\ndocuments_missing_vectors: {}\nfailed_derived_assets: {}\ndead_letter_jobs: {}",
        view.documents_missing_search_rows,
        view.documents_missing_vectors,
        view.failed_derived_assets,
        view.dead_letter_jobs
    )))
}

pub fn dlq_replay_view(replay: &DeadLetterReplay) -> DlqReplayView {
    DlqReplayView {
        dead_letter_id: replay.dead_letter.id.to_string(),
        outbox_id: replay.outbox.id.to_string(),
        job_type: replay.outbox.job_type.clone(),
        queued: replay.queued,
        original_dedupe_key: replay.dead_letter.original_dedupe_key.clone(),
        dedupe_key: replay.outbox.dedupe_key.clone(),
        replayed_at: replay.dead_letter.replayed_at,
    }
}

fn compact_json(value: &impl Serialize) -> Result<String> {
    serde_json::to_string(value).map_err(Into::into)
}

fn with_trailing_newline(mut text: String) -> String {
    text.push('\n');
    text
}

fn display_optional_datetime(value: Option<DateTime<Utc>>) -> String {
    value
        .map(|datetime| datetime.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn display_optional_string(value: Option<&str>) -> &str {
    value.unwrap_or("-")
}

fn replay_status(job: &DeadLetterJob) -> &'static str {
    if job.replayed_at.is_some() {
        "replayed"
    } else {
        "unresolved"
    }
}

#[derive(Debug, Clone, Serialize)]
struct DeadLetterJobView {
    id: String,
    original_job_type: String,
    original_payload: serde_json::Value,
    original_dedupe_key: Option<String>,
    failure_reason_code: Option<String>,
    error_message: String,
    attempts: i32,
    failed_at: DateTime<Utc>,
    replay_status: &'static str,
    replayed_at: Option<DateTime<Utc>>,
    replay_outbox_id: Option<String>,
}

impl From<&DeadLetterJob> for DeadLetterJobView {
    fn from(job: &DeadLetterJob) -> Self {
        Self {
            id: job.id.to_string(),
            original_job_type: job.original_job_type.clone(),
            original_payload: job.original_payload.clone(),
            original_dedupe_key: job.original_dedupe_key.clone(),
            failure_reason_code: job.failure_reason_code.clone(),
            error_message: job.error_message.clone(),
            attempts: job.attempts,
            failed_at: job.failed_at,
            replay_status: replay_status(job),
            replayed_at: job.replayed_at,
            replay_outbox_id: job.replay_outbox_id.map(|id| id.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct DeadLetterStatsView {
    unresolved: i64,
    replayed: i64,
    distinct_unresolved_job_types: i64,
    oldest_unresolved_failed_at: Option<DateTime<Utc>>,
    newest_unresolved_failed_at: Option<DateTime<Utc>>,
}

impl From<&DeadLetterStats> for DeadLetterStatsView {
    fn from(stats: &DeadLetterStats) -> Self {
        Self {
            unresolved: stats.unresolved,
            replayed: stats.replayed,
            distinct_unresolved_job_types: stats.distinct_unresolved_job_types,
            oldest_unresolved_failed_at: stats.oldest_unresolved_failed_at,
            newest_unresolved_failed_at: stats.newest_unresolved_failed_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct IntegrityStatsView {
    documents_missing_search_rows: i64,
    documents_missing_vectors: i64,
    failed_derived_assets: i64,
    dead_letter_jobs: i64,
}

impl From<&IntegrityStats> for IntegrityStatsView {
    fn from(stats: &IntegrityStats) -> Self {
        Self {
            documents_missing_search_rows: stats.documents_missing_search_rows,
            documents_missing_vectors: stats.documents_missing_vectors,
            failed_derived_assets: stats.failed_derived_assets,
            dead_letter_jobs: stats.dead_letter_jobs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ind_domain::{DeadLetterJobId, JobOutbox, JobOutboxId};

    #[test]
    fn operational_views_render_complete_text_and_machine_contracts() -> Result<()> {
        let now = Utc::now();
        let mut dead_letter = DeadLetterJob {
            id: DeadLetterJobId::new(),
            original_job_type: "feed.poll".into(),
            original_payload: serde_json::json!({"source_id": "fso_boundary"}),
            original_dedupe_key: Some("feed.poll:fso_boundary".into()),
            failure_reason_code: Some("provider_error".into()),
            error_message: "provider failed".into(),
            attempts: 3,
            failed_at: now,
            replayed_at: None,
            replay_outbox_id: None,
        };
        let stats = DeadLetterStats {
            unresolved: 1,
            replayed: 2,
            distinct_unresolved_job_types: 1,
            oldest_unresolved_failed_at: Some(now),
            newest_unresolved_failed_at: None,
        };
        let replay = DlqReplayView {
            dead_letter_id: dead_letter.id.to_string(),
            outbox_id: "job_boundary".into(),
            job_type: "feed.poll".into(),
            queued: true,
            original_dedupe_key: dead_letter.original_dedupe_key.clone(),
            dedupe_key: Some("dead-letter.replay:boundary".into()),
            replayed_at: Some(now),
        };
        let reindex = SearchReindexView {
            outbox_id: "job_reindex".into(),
            job_type: "search.reindex_all".into(),
            page_size: 500,
            dedupe_key: None,
            queued: false,
        };
        let repair = EmbeddingsRepairView {
            queued_count: 7,
            limit: 10,
        };
        let integrity = IntegrityStats {
            documents_missing_search_rows: 1,
            documents_missing_vectors: 2,
            failed_derived_assets: 3,
            dead_letter_jobs: 4,
        };

        let text_outputs = [
            format_dlq_list(&[dead_letter.clone()], false)?,
            format_dlq_show(&dead_letter, false)?,
            format_dlq_stats(&stats, false)?,
            format_dlq_replay(&replay, false)?,
            format_search_reindex(&reindex, false)?,
            format_embeddings_repair(&repair, false)?,
            format_integrity_stats(&integrity, false)?,
        ];
        assert!(text_outputs.iter().all(|output| output.ends_with('\n')));
        assert!(text_outputs[0].contains("JOB_TYPE"));
        assert!(text_outputs[1].contains("status: unresolved"));
        assert!(text_outputs[2].contains("newest_unresolved_failed_at: -"));
        assert!(text_outputs[6].contains("documents_missing_vectors: 2"));

        for output in [
            format_dlq_list(&[dead_letter.clone()], true)?,
            format_dlq_show(&dead_letter, true)?,
            format_dlq_stats(&stats, true)?,
            format_dlq_replay(&replay, true)?,
            format_search_reindex(&reindex, true)?,
            format_embeddings_repair(&repair, true)?,
            format_integrity_stats(&integrity, true)?,
        ] {
            serde_json::from_str::<serde_json::Value>(&output)?;
            assert!(!output.contains('\n'));
        }

        let outbox = JobOutbox {
            id: JobOutboxId::new(),
            job_type: "feed.poll".into(),
            payload: dead_letter.original_payload.clone(),
            dedupe_key: Some("feed.poll:fso_boundary".into()),
            available_at: now,
            dispatched_at: None,
            created_at: now,
        };
        dead_letter.replayed_at = Some(now);
        dead_letter.replay_outbox_id = Some(outbox.id);
        let projected = dlq_replay_view(&DeadLetterReplay {
            dead_letter,
            outbox: outbox.clone(),
            queued: false,
        });
        assert_eq!(projected.outbox_id, outbox.id.to_string());
        assert!(!projected.queued);
        Ok(())
    }

    #[test]
    fn recovery_list_renders_patient_rows_in_text_and_json() -> Result<()> {
        use ind_domain::{
            BackgroundJobFailureClass, BackgroundJobRecoveryId, BackgroundJobRecoveryStatus,
        };
        let now = Utc::now();
        let row = BackgroundJobRecovery {
            id: BackgroundJobRecoveryId::new(),
            recovery_key: "rk:doc".into(),
            job_type: "document.ai.embed".into(),
            payload: serde_json::json!({"document_id": "doc_1"}),
            dedupe_key: None,
            outbox_id: None,
            subject_kind: None,
            subject_id: None,
            status: BackgroundJobRecoveryStatus::Waiting,
            failure_class: BackgroundJobFailureClass::Patient,
            failure_reason_code: "ai_provider_unavailable".into(),
            error_message: "provider unavailable".into(),
            apalis_attempts: 1,
            recovery_attempts: 4,
            next_retry_at: Some(now),
            lease_owner: None,
            lease_expires_at: None,
            first_failed_at: now,
            last_failed_at: now,
            resolved_at: None,
            created_at: now,
            updated_at: now,
        };

        let text = format_recovery_list(std::slice::from_ref(&row), false)?;
        assert!(text.contains("document.ai.embed"));
        assert!(text.contains("patient"));
        assert!(text.contains("ai_provider_unavailable"));

        let json = format_recovery_list(std::slice::from_ref(&row), true)?;
        let parsed: serde_json::Value = serde_json::from_str(json.trim())?;
        assert_eq!(parsed[0]["failure_class"], "patient");
        assert_eq!(parsed[0]["recovery_attempts"], 4);
        Ok(())
    }
}
