use super::helpers::{DeliveryClaimOutcome, status_for_claim_outcomes};
use super::*;

pub(super) async fn resend_webhook_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    let Some(ref email_ingest_ops) = state.email_ingest_ops else {
        warn!("email ingest webhook received but email_ingest_ops not configured");
        return StatusCode::SERVICE_UNAVAILABLE;
    };

    let Some(ref provider) = state.email_ingest_provider else {
        warn!("email ingest webhook received but provider not configured");
        return StatusCode::SERVICE_UNAVAILABLE;
    };

    if let Err(e) = provider.verify_signature(&body, &headers) {
        warn!(error = %e, "webhook signature verification failed");
        return StatusCode::UNAUTHORIZED;
    }

    let metadata = match provider.parse_webhook_metadata(&body) {
        Ok(m) => m,
        Err(e) => {
            warn!(error = %e, "failed to parse webhook metadata");
            return StatusCode::BAD_REQUEST;
        }
    };

    let feed_domain = state
        .config
        .email_feed_domain
        .as_deref()
        .unwrap_or("feed.useindelible.com");
    let library_domain = state
        .config
        .email_library_domain
        .as_deref()
        .unwrap_or("library.useindelible.com");

    let mut outcomes = Vec::with_capacity(metadata.to_addresses.len());

    for to_address in &metadata.to_addresses {
        let parsed = match parse_ingest_address(to_address, feed_domain, library_domain) {
            Some(p) => p,
            None => {
                info!(address = %to_address, "unrecognized ingest address, skipping");
                outcomes.push(DeliveryClaimOutcome::Ignored);
                continue;
            }
        };

        let user = match email_ingest_ops
            .resolve_ingest_recipient(parsed.destination, &parsed.token)
            .await
        {
            Ok(Some(u)) => u,
            Ok(None) => {
                info!(
                    token_fingerprint = %email_token_fingerprint(&parsed.token),
                    "unknown email token, skipping"
                );
                outcomes.push(DeliveryClaimOutcome::Ignored);
                continue;
            }
            Err(e) => {
                warn!(error = %e, "failed to look up user by email token");
                outcomes.push(DeliveryClaimOutcome::RetryableFailure);
                continue;
            }
        };

        let job = EmailIngestJob {
            provider: provider.provider().to_string(),
            provider_email_id: metadata.provider_email_id.clone(),
            raw_payload: body.to_vec(),
            user_id: user.id,
            destination: parsed.destination.as_str().to_string(),
            ingest_log_id: None,
        };

        let job_payload = match serde_json::to_value(&job) {
            Ok(p) => p,
            Err(e) => {
                warn!(error = %e, "failed to serialize email ingest job");
                outcomes.push(DeliveryClaimOutcome::RetryableFailure);
                continue;
            }
        };

        match email_ingest_ops
            .claim_and_enqueue(ClaimAndEnqueueInput {
                provider: provider.provider().as_str(),
                provider_email_id: &metadata.provider_email_id,
                user_id: user.id,
                destination: parsed.destination.as_str(),
                job_type: job_types::EMAIL_INGEST,
                job_payload,
                raw_payload: Some(&body),
                from_address: &metadata.from_address,
                list_id: metadata.list_id.as_deref(),
            })
            .await
        {
            Ok(Some(log_row)) => {
                if log_row.status == "blocked" {
                    outcomes.push(DeliveryClaimOutcome::Claimed);
                    info!(
                        log_id = %log_row.id,
                        user_id = %user.id,
                        destination = parsed.destination.as_str(),
                        "email ingest blocked by sender rule"
                    );
                } else {
                    outcomes.push(DeliveryClaimOutcome::Claimed);
                    info!(
                        log_id = %log_row.id,
                        user_id = %user.id,
                        destination = parsed.destination.as_str(),
                        "email ingest job enqueued"
                    );
                }
            }
            Ok(None) => {
                outcomes.push(DeliveryClaimOutcome::Duplicate);
                info!(
                    provider_email_id = %metadata.provider_email_id,
                    user_id = %user.id,
                    destination = parsed.destination.as_str(),
                    "duplicate email delivery, already claimed"
                );
            }
            Err(e) => {
                outcomes.push(DeliveryClaimOutcome::RetryableFailure);
                warn!(error = %e, "failed to claim and enqueue email ingest job");
            }
        }
    }

    status_for_claim_outcomes(outcomes)
}

fn email_token_fingerprint(token: &str) -> String {
    ind_auth::hash_token(token).chars().take(12).collect()
}
