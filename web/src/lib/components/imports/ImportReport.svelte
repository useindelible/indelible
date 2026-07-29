<script lang="ts">
	import Badge from '$lib/components/ui/Badge.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import type { ImportJobStatusResponse } from '$lib/api';
	import { normalizeImportStatus } from '$lib/integrations/status';

	interface Props {
		job: ImportJobStatusResponse;
		canRollback?: boolean;
		onRollback?: () => void;
		initialOutcomeLimit?: number;
	}

	let { job, canRollback = false, onRollback, initialOutcomeLimit = 10 }: Props = $props();

	let showAll = $state(false);

	const normalized = $derived(normalizeImportStatus(job.status));

	const visibleOutcomes = $derived(
		showAll ? job.item_outcomes : job.item_outcomes.slice(0, initialOutcomeLimit)
	);

	const hasMore = $derived(job.item_outcomes.length > initialOutcomeLimit);

	const showRollback = $derived(
		canRollback &&
			(normalized === 'completed' || normalized === 'partial') &&
			onRollback !== undefined
	);

	const readwiseReport = $derived(job.readwise_report ?? null);

	const readwisePostImportMessage = $derived.by(() => {
		if (!readwiseReport) return null;

		const opmlCreated = readwiseReport.opml_feeds_created ?? 0;
		const opmlSkipped = readwiseReport.opml_feeds_skipped ?? 0;
		const searchJobs = readwiseReport.search_reindex_jobs_enqueued ?? 0;
		const embedJobs = readwiseReport.embedding_jobs_enqueued ?? 0;
		const archiveAssets = readwiseReport.archive_assets_imported ?? 0;
		const itemActivity = (job.counts.imported ?? 0) + (job.counts.updated ?? 0);

		const parts: string[] = [];

		if (opmlCreated > 0) {
			const tail = opmlSkipped > 0 ? ` (${opmlSkipped} already subscribed)` : '';
			parts.push(
				`${opmlCreated} RSS feed${opmlCreated === 1 ? '' : 's'} added to your Feeds manager${tail}.`
			);
		} else if (opmlSkipped > 0) {
			parts.push(
				`${opmlSkipped} OPML feed${opmlSkipped === 1 ? ' was' : 's were'} already subscribed; none added.`
			);
		}

		if (itemActivity > 0 || archiveAssets > 0) {
			if (searchJobs > 0 || embedJobs > 0) {
				parts.push(
					`${searchJobs} search job${searchJobs === 1 ? '' : 's'} and ${embedJobs} Mila embedding job${embedJobs === 1 ? '' : 's'} were enqueued after imported content was stored.`
				);
			} else if (archiveAssets > 0) {
				parts.push(
					'Archive assets were imported, but no search or Mila embedding jobs were reported for this job.'
				);
			}
		}

		return parts.length > 0 ? parts.join(' ') : null;
	});

	const outcomeVariant: Record<
		string,
		'default' | 'success' | 'warning' | 'destructive' | 'accent'
	> = {
		imported: 'success',
		updated: 'accent',
		duplicate: 'default',
		skipped_private: 'default',
		failed: 'destructive'
	};
</script>

<section class="report" data-testid="import-report">
	<header class="header">
		<h3 class="title">Import report</h3>
		<p class="subtitle">
			{job.counts.imported} imported · {job.counts.failed} failed · {job.counts.duplicate} duplicates
		</p>
	</header>

	<dl class="counts">
		<div class="count">
			<dt>Imported</dt>
			<dd>{job.counts.imported}</dd>
		</div>
		<div class="count">
			<dt>Updated</dt>
			<dd>{job.counts.updated}</dd>
		</div>
		<div class="count">
			<dt>Duplicates</dt>
			<dd>{job.counts.duplicate}</dd>
		</div>
		<div class="count">
			<dt>Skipped</dt>
			<dd>{job.counts.skipped_private}</dd>
		</div>
		<div class="count">
			<dt>Failed</dt>
			<dd>{job.counts.failed}</dd>
		</div>
	</dl>

	{#if readwiseReport}
		<section class="provider-report" aria-label="Readwise report">
			<h4 class="outcomes-heading">Readwise details</h4>
			<dl class="counts">
				<div class="count">
					<dt>Article rows</dt>
					<dd>{readwiseReport.csv_rows ?? 0}</dd>
				</div>
				<div class="count">
					<dt>Progress rows</dt>
					<dd>{readwiseReport.reading_progress_rows ?? 0}</dd>
				</div>
				<div class="count">
					<dt>Assets</dt>
					<dd>{readwiseReport.archive_assets_imported ?? 0}</dd>
				</div>
				<div class="count">
					<dt>ZIP matched</dt>
					<dd>{readwiseReport.zip_files_matched ?? 0}</dd>
				</div>
				<div class="count">
					<dt>ZIP unmatched</dt>
					<dd>{readwiseReport.zip_files_unmatched ?? 0}</dd>
				</div>
				<div class="count">
					<dt>OPML created</dt>
					<dd>{readwiseReport.opml_feeds_created ?? 0}</dd>
				</div>
				<div class="count">
					<dt>OPML skipped</dt>
					<dd>{readwiseReport.opml_feeds_skipped ?? 0}</dd>
				</div>
			</dl>

			{#if readwisePostImportMessage}
				<p class="post-import-message">{readwisePostImportMessage}</p>
			{/if}

			{#if (readwiseReport.unmatched_zip_assets ?? []).length > 0}
				<div class="outcomes">
					<h4 class="outcomes-heading">Unmatched ZIP assets</h4>
					<ul class="outcomes-list">
						{#each readwiseReport.unmatched_zip_assets ?? [] as asset (asset)}
							<li class="outcome-row">
								<span class="outcome-id" title={asset}>{asset}</span>
								<Badge variant="warning" size="sm">unmatched</Badge>
							</li>
						{/each}
					</ul>
				</div>
			{/if}

			{#if (readwiseReport.opml_errors ?? []).length > 0}
				<div class="outcomes">
					<h4 class="outcomes-heading">OPML errors</h4>
					<ul class="outcomes-list">
						{#each readwiseReport.opml_errors ?? [] as error (error)}
							<li class="outcome-row">
								<span class="outcome-error">{error}</span>
							</li>
						{/each}
					</ul>
				</div>
			{/if}
		</section>
	{/if}

	{#if job.item_outcomes.length > 0}
		<div class="outcomes">
			<h4 class="outcomes-heading">Per-item outcomes</h4>
			<ul class="outcomes-list">
				{#each visibleOutcomes as outcome (outcome.external_id)}
					<li class="outcome-row">
						<span class="outcome-id" title={outcome.external_id}
							>{outcome.title ?? outcome.external_id}</span
						>
						<Badge variant={outcomeVariant[outcome.outcome] ?? 'default'} size="sm">
							{outcome.outcome}
						</Badge>
						{#if outcome.error}
							<span class="outcome-error">{outcome.error}</span>
						{/if}
					</li>
				{/each}
			</ul>
			{#if hasMore && !showAll}
				<button type="button" class="show-all" onclick={() => (showAll = true)}>
					Show all {job.item_outcomes.length}
				</button>
			{/if}
		</div>
	{/if}

	{#if showRollback}
		<footer class="actions">
			<Button variant="destructive-outline" size="sm" onclick={onRollback}>Roll back import</Button>
		</footer>
	{/if}
</section>

<style>
	.report {
		display: flex;
		flex-direction: column;
		gap: 16px;
		padding: 16px;
		border-radius: 12px;
		background: var(--bg-secondary);
		border: 0.5px solid var(--border-primary);
	}

	.header {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.title {
		font-family: var(--font-sans);
		font-size: 15px;
		font-weight: 600;
		letter-spacing: -0.01em;
		color: var(--text-primary);
		margin: 0;
	}

	.subtitle {
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--text-secondary);
		margin: 0;
	}

	.counts {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(96px, 1fr));
		gap: 12px;
		margin: 0;
	}

	.count {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 8px 12px;
		border-radius: 8px;
		background: var(--bg-elevated);
		border: 0.5px solid var(--border-primary);
	}

	.count dt {
		font-family: var(--font-sans);
		font-size: 11px;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		color: var(--text-tertiary);
	}

	.count dd {
		font-family: var(--font-sans);
		font-size: 18px;
		font-weight: 600;
		color: var(--text-primary);
		margin: 0;
	}

	.outcomes {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.provider-report {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.post-import-message {
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--text-secondary);
		margin: 0;
		line-height: 1.4;
	}

	.outcomes-heading {
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 600;
		color: var(--text-primary);
		margin: 0;
	}

	.outcomes-list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.outcome-row {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 8px;
		border-radius: 6px;
		background: var(--bg-elevated);
		border: 0.5px solid var(--border-primary);
		font-family: var(--font-sans);
		font-size: 12px;
		flex-wrap: wrap;
	}

	.outcome-id {
		font-family: var(--font-mono, 'SF Mono', monospace);
		color: var(--text-secondary);
		max-width: 50%;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		flex: 1;
		min-width: 100px;
	}

	.outcome-error {
		color: var(--destructive);
		font-size: 11px;
	}

	.show-all {
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 500;
		color: var(--accent);
		background: transparent;
		border: none;
		cursor: pointer;
		padding: 4px 0;
		text-align: left;
	}

	.show-all:hover {
		text-decoration: underline;
	}

	.actions {
		display: flex;
		gap: 8px;
		justify-content: flex-end;
	}
</style>
