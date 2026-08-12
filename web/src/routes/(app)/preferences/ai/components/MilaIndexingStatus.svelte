<script lang="ts">
	import type { MilaStatusResponse } from '$lib/api';

	let {
		status,
		embeddingModel,
		retrying = false,
		onRetry
	}: {
		status: MilaStatusResponse;
		embeddingModel: string;
		retrying?: boolean;
		onRetry: () => void;
	} = $props();

	const complete = $derived(
		!status.is_indexing && status.stale_items === 0 && status.indexed_items >= status.eligible_items
	);
	const title = $derived(
		!status.enabled
			? 'Mila indexing is paused'
			: status.is_indexing
				? 'Indexing Mila library'
				: complete
					? 'Mila library is ready'
					: 'Mila indexing needs attention'
	);
</script>

<section class="indexing-status" role="status" aria-label="Mila indexing status">
	<div class="status-copy">
		<strong>{title}</strong>
		<span>{status.indexed_items} of {status.eligible_items} items indexed</span>
		<span>{embeddingModel}</span>
		{#if status.stale_items > 0}
			<span>{status.stale_items} stale</span>
		{/if}
	</div>
	<div class="progress-row">
		<progress max="100" value={status.progress_percent}>{status.progress_percent}%</progress>
		<span>{status.progress_percent}%</span>
	</div>
	{#if status.enabled && !status.is_indexing && status.reindex_required}
		<button type="button" onclick={onRetry} disabled={retrying}>
			{retrying ? 'Retrying…' : 'Retry indexing'}
		</button>
	{/if}
</section>

<style>
	.indexing-status {
		display: grid;
		gap: 12px;
		padding: 18px 20px;
		border: 1px solid var(--border-primary);
		border-radius: 12px;
		background: var(--bg-secondary);
	}
	.status-copy,
	.progress-row {
		display: flex;
		align-items: center;
		gap: 12px;
	}
	.status-copy span,
	.progress-row span {
		font-size: 12px;
		color: var(--text-secondary);
	}
	.progress-row progress {
		flex: 1;
		height: 7px;
	}
	button {
		justify-self: start;
		border: 1px solid var(--border-primary);
		border-radius: 8px;
		background: var(--bg-primary);
		color: var(--text-primary);
		padding: 7px 12px;
		font: inherit;
		cursor: pointer;
	}
	button:disabled {
		cursor: default;
		opacity: 0.6;
	}
	@media (max-width: 640px) {
		.status-copy {
			align-items: flex-start;
			flex-direction: column;
			gap: 4px;
		}
	}
</style>
