<script lang="ts">
	import type { TocEntry } from './book-source';

	interface Props {
		toc: TocEntry[];
		currentIndex: number;
		totalChapters: number;
		onPrev: () => void;
		onNext: () => void;
		isPdf?: boolean;
	}

	let { toc, currentIndex, totalChapters, onPrev, onNext, isPdf = false }: Props = $props();

	const depth2Entries = $derived(
		(() => {
			const deep = toc.filter((e) => e.depth >= 2);
			return deep.length > 0 ? deep : toc;
		})()
	);
	const currentChapterIndex = $derived(depth2Entries.findIndex((e) => e.index === currentIndex));
	const currentChapterNum = $derived(
		currentChapterIndex >= 0 ? currentChapterIndex + 1 : currentIndex + 1
	);

	const prevEntry = $derived(
		(() => {
			const idx = depth2Entries.findIndex((e) => e.index === currentIndex);
			return idx > 0 ? depth2Entries[idx - 1] : null;
		})()
	);

	const nextEntry = $derived(
		(() => {
			const idx = depth2Entries.findIndex((e) => e.index === currentIndex);
			return idx >= 0 && idx < depth2Entries.length - 1 ? depth2Entries[idx + 1] : null;
		})()
	);

	const hasPrev = $derived(isPdf ? currentIndex > 0 : prevEntry != null);
	const hasNext = $derived(isPdf ? currentIndex < totalChapters - 1 : nextEntry != null);

	function prevLabel(): string {
		if (isPdf) return `Page ${currentIndex}`;
		if (!prevEntry) return '';
		const num = depth2Entries.indexOf(prevEntry) + 1;
		return `Ch. ${num}: ${prevEntry.title}`;
	}

	function nextLabel(): string {
		if (isPdf) return `Page ${currentIndex + 2}`;
		if (!nextEntry) return '';
		const num = depth2Entries.indexOf(nextEntry) + 1;
		return `Ch. ${num}: ${nextEntry.title}`;
	}

	function centerLabel(): string {
		if (isPdf) return `Page ${currentIndex + 1} of ${totalChapters}`;
		return `${currentChapterNum} of ${depth2Entries.length} chapters`;
	}
</script>

<div class="page-nav">
	<button type="button" class="page-nav-btn prev" disabled={!hasPrev} onclick={onPrev}>
		<svg
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"><polyline points="15 18 9 12 15 6" /></svg
		>
		<span class="page-nav-label">{prevLabel()}</span>
	</button>
	<div class="page-nav-center">{centerLabel()}</div>
	<button type="button" class="page-nav-btn next" disabled={!hasNext} onclick={onNext}>
		<span class="page-nav-label">{nextLabel()}</span>
		<svg
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"><polyline points="9 18 15 12 9 6" /></svg
		>
	</button>
</div>

<style>
	.page-nav {
		height: 44px;
		min-height: 44px;
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0 16px;
		border-top: 0.5px solid var(--border-primary);
		background: var(--bg-content);
		flex-shrink: 0;
	}

	.page-nav-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 10px;
		border-radius: 7px;
		border: none;
		background: none;
		cursor: pointer;
		color: var(--text-secondary);
		font-size: 12px;
		font-weight: 500;
		font-family: var(--font-sans);
		transition: all 120ms ease;
		max-width: 280px;
	}

	.page-nav-btn:hover:not(:disabled) {
		background: var(--fill-hover);
		color: var(--text-primary);
	}

	.page-nav-btn:disabled {
		opacity: 0.3;
		cursor: default;
	}

	.page-nav-btn :global(svg) {
		width: 14px;
		height: 14px;
		flex-shrink: 0;
	}

	.page-nav-label {
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.page-nav-center {
		font-size: 12px;
		font-weight: 500;
		color: var(--text-tertiary);
		white-space: nowrap;
		font-family: var(--font-sans);
	}

	/* Mobile keeps the arrows and the position readout; chapter titles yield. */
	@media (max-width: 599px) {
		.page-nav {
			padding: 0 8px;
		}

		.page-nav-label {
			display: none;
		}

		.page-nav-btn {
			padding: 6px 12px;
		}
	}
</style>
