<script lang="ts">
	import Badge from '$lib/components/ui/Badge.svelte';
	import type { ImportJobStatusResponse } from '$lib/api';
	import { normalizeImportStatus } from '$lib/integrations/status';
	import { t, type MessageKey } from '$lib/i18n';

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

	const labelKeyMap = {
		awaiting_provider: 'imports_status_queued',
		pending: 'imports_status_queued',
		running: 'imports_status_running',
		completed: 'imports_status_completed',
		failed: 'imports_status_failed',
		partial: 'imports_status_partial',
		rolled_back: 'imports_status_rolled_back',
		unknown: 'imports_status_unknown'
	} as const;

	const methodLabelKey: Record<string, MessageKey> = {
		csv: 'imports_method_csv_upload',
		file_upload: 'imports_method_file_upload',
		zip: 'imports_method_file_upload',
		oauth: 'imports_method_connected_account'
	};

	function methodLabel(value: string): string {
		const key = methodLabelKey[value];
		return key ? $t(key) : value;
	}

	const source = $derived(
		job.import_source === 'readwise_import'
			? 'Readwise Reader'
			: job.import_source === 'notion_import'
				? 'Notion'
				: job.import_source
	);
	const method = $derived(methodLabel(job.import_method));
</script>

<section class="progress-card" data-testid="import-progress-card">
	<header class="header">
		<div class="title-row">
			<h3 class="title">{$t('imports_progress_title')}</h3>
			<Badge variant={variantMap[normalized]} size="sm">{$t(labelKeyMap[normalized])}</Badge>
		</div>
		<p class="meta">{source} · {method}</p>
	</header>

	{#if normalized === 'awaiting_provider'}
		<p class="meta meta-info">{$t('imports_queued')}</p>
	{/if}

	<dl class="counts">
		<div class="count">
			<dt>{$t('imports_count_imported')}</dt>
			<dd>{job.counts.imported}</dd>
		</div>
		<div class="count">
			<dt>{$t('imports_count_updated')}</dt>
			<dd>{job.counts.updated}</dd>
		</div>
		<div class="count">
			<dt>{$t('imports_count_duplicates')}</dt>
			<dd>{job.counts.duplicate}</dd>
		</div>
		<div class="count">
			<dt>{$t('imports_count_skipped')}</dt>
			<dd>{job.counts.skipped_private}</dd>
		</div>
		<div class="count">
			<dt>{$t('imports_count_failed')}</dt>
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
