//! Helpers for the universal background job recovery ledger.
//!
//! `recovery_key_for` derives a stable identity for a logical job:
//! prefer the original outbox dedupe key, otherwise hash the payload.
//!
//! `extract_subject` discovers the natural subject of a job payload so
//! operators can filter active recoveries by document/library/feed/etc.
//!
//! `last_backoff_for` returns the longest backoff in the job's retry policy.
//! The worker handler uses this to push `next_retry_at` forward when an
//! exhausted job is recorded as `waiting` so the recovery sweeper does not
//! race Apalis on a job Apalis is still about to retry.

use std::time::Duration;

use ind_domain::{BackgroundJobSubjectKind, retry_policy_for};
use sha2::{Digest, Sha256};

pub fn recovery_key_for(
    job_type: &str,
    payload: &serde_json::Value,
    dedupe_key: Option<&str>,
) -> String {
    if let Some(key) = dedupe_key {
        return format!("dedupe:{key}");
    }
    let canonical = serde_json::to_string(payload).unwrap_or_default();
    let hash = Sha256::digest(canonical.as_bytes());
    format!("payload:{job_type}:{}", hex::encode(hash))
}

pub fn extract_subject(payload: &serde_json::Value) -> Option<(BackgroundJobSubjectKind, String)> {
    let obj = payload.as_object()?;
    if let Some(v) = obj.get("document_id").and_then(|v| v.as_str()) {
        return Some((BackgroundJobSubjectKind::Document, v.to_owned()));
    }
    if let Some(v) = obj.get("library_entry_id").and_then(|v| v.as_str()) {
        return Some((BackgroundJobSubjectKind::LibraryEntry, v.to_owned()));
    }
    if let Some(v) = obj
        .get("feed_delivery_id")
        .or_else(|| obj.get("delivery_id"))
        .and_then(|v| v.as_str())
    {
        return Some((BackgroundJobSubjectKind::FeedDelivery, v.to_owned()));
    }
    if let Some(v) = obj.get("source_id").and_then(|v| v.as_str()) {
        return Some((BackgroundJobSubjectKind::FeedSource, v.to_owned()));
    }
    if let Some(v) = obj.get("connection_id").and_then(|v| v.as_str()) {
        return Some((
            BackgroundJobSubjectKind::IntegrationConnection,
            v.to_owned(),
        ));
    }
    if let Some(v) = obj
        .get("import_job_id")
        .or_else(|| obj.get("import_id"))
        .and_then(|v| v.as_str())
    {
        return Some((BackgroundJobSubjectKind::ImportJob, v.to_owned()));
    }
    None
}

pub fn last_backoff_for(job_type: &str) -> Duration {
    retry_policy_for(job_type)
        .backoff_durations
        .into_iter()
        .max()
        .unwrap_or(Duration::from_secs(60))
}
