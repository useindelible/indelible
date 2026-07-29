<script lang="ts">
	import type { BookSource } from './book-source';
	import type { HighlightWithNoteResponse } from '$lib/api/generated/types.gen';
	import BookToc from './BookToc.svelte';
	import BookBookmarks from './BookBookmarks.svelte';
	import BookSearch from './BookSearch.svelte';

	export type SidebarTab = 'contents' | 'bookmarks' | 'search';

	interface Props {
		source: BookSource;
		currentIndex: number;
		activeEntryId: string | null;
		progress: number;
		highlights: HighlightWithNoteResponse[];
		activeTab: SidebarTab;
		onTabChange: (tab: SidebarTab) => void;
		onNavigate: (index: number, fragment?: string) => void;
		onBookmarkNavigate: (chapterId: string, offset: number) => void;
		thumbnailUrl?: string | null;
	}

	let {
		source,
		currentIndex,
		activeEntryId,
		progress,
		highlights,
		activeTab,
		onTabChange,
		onNavigate,
		onBookmarkNavigate,
		thumbnailUrl = null
	}: Props = $props();

	let thumbnailFailed = $state(false);
	let previousThumbnailUrl = $state<string | null>(null);
	const coverThumbnailUrl = $derived(thumbnailUrl?.trim() || null);

	$effect(() => {
		if (coverThumbnailUrl !== previousThumbnailUrl) {
			previousThumbnailUrl = coverThumbnailUrl;
			thumbnailFailed = false;
		}
	});

	const tabs: { value: SidebarTab; label: string }[] = [
		{ value: 'contents', label: 'Contents' },
		{ value: 'bookmarks', label: 'Bookmarks' },
		{ value: 'search', label: 'Search' }
	];
</script>

<div class="left-panel">
	<div class="left-header">
		<div class="book-info">
			<div class="book-cover">
				{#if coverThumbnailUrl && !thumbnailFailed}
					<img
						class="book-cover-image"
						src={coverThumbnailUrl}
						alt=""
						loading="lazy"
						decoding="async"
						onerror={() => {
							thumbnailFailed = true;
						}}
					/>
				{:else}
					<div class="book-cover-inner">
						<svg
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="1.5"
							stroke-linecap="round"
							stroke-linejoin="round"
							><path d="M2 3h6a4 4 0 014 4v14a3 3 0 00-3-3H2z" /><path
								d="M22 3h-6a4 4 0 00-4 4v14a3 3 0 013-3h7z"
							/></svg
						>
					</div>
				{/if}
			</div>
			<div class="book-details">
				<div class="book-title">{source.metadata.title ?? 'Untitled'}</div>
				<div class="book-author">{source.metadata.author ?? 'Unknown author'}</div>
				<div class="book-progress-mini">
					<div class="progress-bar-mini">
						<div class="progress-bar-mini-fill" style:width="{Math.round(progress)}%"></div>
					</div>
					<span class="progress-text-mini">{Math.round(progress)}%</span>
				</div>
			</div>
		</div>
	</div>

	<div class="panel-tabs">
		{#each tabs as tab (tab.value)}
			<button
				type="button"
				class="panel-tab"
				class:active={activeTab === tab.value}
				onclick={() => onTabChange(tab.value)}
			>
				{tab.label}
			</button>
		{/each}
	</div>

	<div class="left-body">
		{#if activeTab === 'contents'}
			<BookToc toc={source.toc} {currentIndex} {activeEntryId} {onNavigate} />
		{:else if activeTab === 'bookmarks'}
			<BookBookmarks {highlights} toc={source.toc} onNavigate={onBookmarkNavigate} />
		{:else if activeTab === 'search'}
			<BookSearch {source} {onNavigate} />
		{/if}
	</div>
</div>

<style>
	.left-panel {
		width: 280px;
		min-width: 280px;
		display: flex;
		flex-direction: column;
		background: var(--sidebar-bg);
		backdrop-filter: blur(40px) saturate(180%);
		-webkit-backdrop-filter: blur(40px) saturate(180%);
		border-right: 0.5px solid var(--border-primary);
		position: relative;
		z-index: 2;
	}

	.left-header {
		padding: 16px 16px 12px;
		flex-shrink: 0;
	}

	.book-info {
		display: flex;
		gap: 12px;
		align-items: flex-start;
	}

	.book-cover {
		width: 48px;
		height: 64px;
		border-radius: 4px;
		background: var(--bg-tertiary);
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
		position: relative;
		overflow: hidden;
		box-shadow: 0 1px 4px rgba(0, 0, 0, 0.12);
	}

	.book-cover-inner {
		color: var(--text-tertiary);
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.book-cover-inner :global(svg) {
		width: 24px;
		height: 24px;
	}

	.book-cover-image {
		width: 100%;
		height: 100%;
		display: block;
		object-fit: cover;
	}

	.book-details {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.book-title {
		font-size: 13px;
		font-weight: 600;
		color: var(--text-primary);
		line-height: 1.3;
		display: -webkit-box;
		-webkit-line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
		font-family: var(--font-sans);
	}

	.book-author {
		font-size: 12px;
		font-weight: 400;
		color: var(--text-secondary);
		font-family: var(--font-sans);
	}

	.book-progress-mini {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-top: 4px;
	}

	.progress-bar-mini {
		flex: 1;
		height: 3px;
		background: var(--seg-bg);
		border-radius: 2px;
		overflow: hidden;
	}

	.progress-bar-mini-fill {
		height: 100%;
		background: var(--accent);
		border-radius: 2px;
		transition: width 300ms ease;
	}

	.progress-text-mini {
		font-size: 11px;
		font-weight: 500;
		color: var(--text-tertiary);
		flex-shrink: 0;
		font-family: var(--font-sans);
	}

	.panel-tabs {
		display: flex;
		padding: 0 12px;
		gap: 0;
		border-bottom: 0.5px solid var(--border-primary);
		flex-shrink: 0;
	}

	.panel-tab {
		flex: 1;
		padding: 8px 0;
		font-size: 12px;
		font-weight: 500;
		color: var(--text-tertiary);
		text-align: center;
		border: none;
		background: none;
		cursor: pointer;
		transition: color 120ms ease;
		position: relative;
		font-family: var(--font-sans);
	}

	.panel-tab:hover {
		color: var(--text-secondary);
	}

	.panel-tab.active {
		color: var(--text-primary);
		font-weight: 600;
	}

	.panel-tab.active::after {
		content: '';
		position: absolute;
		bottom: -0.5px;
		left: 8px;
		right: 8px;
		height: 2px;
		background: var(--accent);
		border-radius: 1px;
	}

	.left-body {
		flex: 1;
		overflow-y: auto;
		overflow-x: hidden;
		padding: 8px 0;
	}

	.left-body::-webkit-scrollbar {
		width: 4px;
	}

	.left-body::-webkit-scrollbar-track {
		background: transparent;
	}

	.left-body::-webkit-scrollbar-thumb {
		background: var(--text-quaternary);
		border-radius: 2px;
	}
</style>
