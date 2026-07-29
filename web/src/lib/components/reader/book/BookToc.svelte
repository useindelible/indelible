<script lang="ts">
	import type { TocEntry } from './book-source';

	interface Props {
		toc: TocEntry[];
		currentIndex: number;
		activeEntryId: string | null;
		onNavigate: (index: number, fragment?: string) => void;
	}

	let { toc, currentIndex, activeEntryId, onNavigate }: Props = $props();

	// If all entries are the same depth (flat TOC), treat them all as chapters.
	// Otherwise use the section-header / chapter hierarchy.
	const hasNesting = $derived(toc.some((e) => e.depth > (toc[0]?.depth ?? 1)));

	const numberedToc = $derived(
		(() => {
			let num = 0;
			return toc.map((entry) => {
				const isChapter = hasNesting ? entry.depth >= 2 : true;
				if (isChapter) {
					num++;
					return { ...entry, chapterNum: num };
				}
				return { ...entry, chapterNum: null as number | null };
			});
		})()
	);

	// When activeEntryId is set (EPUB with fragment tracking), use it directly.
	// Otherwise fall back to spine-index matching (PDF or no fragment data).
	const activeEntryIndex = $derived(
		(() => {
			if (activeEntryId) return -1;
			let best = -1;
			for (const entry of toc) {
				if (entry.index <= currentIndex) best = entry.index;
			}
			return best;
		})()
	);

	function isActive(entry: TocEntry): boolean {
		if (activeEntryId) return entry.id === activeEntryId;
		return entry.index === activeEntryIndex;
	}
</script>

<div class="toc-list">
	{#each numberedToc as entry, i (entry.id)}
		{#if hasNesting && entry.depth === 1}
			{#if i > 0}
				<div class="toc-divider"></div>
			{/if}
			<button
				type="button"
				class="toc-item depth-1"
				class:active={isActive(entry)}
				onclick={() => onNavigate(entry.index, entry.fragment)}
			>
				<span class="toc-section-label">{entry.title}</span>
			</button>
		{:else}
			<button
				type="button"
				class="toc-item depth-2"
				class:active={entry.index === activeEntryIndex}
				onclick={() => onNavigate(entry.index, entry.fragment)}
			>
				{#if entry.chapterNum}
					<span class="toc-number">{entry.chapterNum}</span>
				{/if}
				<span class="toc-label">{entry.title}</span>
				{#if entry.startPage}
					<span class="toc-page">{entry.startPage}</span>
				{/if}
			</button>
		{/if}
	{/each}
</div>

<style>
	.toc-list {
		display: flex;
		flex-direction: column;
	}

	.toc-divider {
		height: 0.5px;
		background: var(--border-primary);
		margin: 8px 0;
	}

	.toc-item {
		display: flex;
		align-items: baseline;
		padding: 6px 16px;
		gap: 8px;
		border: none;
		background: none;
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

	.toc-item.depth-2.active .toc-label {
		color: var(--accent);
		font-weight: 600;
	}

	.toc-item.depth-1.active .toc-section-label {
		color: var(--accent);
	}

	.toc-section-label {
		font-weight: 600;
		font-size: 12px;
		color: var(--text-secondary);
		letter-spacing: 0.06em;
		text-transform: uppercase;
		line-height: 1.4;
	}

	.toc-number {
		font-size: 12px;
		font-weight: 500;
		color: var(--text-tertiary);
		min-width: 18px;
		flex-shrink: 0;
		font-family: var(--font-sans);
	}

	.toc-label {
		font-size: 13px;
		font-weight: 400;
		color: var(--text-primary);
		line-height: 1.35;
		flex: 1;
		min-width: 0;
	}

	.toc-page {
		font-size: 12px;
		font-weight: 400;
		color: var(--text-tertiary);
		flex-shrink: 0;
		font-family: var(--font-sans);
	}
</style>
