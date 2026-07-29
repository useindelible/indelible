<script lang="ts">
	import type { HighlightWithNoteResponse } from '$lib/api/generated/types.gen';
	import type { TocEntry } from './book-source';
	import { estimatePageNumber } from './book-source';

	interface Props {
		highlights: HighlightWithNoteResponse[];
		toc: TocEntry[];
		onNavigate: (chapterId: string, offset: number) => void;
	}

	let { highlights, toc, onNavigate }: Props = $props();

	const bookmarks = $derived(
		highlights
			.filter((h) => h.color === 'bookmark')
			.sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime())
	);

	const navigableEntries = $derived(
		(() => {
			const deep = toc.filter((e) => e.depth >= 2);
			return deep.length > 0 ? deep : toc;
		})()
	);

	function getBookmarkChapterLabel(bookmark: HighlightWithNoteResponse): string {
		const loc = bookmark.locator;
		if (!loc) return '';
		if (loc.type === 'pdf') {
			return `Page ${loc.page}`;
		}
		if (loc.type !== 'epub') return '';
		const entry = toc.find((e) => e.id === loc.chapter);
		if (!entry) return '';
		return `Ch. ${navigableEntries.indexOf(entry) + 1} \u00B7 ${entry.title}`;
	}

	function getBookmarkPageLabel(bookmark: HighlightWithNoteResponse): string {
		const loc = bookmark.locator;
		if (!loc) return '';
		if (loc.type === 'epub') {
			const entry = toc.find((e) => e.id === loc.chapter);
			if (entry) {
				const page = estimatePageNumber(
					entry,
					loc.start_offset ?? 0,
					entry.wordCount ? entry.wordCount * 5 : 1
				);
				return `Page ${page}`;
			}
		}
		if (loc.type === 'pdf') {
			return `Page ${loc.page}`;
		}
		return '';
	}

	function getRelativeTime(dateStr: string): string {
		const now = Date.now();
		const then = new Date(dateStr).getTime();
		const diff = now - then;
		const minutes = Math.floor(diff / 60000);
		if (minutes < 1) return 'just now';
		if (minutes < 60) return `${minutes}m ago`;
		const hours = Math.floor(minutes / 60);
		if (hours < 24) return `${hours}h ago`;
		const days = Math.floor(hours / 24);
		if (days === 1) return 'yesterday';
		if (days < 30) return `${days} days ago`;
		return new Date(dateStr).toLocaleDateString();
	}

	function handleClick(bookmark: HighlightWithNoteResponse) {
		const loc = bookmark.locator;
		if (!loc) return;
		if (loc.type === 'epub') {
			onNavigate(loc.chapter ?? '', loc.start_offset ?? 0);
		} else if (loc.type === 'pdf') {
			onNavigate(`page:${loc.page}`, 0);
		}
	}
</script>

<div class="bookmarks-list">
	{#if bookmarks.length === 0}
		<div class="bookmarks-empty">
			<p>No bookmarks yet</p>
			<p class="empty-hint">Click the bookmark icon in the toolbar to save your reading position</p>
		</div>
	{:else}
		{#each bookmarks as bookmark (bookmark.id)}
			<button type="button" class="bookmark-item" onclick={() => handleClick(bookmark)}>
				<div class="bookmark-icon">
					<svg viewBox="0 0 24 24" fill="currentColor" stroke="currentColor" stroke-width="1.5"
						><path d="M19 21l-7-5-7 5V5a2 2 0 012-2h10a2 2 0 012 2z" /></svg
					>
				</div>
				<div class="bookmark-content">
					<div class="bookmark-chapter">{getBookmarkChapterLabel(bookmark)}</div>
					<div class="bookmark-excerpt">{bookmark.text_content}</div>
					<div class="bookmark-meta">
						{getBookmarkPageLabel(bookmark)}
						{#if getBookmarkPageLabel(bookmark)}
							&middot;
						{/if}
						Bookmarked {getRelativeTime(bookmark.created_at)}
					</div>
				</div>
			</button>
		{/each}
	{/if}
</div>

<style>
	.bookmarks-list {
		display: flex;
		flex-direction: column;
	}

	.bookmarks-empty {
		padding: 32px 16px;
		text-align: center;
		color: var(--text-tertiary);
		font-family: var(--font-sans);
	}

	.bookmarks-empty p {
		font-size: 13px;
		margin: 0;
	}

	.empty-hint {
		font-size: 12px !important;
		margin-top: 4px !important;
		color: var(--text-quaternary);
	}

	.bookmark-item {
		display: flex;
		align-items: flex-start;
		gap: 10px;
		padding: 10px 16px;
		border: none;
		background: none;
		width: 100%;
		text-align: left;
		cursor: pointer;
		transition: background 120ms ease;
		font-family: var(--font-sans);
	}

	.bookmark-item:hover {
		background: var(--fill-hover);
	}

	.bookmark-icon {
		flex-shrink: 0;
		width: 16px;
		height: 16px;
		color: var(--warning, #ff9500);
		margin-top: 1px;
	}

	.bookmark-icon :global(svg) {
		width: 16px;
		height: 16px;
	}

	.bookmark-content {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 3px;
	}

	.bookmark-chapter {
		font-size: 12px;
		font-weight: 600;
		color: var(--text-primary);
		line-height: 1.35;
	}

	.bookmark-excerpt {
		font-size: 12px;
		font-weight: 400;
		color: var(--text-secondary);
		line-height: 1.45;
		display: -webkit-box;
		-webkit-line-clamp: 3;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}

	.bookmark-meta {
		font-size: 11px;
		font-weight: 400;
		color: var(--text-tertiary);
		line-height: 1.3;
	}
</style>
