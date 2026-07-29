use ind_application::AppError;
use ind_application::repos::email_unsubscribe_target::UnsubscribeTargetUpsert;
use ind_domain::{UserId, parse_from_header};
use ind_integrations::email::{InboundEmail, parse_unsubscribe_targets};

use crate::context::EmailJobDeps;

/// Captures any RFC 2369 / RFC 8058 unsubscribe targets present on the inbound
/// email and persists them against the sender row. Silently no-ops when:
///   * neither header is present
///   * neither repository is configured on the worker context (test paths)
///   * the sender hasn't been resolved yet (defensive — should not occur after
///     `claim_and_enqueue` has run, but we don't want to fail an ingest job
///     over a missing audit row).
pub async fn persist_unsubscribe_targets(
    ctx: &EmailJobDeps,
    user_id: UserId,
    email: &InboundEmail,
) -> Result<(), AppError> {
    let Some(target_repo) = ctx.email_unsubscribe_target_repo.as_ref() else {
        return Ok(());
    };
    let Some(sender_repo) = ctx.email_sender_repo.as_ref() else {
        return Ok(());
    };

    let targets = parse_unsubscribe_targets(
        email.list_unsubscribe.as_deref(),
        email.list_unsubscribe_post.as_deref(),
    );
    if targets.is_empty() {
        return Ok(());
    }

    let (canonical, _) = parse_from_header(&email.from_address);
    let Some(sender) = sender_repo
        .find_by_user_and_canonical(user_id, &canonical)
        .await?
    else {
        return Ok(());
    };

    target_repo
        .upsert(
            sender.id,
            UnsubscribeTargetUpsert {
                one_click_post_url: targets.one_click_post_url,
                mailto_addr: targets.mailto_addr,
                web_url: targets.web_url,
            },
        )
        .await?;

    Ok(())
}

/// Resolves the `email_senders` row for this user + from-address pair. The
/// sender is already upserted in `claim_and_enqueue` (Phase 2), so the lookup
/// should hit. Returns `None` when the sender repo is not configured (test
/// contexts that bypass sender persistence) so the worker still ingests.
pub(super) async fn resolve_sender_id(
    ctx: &EmailJobDeps,
    user_id: UserId,
    from_address: &str,
) -> Result<Option<ind_domain::EmailSenderId>, AppError> {
    let Some(sender_repo) = ctx.email_sender_repo.as_ref() else {
        return Ok(None);
    };
    let (canonical, _) = parse_from_header(from_address);
    let sender = sender_repo
        .find_by_user_and_canonical(user_id, &canonical)
        .await?;
    Ok(sender.map(|s| s.id))
}

/// Bumps `email_senders.delivery_count` (and `last_seen_at`) for a successful
/// ingest. Called after `mark_processed` so failed attempts don't inflate the
/// counter shown in the Settings → Email "Deliveries" stat.
pub async fn bump_sender_delivery_count(
    ctx: &EmailJobDeps,
    sender_id: Option<ind_domain::EmailSenderId>,
) -> Result<(), AppError> {
    let (Some(sender_repo), Some(sender_id)) = (ctx.email_sender_repo.as_ref(), sender_id) else {
        return Ok(());
    };
    sender_repo.increment_delivery(sender_id).await
}
