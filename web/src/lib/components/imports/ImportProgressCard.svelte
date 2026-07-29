<script lang="ts">
	import Badge from '$lib/components/ui/Badge.svelte';
	import type { ImportJobStatusResponse } from '$lib/api';
	import { normalizeImportStatus } from '$lib/integrations/status';

	interface Props {
		job: ImportJobStatusResponse;
	}

	let { job }: Props = $props();

	const normalized = $derived(normalizeImportStatus(job.status));

	const variantMap = {
		awaiting_provider: 'default',
		pending: 'accent',
		running: 'accent',
		completed: 'success',
		failed: 'destructive',
		partial: 'warning',
		rolled_back: 'default',
		unknown: 'default'
	} as const;

	const labelMap = {
		awaiting_provider: 'Queued',
		pending: 'Queued',
		running: 'Running',
		completed: 'Completed',
		failed: 'Failed',
		partial: 'Partial',
		rolled_back: 'Rolled back',
		unknown: 'Unknown'
	} as const;

	const sourceLabel: Record<string, string> = {
		readwise_import: 'Readwise Reader',
		notion_import: 'Notion'
	};

	const methodLabel: Record<string, string> = {
		csv: 'CSV upload',
		zip: 'File upload',
		oauth: 'Connected account'
	};

	const source = $derived(sourceLabel[job.import_source] ?? job.import_source);
	const method = $derived(methodLabel[job.import_method] ?? job.import_method);
</script>

<section class="progress-card" data-testid="import-progress-card">
	<header class="header">
		<div class="title-row">
			<h3 class="title">Import in progress</h3>
			<Badge variant={variantMap[normalized]} size="sm">{labelMap[normalized]}</Badge>
		</div>
		<p class="meta">{source} · {method}</p>
	</header>

	{#if normalized === 'awaiting_provider'}
		<p class="meta meta-info">Queued.</p>
	{/if}

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

	{#if job.error}
		<p class="error" role="alert">{job.error}</p>
	{/if}
</section>

<style>
	.progress-card {
		display: flex;
		flex-direction: column;
		gap: 12px;
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

	.title-row {
		display: flex;
		align-items: center;
		gap: 8px;
		flex-wrap: wrap;
	}

	.title {
		font-family: var(--font-sans);
		font-size: 15px;
		font-weight: 600;
		letter-spacing: -0.01em;
		color: var(--text-primary);
		margin: 0;
	}

	.meta {
		font-family: var(--font-sans);
		font-size: 12px;
		color: var(--text-tertiary);
		margin: 0;
	}

	.meta-info {
		color: var(--text-secondary);
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

	.error {
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--destructive);
		margin: 0;
	}
</style>
