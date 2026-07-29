use ind_application::AppError;
use ind_application::repos::feed::FeedRepository;
use ind_domain::CanonicalizationConfig;

#[derive(Debug, Default, Clone, Copy)]
pub struct BackfillStats {
    pub updated: u64,
    pub skipped: u64,
}

/// One-off backfill (TASK-239) for `feed_source_entries.canonical_url`. Keyset-paginates by `id`
/// to guaranteed termination: a row whose url never canonicalizes is passed once (the cursor still
/// advances) and counted as `skipped`, so the scan ends when a batch comes back empty -- never an
/// until-zero-updates loop that would re-scan the same bad rows forever. Idempotent: a re-run only
/// sees rows still missing a canonical_url.
pub async fn run_feed_source_entry_canonical_url_backfill(
    feed_repo: &dyn FeedRepository,
    batch_size: i64,
) -> Result<BackfillStats, AppError> {
    let mut stats = BackfillStats::default();
    let mut cursor = uuid::Uuid::nil();

    loop {
        let rows = feed_repo
            .source_entries_missing_canonical_url_after(cursor, batch_size)
            .await?;
        if rows.is_empty() {
            break;
        }

        for (entry_id, url) in rows {
            cursor = *entry_id.as_uuid();
            match ind_domain::canonicalize_url(&url, &CanonicalizationConfig::default()) {
                Ok(canonical) => {
                    feed_repo
                        .set_source_entry_canonical_url(entry_id, &canonical.into_string())
                        .await?;
                    stats.updated += 1;
                }
                Err(err) => {
                    tracing::warn!(
                        entry_id = %cursor,
                        url = %url,
                        error = %err,
                        "canonical_url backfill: skipping un-canonicalizable url"
                    );
                    stats.skipped += 1;
                }
            }
        }
    }

    tracing::info!(
        updated = stats.updated,
        skipped = stats.skipped,
        "feed_source_entries canonical_url backfill complete"
    );
    Ok(stats)
}
