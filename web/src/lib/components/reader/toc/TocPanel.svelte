<script lang="ts">
	import type { ArticleTocEntry } from '$lib/api';

	interface Props {
		entries: ArticleTocEntry[];
		activeIndex: number;
		progress?: number | null;
		onNavigate: (entry: ArticleTocEntry, index: number) => void;
	}

	let { entries, activeIndex, progress = null, onNavigate }: Props = $props();

	// Same convention as reading_time_minutes: 238 WPM, rounded up, minimum 1.
	function minutes(wordCount: number): number {
		return Math.max(1, Math.ceil(wordCount / 238));
	}
</script>

<nav class="toc-panel" aria-label="Table of contents">
	<div class="toc-heading-row">
		<span class="toc-heading">Contents</span>
		{#if progress != null}
			<span class="toc-progress">{Math.round(progress)}% read</span>
		{/if}
	</div>
	{#each entries as entry, index (`${entry.source_heading_index}-${entry.id}`)}
		<button
			type="button"
			class="toc-item"
			class:active={index === activeIndex}
			style:padding-left="{14 + entry.depth * 14}px"
			onclick={() => onNavigate(entry, index)}
		>
			<span class="toc-label">{entry.title}</span>
			<span class="toc-min">{minutes(entry.word_count)} min</span>
		</button>
	{/each}
</nav>

<style>
	.toc-panel {
		display: flex;
		flex-direction: column;
		padding: 10px 0 12px;
	}

	.toc-heading-row {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		padding: 4px 16px 10px;
		border-bottom: 1px solid var(--border-primary);
		margin-bottom: 6px;
	}

	.toc-heading {
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		color: var(--text-tertiary);
	}

	.toc-progress {
		font-size: 11px;
		font-weight: 500;
		color: var(--text-quaternary);
	}

	.toc-item {
		display: flex;
		align-items: baseline;
		gap: 10px;
		padding: 6px 14px;
		border: none;
		background: transparent;
		width: 100%;
		text-align: left;
		cursor: pointer;
		font-family: var(--font-sans);
		transition: background 120ms ease;
		position: relative;
	}

	.toc-item:hover {
		background: var(--fill-hover);
	}

	.toc-item.active {
		background: var(--fill-selected);
	}

	.toc-item.active::before {
		content: '';
		position: absolute;
		left: 0;
		top: 4px;
		bottom: 4px;
		width: 3px;
		border-radius: 0 2px 2px 0;
		background: var(--accent);
	}

	.toc-item.active .toc-label {
		color: var(--accent);
		font-weight: 600;
	}

	.toc-label {
		font-size: 13px;
		font-weight: 400;
		color: var(--text-primary);
		line-height: 1.35;
		flex: 1;
		min-width: 0;
	}

	.toc-min {
		font-size: 11px;
		color: var(--text-quaternary);
		flex-shrink: 0;
	}
</style>
