use std::time::Duration;

use super::job_types;

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: i32,
    pub backoff_durations: Vec<Duration>,
}

/// Apalis dispatch priority for a job type. The shared queue fetches highest-priority first
/// (`ORDER BY priority DESC`) and the default for every job is `0`. The worker's per-job-type
/// concurrency semaphore is acquired only after a job is claimed, so it bounds how many renders
/// run at once but does not affect claim order; priority is what keeps a Feed-open burst of
/// read-ahead renders from being claimed ahead of foreground/user-triggered work. Feed
/// preparation therefore sits below the default.
pub fn job_priority_for(job_type: &str) -> i32 {
    match job_type {
        job_types::FEED_PREPARE_DOCUMENT => -10,
        _ => 0,
    }
}

/// Reason code stamped on patient recoveries created by AI provider transport failures.
pub const AI_PROVIDER_UNAVAILABLE: &str = "ai_provider_unavailable";

/// Sparse replay schedule for patient (dependency-outage) recoveries, indexed by
/// `recovery_attempts` and clamped at the six-hour tail.
pub const PATIENT_BACKOFF_SECS: [u64; 5] = [60, 300, 900, 3_600, 21_600];

pub fn patient_backoff(recovery_attempts: i32) -> Duration {
    let index = usize::try_from(recovery_attempts).unwrap_or(0);
    let secs = PATIENT_BACKOFF_SECS
        .get(index)
        .copied()
        .unwrap_or(PATIENT_BACKOFF_SECS[PATIENT_BACKOFF_SECS.len() - 1]);
    Duration::from_secs(secs)
}

/// Returns the retry policy for a given job type per the spec (Section 32.2).
/// Unknown job types get a conservative default.
pub fn retry_policy_for(job_type: &str) -> RetryPolicy {
    match job_type {
        job_types::FEED_AUTOSAVE
        | job_types::DOCUMENT_REPROCESS
        | job_types::SEARCH_REINDEX_DOCUMENT
        | job_types::SEARCH_REINDEX_ALL => RetryPolicy {
            max_attempts: 3,
            backoff_durations: vec![
                Duration::from_secs(60),
                Duration::from_secs(300),
                Duration::from_secs(900),
            ],
        },
        job_types::FEED_POLL => RetryPolicy {
            max_attempts: 3,
            backoff_durations: vec![
                Duration::from_secs(30),
                Duration::from_secs(120),
                Duration::from_secs(600),
            ],
        },
        job_types::WEBHOOK_DELIVER => RetryPolicy {
            max_attempts: 3,
            backoff_durations: vec![
                Duration::from_secs(60),
                Duration::from_secs(600),
                Duration::from_secs(3600),
            ],
        },
        job_types::EMAIL_INGEST => RetryPolicy {
            max_attempts: 3,
            backoff_durations: vec![
                Duration::from_secs(30),
                Duration::from_secs(120),
                Duration::from_secs(600),
            ],
        },
        job_types::DOCUMENT_AI_EMBED
        | job_types::DOCUMENT_AI_SUMMARIZE
        | job_types::DOCUMENT_AI_TAGS
        | job_types::DOCUMENT_AI_ENTITIES => RetryPolicy {
            max_attempts: 2,
            backoff_durations: vec![Duration::from_secs(30), Duration::from_secs(120)],
        },
        job_types::INTEGRATION_OBSIDIAN_SYNC_CONNECTION => RetryPolicy {
            max_attempts: 3,
            backoff_durations: vec![
                Duration::from_secs(60),
                Duration::from_secs(300),
                Duration::from_secs(900),
            ],
        },
        job_types::INTEGRATION_NOTION_EXPORT_DOCUMENT
        | job_types::INTEGRATION_NOTION_SYNC_CONNECTION => RetryPolicy {
            max_attempts: 4,
            backoff_durations: vec![
                Duration::from_secs(15),
                Duration::from_secs(60),
                Duration::from_secs(300),
                Duration::from_secs(900),
            ],
        },
        job_types::IMPORT_READWISE => RetryPolicy {
            max_attempts: 2,
            backoff_durations: vec![Duration::from_secs(60), Duration::from_secs(300)],
        },
        // Not latency-sensitive: the account's rows are already gone and the
        // bucket cleanup only needs to happen eventually.
        job_types::ACCOUNT_STORAGE_PURGE => RetryPolicy {
            max_attempts: 4,
            backoff_durations: vec![
                Duration::from_secs(60),
                Duration::from_secs(600),
                Duration::from_secs(3600),
                Duration::from_secs(3600),
            ],
        },
        _ => RetryPolicy {
            max_attempts: 3,
            backoff_durations: vec![
                Duration::from_secs(60),
                Duration::from_secs(300),
                Duration::from_secs(900),
            ],
        },
    }
}

#[cfg(test)]
mod patient_backoff_tests {
    use super::*;

    #[test]
    fn patient_backoff_follows_the_sparse_schedule_and_clamps_at_six_hours() {
        assert_eq!(PATIENT_BACKOFF_SECS, [60, 300, 900, 3_600, 21_600]);
        let cases = [
            (0, 60),
            (1, 300),
            (2, 900),
            (3, 3_600),
            (4, 21_600),
            (5, 21_600),
            (5_000, 21_600),
            (-1, 60),
        ];
        for (attempts, secs) in cases {
            assert_eq!(
                patient_backoff(attempts),
                Duration::from_secs(secs),
                "attempts={attempts}"
            );
        }
    }
}

#[cfg(test)]
mod priority_tests {
    use super::*;

    #[test]
    fn feed_prefetch_is_lower_priority_than_foreground_and_default() {
        assert!(
            job_priority_for(job_types::FEED_PREPARE_DOCUMENT)
                < job_priority_for(job_types::DOCUMENT_YOUTUBE_INGEST),
            "read-ahead must not be claimed ahead of foreground document jobs"
        );
        assert_eq!(job_priority_for(job_types::DOCUMENT_YOUTUBE_INGEST), 0);
        assert_eq!(job_priority_for("anything.else"), 0);
    }
}
