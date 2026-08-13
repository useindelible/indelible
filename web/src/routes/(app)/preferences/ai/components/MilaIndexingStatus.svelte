<script lang="ts">
	import type { MilaStatusResponse } from '$lib/api';

	interface Props {
		status: MilaStatusResponse | null;
		embeddingModel: string;
		error?: string;
		retrying?: boolean;
		onRetry: () => void;
		onRefresh: () => void;
	}

	let {
		status,
		embeddingModel,
		error = '',
		retrying = false,
		onRetry,
		onRefresh
	}: Props = $props();

	type Variant = 'paused' | 'indexing' | 'ready' | 'attention' | 'unavailable';

	const complete = $derived(!!status && !status.is_indexing && !status.reindex_required);

	const variant = $derived<Variant>(
		error || !status
			? 'unavailable'
			: !status.enabled
				? 'paused'
				: status.is_indexing
					? 'indexing'
					: complete
						? 'ready'
						: 'attention'
	);

	const title = $derived(
		{
			paused: 'Indexing is paused',
			indexing: 'Indexing your library',
			ready: 'Your library is ready',
			attention: status?.reindex_required ? 'Indexing stopped early' : 'Library is partly indexed',
			unavailable: 'Can’t reach the index'
		}[variant]
	);

	const subtitle = $derived(
		{
			paused: 'Turn Mila on to resume. Nothing is sent to your provider while it is off.',
			indexing: 'Mila can already answer about the items that are done.',
			ready: 'Every eligible item is indexed and searchable.',
			attention: status?.reindex_required
				? 'Some items could not be indexed. Retrying picks up where Mila left off.'
				: 'Mila answers from the items that are already indexed.',
			unavailable: error || 'The status service did not respond.'
		}[variant]
	);

	// Retry stays bound to the server's own reindex_required flag rather than to
	// the visual variant, so the button appears exactly when a rebuild is useful.
	const canRetry = $derived(
		!error && !!status && status.enabled && !status.is_indexing && status.reindex_required
	);
	const progress = $derived(
		status ? Math.max(0, Math.min(100, Math.round(status.progress_percent))) : 0
	);
	// The fill gradient is painted on the progress value box, which is only
	// `progress`% as wide as the track. Scaling the gradient back up by the
	// inverse keeps the ramp spanning the whole track, so the colour at the
	// leading edge reads as how far along the run is.
	const trackScale = $derived(progress > 0 ? `${(100 / progress) * 100}%` : '100%');
</script>

<!-- A failed status read is assertive; ordinary progress stays polite. -->
<section
	class="index-card"
	data-variant={variant}
	role={variant === 'unavailable' ? 'alert' : 'status'}
	aria-label="Mila indexing status"
>
	<div class="head">
		<span class="dot" aria-hidden="true"></span>
		<div class="head-text">
			<div class="title">{title}</div>
			<div class="subtitle">{subtitle}</div>
		</div>
		{#if variant === 'unavailable'}
			<button type="button" class="action" onclick={onRefresh}>Try again</button>
		{:else if canRetry}
			<button type="button" class="action" onclick={onRetry} disabled={retrying}>
				{retrying ? 'Retrying…' : 'Retry indexing'}
			</button>
		{/if}
	</div>

	{#if status && variant !== 'unavailable'}
		<div class="meter-row">
			<div class="meter-wrap">
				<progress class="meter" max="100" value={progress} style:--track-scale={trackScale}>
					{progress}%
				</progress>
				<div class="sheen-clip" style:width="{progress}%" aria-hidden="true">
					<div class="sheen"></div>
				</div>
			</div>
			<span class="percent">{progress}%</span>
		</div>

		<div class="facts">
			<span>
				<strong>{status.indexed_items}</strong>
				of
				<strong>{status.eligible_items}</strong>
				items indexed
			</span>
			<span class="sep">·</span>
			<span class="chip">{embeddingModel}</span>
			{#if status.stale_items > 0}
				<span class="sep">·</span>
				<span class="chip warn">{status.stale_items} stale</span>
			{/if}
		</div>
	{/if}
</section>

<style>
	.index-card {
		--accent: var(--text-tertiary);
		--accent-soft: var(--mila-status-idle-bg);
		--meter-from: var(--text-quaternary);
		--meter-to: var(--text-quaternary);
		background: var(--bg-elevated);
		border-radius: 16px;
		box-shadow: var(--mila-card-shadow);
		padding: 18px 20px 16px;
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	.index-card[data-variant='indexing'] {
		--accent: var(--mila-violet);
		--accent-soft: var(--mila-violet-soft);
		--meter-from: var(--mila-violet);
		--meter-to: var(--mila-pink);
	}
	.index-card[data-variant='ready'] {
		--accent: var(--mila-status-ok-text);
		--accent-soft: var(--mila-status-ok-bg);
		--meter-from: var(--accent);
		--meter-to: var(--accent);
	}
	.index-card[data-variant='attention'] {
		--accent: var(--mila-status-warn-text);
		--accent-soft: var(--mila-status-warn-bg);
		--meter-from: var(--accent);
		--meter-to: var(--accent);
	}
	.index-card[data-variant='unavailable'] {
		--accent: var(--mila-status-err-text);
		--accent-soft: var(--mila-status-err-bg);
	}

	.head {
		display: flex;
		align-items: center;
		gap: 12px;
	}
	.dot {
		width: 9px;
		height: 9px;
		border-radius: 50%;
		flex-shrink: 0;
		background: var(--accent);
		box-shadow: 0 0 0 3px var(--accent-soft);
		transition:
			background 220ms,
			box-shadow 220ms;
	}
	.index-card[data-variant='indexing'] .dot {
		animation: pulse 1700ms ease-in-out infinite;
	}
	@keyframes pulse {
		0%,
		100% {
			box-shadow: 0 0 0 3px var(--accent-soft);
		}
		50% {
			box-shadow: 0 0 0 7px transparent;
		}
	}
	.head-text {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.title {
		font-size: 14px;
		font-weight: 600;
		color: var(--text-primary);
		letter-spacing: -0.01em;
		line-height: 1.25;
	}
	.subtitle {
		font-size: 12px;
		color: var(--text-secondary);
		letter-spacing: -0.005em;
		line-height: 1.4;
	}

	/* The action carries the state colour so retry never reads as an unrelated
	   call-to-action sitting inside an amber or red card. */
	.action {
		border: 0;
		border-radius: 9px;
		padding: 5px 10px;
		font: inherit;
		font-size: 11.5px;
		font-weight: 500;
		letter-spacing: -0.005em;
		white-space: nowrap;
		cursor: pointer;
		background: var(--accent-soft);
		color: var(--accent);
		transition: opacity 140ms;
	}
	.action:hover:not(:disabled) {
		opacity: 0.82;
	}
	.action:disabled {
		opacity: 0.45;
		cursor: default;
	}

	.meter-row {
		display: flex;
		align-items: center;
		gap: 14px;
	}
	.meter-wrap {
		--seg-mask: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='15' height='10'><rect width='12' height='10' rx='2' fill='%23000'/></svg>");
		position: relative;
		flex: 1;
		min-width: 0;
		display: flex;
	}
	/* Segmented meter: embedding runs item by item, so countable ticks match the
	   work better than a solid bar. The mask repeats at a fixed pitch, which keeps
	   the segments the same size at every width instead of a fixed segment count. */
	.meter {
		appearance: none;
		-webkit-appearance: none;
		width: 100%;
		height: 10px;
		border: 0;
		background: var(--mila-meter-empty);
		-webkit-mask-image: var(--seg-mask);
		mask-image: var(--seg-mask);
		-webkit-mask-repeat: repeat-x;
		mask-repeat: repeat-x;
		-webkit-mask-size: 15px 10px;
		mask-size: 15px 10px;
	}
	.meter::-webkit-progress-bar {
		background: var(--mila-meter-empty);
	}
	.meter::-webkit-progress-value {
		background-image: linear-gradient(90deg, var(--meter-from), var(--meter-to));
		background-size: var(--track-scale) 100%;
		transition: width 300ms ease;
	}
	.meter::-moz-progress-bar {
		background-image: linear-gradient(90deg, var(--meter-from), var(--meter-to));
		background-size: var(--track-scale) 100%;
	}
	.sheen-clip {
		position: absolute;
		top: 0;
		bottom: 0;
		left: 0;
		overflow: hidden;
		pointer-events: none;
		transition: width 300ms ease;
		-webkit-mask-image: var(--seg-mask);
		mask-image: var(--seg-mask);
		-webkit-mask-repeat: repeat-x;
		mask-repeat: repeat-x;
		-webkit-mask-size: 15px 10px;
		mask-size: 15px 10px;
	}
	.sheen {
		position: absolute;
		top: 0;
		bottom: 0;
		left: -45%;
		width: 40%;
		opacity: 0;
		background: linear-gradient(90deg, transparent, var(--mila-meter-sheen), transparent);
	}
	.index-card[data-variant='indexing'] .sheen {
		opacity: 1;
		animation: sheen 1800ms linear infinite;
	}
	@keyframes sheen {
		to {
			left: 110%;
		}
	}

	.percent {
		font-family: 'SF Mono', 'Fira Code', Menlo, ui-monospace, monospace;
		font-size: 13px;
		font-variant-numeric: tabular-nums;
		letter-spacing: -0.02em;
		color: var(--text-primary);
		min-width: 42px;
		text-align: right;
		flex-shrink: 0;
	}

	.facts {
		display: flex;
		align-items: center;
		flex-wrap: wrap;
		gap: 8px;
		font-size: 12px;
		color: var(--text-secondary);
		letter-spacing: -0.005em;
	}
	.facts strong {
		color: var(--text-primary);
		font-weight: 600;
		font-variant-numeric: tabular-nums;
	}
	.facts .sep {
		color: var(--text-quaternary);
	}
	.chip {
		font-family: 'SF Mono', 'Fira Code', Menlo, ui-monospace, monospace;
		font-size: 11px;
		padding: 3px 7px;
		border-radius: 6px;
		background: var(--mila-code-bg);
		color: var(--mila-code-text);
		letter-spacing: -0.01em;
	}
	.chip.warn {
		font-family: inherit;
		font-size: 11.5px;
		font-weight: 500;
		background: var(--mila-status-warn-bg);
		color: var(--mila-status-warn-text);
	}

	@media (prefers-reduced-motion: reduce) {
		.index-card[data-variant='indexing'] .dot,
		.index-card[data-variant='indexing'] .sheen {
			animation: none;
		}
		.meter::-webkit-progress-value,
		.sheen-clip {
			transition: none;
		}
	}

	@media (max-width: 640px) {
		.index-card {
			padding: 16px;
		}
		.head {
			flex-wrap: wrap;
			align-items: flex-start;
		}
		.dot {
			margin-top: 6px;
		}
		.action {
			margin-left: 21px;
		}
		.facts {
			font-size: 11.5px;
		}
	}
</style>
