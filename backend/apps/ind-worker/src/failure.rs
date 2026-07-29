use std::sync::Arc;
use std::time::Duration;

use apalis::prelude::{Attempt, BoxDynError, Data};
use apalis_postgres::{PgContext, PgTaskId};
use ind_application::AppError;
use ind_application::recovery_keys::{extract_subject, last_backoff_for, recovery_key_for};
use ind_domain::{GenericJobEnvelope, retry_policy_for};

use crate::context::WorkerContext;
use crate::{jobs, recovery_handler};

enum FailureDisposition {
    Retryable,
    Terminal,
    Patient,
}

struct ClassifiedFailure {
    disposition: FailureDisposition,
    reason_code: &'static str,
}

pub async fn handle_job(
    job: GenericJobEnvelope,
    ctx: Data<Arc<WorkerContext>>,
    attempt: Attempt,
    task_id: PgTaskId,
    task_ctx: PgContext,
) -> Result<(), BoxDynError> {
    tracing::info!(
        job_type = %job.job_type,
        task_id = %task_id,
        outbox_id = %job.outbox_id,
        attempt = attempt.current(),
        "processing job"
    );

    let _permit = ctx.concurrency.acquire(&job.job_type).await;

    let policy = retry_policy_for(&job.job_type);
    let job_type = job.job_type.clone();
    let payload = job.payload.clone();
    let dedupe_key = job.dedupe_key.clone();
    let outbox_id = job.outbox_id;
    let recovery_key = recovery_key_for(&job_type, &payload, dedupe_key.as_deref());
    let subject = extract_subject(&payload);

    match jobs::render::dispatch_generic_job(&ctx, job).await {
        Ok(()) => {
            recovery_handler::record_success(
                &ctx.background_recovery_repo,
                &recovery_key,
                chrono::Utc::now(),
            )
            .await;
            Ok(())
        }
        Err(error) => {
            let current = attempt.current();
            let classified = classify_error(&error);
            let error_message = error.to_string();

            match classified.disposition {
                FailureDisposition::Terminal => {
                    tracing::warn!(
                        job_type = %job_type,
                        task_id = %task_id,
                        attempt = current,
                        reason_code = classified.reason_code,
                        error = %error,
                        "job hit terminal failure"
                    );

                    mark_ingest_log_failed(&ctx, &job_type, &payload, &error_message).await;

                    if let Err(record_err) = recovery_handler::record_terminal_failure(
                        &ctx.background_recovery_repo,
                        recovery_handler::RecordedFailure {
                            recovery_key: &recovery_key,
                            job_type: &job_type,
                            payload,
                            dedupe_key: dedupe_key.as_deref(),
                            outbox_id: Some(outbox_id),
                            subject_kind: subject.as_ref().map(|s| s.0),
                            subject_id: subject.as_ref().map(|s| s.1.as_str()),
                            failure_reason_code: classified.reason_code,
                            error_message: &error_message,
                            attempt: current as i32,
                            now: chrono::Utc::now(),
                        },
                    )
                    .await
                    {
                        tracing::error!(
                            error = %record_err,
                            recovery_key = %recovery_key,
                            job_type = %job_type,
                            "failed to record terminal background job recovery; leaving job unacked",
                        );
                        return Err(Box::new(record_err) as BoxDynError);
                    }

                    Ok(())
                }
                FailureDisposition::Patient => {
                    tracing::warn!(
                        job_type = %job_type,
                        task_id = %task_id,
                        attempt = current,
                        reason_code = classified.reason_code,
                        error = %error,
                        "dependency unavailable; parking job as patient recovery"
                    );

                    if let Err(record_err) = recovery_handler::record_patient_failure(
                        &ctx.background_recovery_repo,
                        recovery_handler::RecordedFailure {
                            recovery_key: &recovery_key,
                            job_type: &job_type,
                            payload,
                            dedupe_key: dedupe_key.as_deref(),
                            outbox_id: Some(outbox_id),
                            subject_kind: subject.as_ref().map(|s| s.0),
                            subject_id: subject.as_ref().map(|s| s.1.as_str()),
                            failure_reason_code: classified.reason_code,
                            error_message: &error_message,
                            attempt: current as i32,
                            now: chrono::Utc::now(),
                        },
                    )
                    .await
                    {
                        tracing::error!(
                            error = %record_err,
                            recovery_key = %recovery_key,
                            job_type = %job_type,
                            "failed to record patient background job recovery; leaving job unacked",
                        );
                        return Err(Box::new(record_err) as BoxDynError);
                    }

                    Ok(())
                }
                FailureDisposition::Retryable => {
                    let backoff_idx = current.saturating_sub(1);
                    let delay = policy
                        .backoff_durations
                        .get(backoff_idx)
                        .or_else(|| policy.backoff_durations.last())
                        .copied()
                        .unwrap_or_default();
                    if current >= policy.max_attempts as usize {
                        tracing::warn!(
                            job_type = %job_type,
                            task_id = %task_id,
                            attempt = current,
                            max = policy.max_attempts,
                            reason_code = classified.reason_code,
                            "job exhausted apalis retries; recording recovery row"
                        );
                        mark_ingest_log_failed(&ctx, &job_type, &payload, &error_message).await;

                        let now = chrono::Utc::now();
                        let recovery_next = now
                            + chrono::Duration::from_std(last_backoff_for(&job_type))
                                .unwrap_or_else(|_| chrono::Duration::seconds(900));

                        if let Err(record_err) = recovery_handler::record_retryable_exhausted(
                            &ctx.background_recovery_repo,
                            recovery_handler::RecordedFailure {
                                recovery_key: &recovery_key,
                                job_type: &job_type,
                                payload,
                                dedupe_key: dedupe_key.as_deref(),
                                outbox_id: Some(outbox_id),
                                subject_kind: subject.as_ref().map(|s| s.0),
                                subject_id: subject.as_ref().map(|s| s.1.as_str()),
                                failure_reason_code: classified.reason_code,
                                error_message: &error_message,
                                attempt: current as i32,
                                now,
                            },
                            recovery_next,
                        )
                        .await
                        {
                            tracing::error!(
                                error = %record_err,
                                recovery_key = %recovery_key,
                                job_type = %job_type,
                                "failed to record exhausted background job recovery; leaving job unacked",
                            );
                            return Err(Box::new(record_err) as BoxDynError);
                        }

                        return Ok(());
                    }

                    tracing::info!(
                        job_type = %job_type,
                        task_id = %task_id,
                        attempt = current,
                        backoff_secs = delay.as_secs(),
                        reason_code = classified.reason_code,
                        "job failed, scheduling retry with backoff"
                    );

                    schedule_retry(&ctx, &task_id, &task_ctx, delay).await;
                    Err(Box::new(error) as BoxDynError)
                }
            }
        }
    }
}

fn extract_ingest_log_id(job_type: &str, payload: &serde_json::Value) -> Option<uuid::Uuid> {
    if job_type == "email.ingest" {
        payload
            .get("ingest_log_id")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<uuid::Uuid>().ok())
    } else {
        None
    }
}

async fn mark_ingest_log_failed(
    ctx: &WorkerContext,
    job_type: &str,
    payload: &serde_json::Value,
    error: &str,
) {
    if let Some(log_id) = extract_ingest_log_id(job_type, payload)
        && let Some(repo) = ctx.email_ingest_log_repo.as_ref()
        && let Err(e) = repo.mark_failed(log_id, error).await
    {
        tracing::error!(
            ingest_log_id = %log_id,
            error = %e,
            "failed to mark email ingest log as failed"
        );
    }
}

fn classify_error(error: &AppError) -> ClassifiedFailure {
    match error {
        AppError::Auth | AppError::Forbidden => ClassifiedFailure {
            disposition: FailureDisposition::Terminal,
            reason_code: "auth_required",
        },
        AppError::RateLimited => ClassifiedFailure {
            disposition: FailureDisposition::Retryable,
            reason_code: "network_transient",
        },
        AppError::Repository(_) => ClassifiedFailure {
            disposition: FailureDisposition::Retryable,
            reason_code: "db_transient",
        },
        AppError::Domain(ind_domain::DomainError::Validation { .. }) => ClassifiedFailure {
            disposition: FailureDisposition::Terminal,
            reason_code: "invalid_input",
        },
        AppError::Domain(ind_domain::DomainError::NotFound { .. }) => ClassifiedFailure {
            disposition: FailureDisposition::Terminal,
            reason_code: "missing_source",
        },
        AppError::Domain(ind_domain::DomainError::InvariantViolation { .. }) => ClassifiedFailure {
            disposition: FailureDisposition::Terminal,
            reason_code: "deterministic_renderer_failure",
        },
        AppError::ExternalService { service, message } if service == "renderer" => {
            classify_renderer_error_message(message)
        }
        AppError::ExternalService { service, message } if service == "mila-provider" => {
            classify_ai_provider_error_message(message)
        }
        AppError::ExternalService { .. } => ClassifiedFailure {
            disposition: FailureDisposition::Retryable,
            reason_code: "external_service_error",
        },
        AppError::ProviderUnavailable { .. } => ClassifiedFailure {
            disposition: FailureDisposition::Patient,
            reason_code: ind_domain::AI_PROVIDER_UNAVAILABLE,
        },
        _ => ClassifiedFailure {
            disposition: FailureDisposition::Retryable,
            reason_code: "worker_crash_or_abandonment",
        },
    }
}

fn classify_renderer_error_message(message: &str) -> ClassifiedFailure {
    let lower = message.to_ascii_lowercase();

    if lower.contains("http 400") || lower.contains("http 422") {
        ClassifiedFailure {
            disposition: FailureDisposition::Terminal,
            reason_code: "invalid_input",
        }
    } else if lower.contains("http 401") || lower.contains("http 403") {
        ClassifiedFailure {
            disposition: FailureDisposition::Terminal,
            reason_code: "auth_required",
        }
    } else if lower.contains("http 451") {
        ClassifiedFailure {
            disposition: FailureDisposition::Terminal,
            reason_code: "policy_blocked",
        }
    } else if lower.contains("http 404") {
        ClassifiedFailure {
            disposition: FailureDisposition::Terminal,
            reason_code: "unsupported_target",
        }
    } else if lower.contains("page is too large")
        || lower.contains("page too large")
        || lower.contains("page_too_large")
        || lower.contains("unsupported(page_too_large)")
    {
        ClassifiedFailure {
            disposition: FailureDisposition::Terminal,
            reason_code: "page_too_large",
        }
    } else if lower.contains("timeout") || lower.contains("deadline") {
        ClassifiedFailure {
            disposition: FailureDisposition::Retryable,
            reason_code: "renderer_timeout",
        }
    } else {
        ClassifiedFailure {
            disposition: FailureDisposition::Retryable,
            reason_code: "renderer_unavailable",
        }
    }
}

fn classify_ai_provider_error_message(message: &str) -> ClassifiedFailure {
    let lower = message.to_ascii_lowercase();

    if lower.contains("authentication") || lower.contains("invalid api key") {
        ClassifiedFailure {
            disposition: FailureDisposition::Terminal,
            reason_code: "ai_auth_failed",
        }
    } else if lower.contains("rate limit") || lower.contains("429") {
        ClassifiedFailure {
            disposition: FailureDisposition::Retryable,
            reason_code: "network_transient",
        }
    } else if lower.contains("context length")
        || lower.contains("too long")
        || lower.contains("too many tokens")
    {
        ClassifiedFailure {
            disposition: FailureDisposition::Terminal,
            reason_code: "ai_context_too_long",
        }
    } else if lower.contains("truncated") || lower.contains("finish_reason=length") {
        // Terminal: action prompts run at low temperature with a generous output cap, so a
        // truncation is a deterministic oversized response that a retry reproduces, not transient
        // sampling variance. Retrying only burns attempts before the row lands in the DLQ anyway.
        ClassifiedFailure {
            disposition: FailureDisposition::Terminal,
            reason_code: "ai_output_truncated",
        }
    } else {
        ClassifiedFailure {
            disposition: FailureDisposition::Retryable,
            reason_code: "ai_provider_error",
        }
    }
}

async fn schedule_retry(
    ctx: &WorkerContext,
    task_id: &PgTaskId,
    task_ctx: &PgContext,
    delay: Duration,
) {
    let Some(lock_by) = task_ctx.lock_by().clone() else {
        tracing::warn!(task_id = %task_id, "failed to schedule retry: task lock owner missing");
        return;
    };

    match ctx
        .apalis_job_repo
        .reschedule_locked_job(&task_id.to_string(), &lock_by, delay)
        .await
    {
        Ok(1) => {}
        Ok(rows_affected) => {
            tracing::warn!(
                task_id = %task_id,
                rows_affected,
                "failed to schedule retry: task row was not updated"
            );
        }
        Err(error) => {
            tracing::warn!(task_id = %task_id, %error, "failed to schedule retry");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_unavailable_classifies_as_patient() {
        let classified = classify_error(&AppError::ProviderUnavailable {
            message: "error sending request: connection refused".into(),
        });
        assert!(matches!(
            classified.disposition,
            FailureDisposition::Patient
        ));
        assert_eq!(classified.reason_code, ind_domain::AI_PROVIDER_UNAVAILABLE);
    }

    #[test]
    fn failure_classification_preserves_retry_and_terminal_boundaries() {
        let cases = [
            (classify_error(&AppError::Auth), false, "auth_required"),
            (classify_error(&AppError::Forbidden), false, "auth_required"),
            (
                classify_error(&AppError::RateLimited),
                true,
                "network_transient",
            ),
            (
                classify_error(&AppError::Domain(ind_domain::DomainError::Validation {
                    field: "payload".into(),
                    message: "invalid".into(),
                })),
                false,
                "invalid_input",
            ),
            (
                classify_error(&AppError::Domain(ind_domain::DomainError::NotFound {
                    entity: "document",
                    id: "missing".into(),
                })),
                false,
                "missing_source",
            ),
            (
                classify_error(&AppError::Domain(
                    ind_domain::DomainError::InvariantViolation {
                        message: "deterministic".into(),
                    },
                )),
                false,
                "deterministic_renderer_failure",
            ),
            (
                classify_renderer_error_message("renderer returned HTTP 400"),
                false,
                "invalid_input",
            ),
            (
                classify_renderer_error_message("renderer returned HTTP 401"),
                false,
                "auth_required",
            ),
            (
                classify_renderer_error_message("renderer returned HTTP 451"),
                false,
                "policy_blocked",
            ),
            (
                classify_renderer_error_message("renderer returned HTTP 404"),
                false,
                "unsupported_target",
            ),
            (
                classify_renderer_error_message("renderer returned HTTP 500: screenshot timeout"),
                true,
                "renderer_timeout",
            ),
            (
                classify_renderer_error_message(
                    "unsupported(page_too_large): dimensions exceed Chromium limit",
                ),
                false,
                "page_too_large",
            ),
            (
                classify_ai_provider_error_message(
                    "model response truncated before completion (finish_reason=length)",
                ),
                false,
                "ai_output_truncated",
            ),
            (
                classify_ai_provider_error_message(
                    "This model's maximum context length is 8192 tokens",
                ),
                false,
                "ai_context_too_long",
            ),
            (
                classify_ai_provider_error_message("authentication failed: invalid api key"),
                false,
                "ai_auth_failed",
            ),
            (
                classify_ai_provider_error_message("provider returned 429 rate limit"),
                true,
                "network_transient",
            ),
            (
                classify_ai_provider_error_message("provider connection reset"),
                true,
                "ai_provider_error",
            ),
            (
                classify_renderer_error_message("renderer connection reset"),
                true,
                "renderer_unavailable",
            ),
        ];
        for (actual, retryable, reason) in cases {
            assert_eq!(
                matches!(actual.disposition, FailureDisposition::Retryable),
                retryable
            );
            assert_eq!(actual.reason_code, reason);
        }
    }
}
