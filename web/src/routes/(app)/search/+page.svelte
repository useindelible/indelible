<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { untrack } from 'svelte';
	import { getSearch, resultKey } from '$lib/stores/search.svelte';
	import { getViewport } from '$lib/stores/viewport.svelte';
	import { openSearchResult } from '$lib/components/search/open-result';
	import LibraryShell from '$lib/components/library/LibraryShell.svelte';
	import LibrarySidebar from '$lib/components/library/LibrarySidebar.svelte';
	import DetailPanel from '$lib/components/library/DetailPanel.svelte';
	import type { SearchSuggestionResponse } from '$lib/api/generated/types.gen';
	import {
		buildSearchQuery,
		FILTER_HINTS,
		parseEntityPrefix,
		type ActiveEntityFilter
	} from './search-page-model';
	import SearchHeader from './components/SearchHeader.svelte';
	import SearchRecentList from './components/SearchRecentList.svelte';
	import SearchResultsPane from './components/SearchResultsPane.svelte';
	import './components/search-page.css';

	const search = getSearch();
	const vp = getViewport();

	// Below the desktop breakpoint the docked detail panel becomes a slide-over
	// (tablet) or full-screen view (mobile); session-only, matching the library list.
	let compactDetailOpen = $state(false);

	let inputEl: HTMLInputElement | undefined = $state();
	let initialized = $state(false);
	let activeEntityFilter = $state<ActiveEntityFilter | null>(null);
	let tipsVisible = $state(false);

	$effect(() => {
		const urlQuery = page.url.searchParams.get('q');
		if (!initialized) {
			initialized = true;
			untrack(() => {
				if (urlQuery) {
					const parsed = parseEntityPrefix(urlQuery);
					if (parsed) {
						activeEntityFilter = { name: parsed.entityName, entityType: parsed.entityType };
						search.query = parsed.remainder;
					} else {
						search.query = urlQuery;
					}
					search.submitSearch(urlQuery);
				} else {
					search.loadRecentSearches();
				}
			});
		}
	});

	$effect(() => {
		if (activeEntityFilter && !activeEntityFilter.entityType && search.entityCard) {
			activeEntityFilter = {
				...activeEntityFilter,
				entityType: search.entityCard.entity_type
			};
		}
	});

	$effect(() => {
		if (inputEl) {
			inputEl.focus();
		}
	});

	function updateUrl(q: string) {
		const url = new URL(page.url);
		if (q.trim()) {
			url.searchParams.set('q', q.trim());
		} else {
			url.searchParams.delete('q');
		}
		// eslint-disable-next-line svelte/no-navigation-without-resolve -- URL is derived from page.url which already includes the base path
		goto(url.toString(), { replaceState: true, keepFocus: true });
	}

	function buildQuery(userInput: string): string {
		return buildSearchQuery(userInput, activeEntityFilter);
	}

	function submitQuery(q: string) {
		if (!q) return;
		updateUrl(q);
		search.submitSearch(q);
		if (activeEntityFilter) {
			search.query = search.query.trim();
		}
	}

	function handleSubmit(e: SubmitEvent) {
		e.preventDefault();
		search.hideSuggestions();
		submitQuery(buildQuery(search.query));
	}

	function handleInput(e: Event) {
		const target = e.target as HTMLInputElement;
		search.query = target.value;
	}

	function handleClear() {
		search.clearSearch();
		updateUrl('');
		search.loadRecentSearches();
		activeEntityFilter = null;
		inputEl?.focus();
	}

	function handleSenderClick(canonicalAddr: string) {
		const q = `sender:${canonicalAddr}`;
		search.query = q;
		updateUrl(q);
		search.submitSearch(q);
	}

	function handleSuggestionSelect(suggestion: SearchSuggestionResponse) {
		search.applySuggestion(suggestion);
		const q = buildQuery(search.query);
		if (q) {
			submitQuery(q);
		} else {
			inputEl?.focus();
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (search.suggestionsVisible) {
			const count = search.suggestionItems.length;
			if (e.key === 'ArrowDown') {
				e.preventDefault();
				search.highlightedIndex =
					search.highlightedIndex < count - 1 ? search.highlightedIndex + 1 : 0;
			} else if (e.key === 'ArrowUp') {
				e.preventDefault();
				search.highlightedIndex =
					search.highlightedIndex > 0 ? search.highlightedIndex - 1 : count - 1;
			} else if (e.key === 'Enter' && search.highlightedIndex >= 0) {
				e.preventDefault();
				const selected = search.suggestionItems[search.highlightedIndex];
				if (selected) handleSuggestionSelect(selected);
			} else if (e.key === 'Escape') {
				search.hideSuggestions();
			}
		}
		// Enter with no suggestion highlighted must always run the search:
		// leaving it to the browser's implicit form submission silently breaks
		// as soon as the suggestion dropdown intercepts the keystroke.
		if (e.key === 'Enter' && !e.defaultPrevented) {
			e.preventDefault();
			search.hideSuggestions();
			submitQuery(buildQuery(search.query));
		}
	}

	function handleRecentClick(q: string) {
		search.query = q;
		updateUrl(q);
		search.submitSearch(q);
	}

	function handleEntityFilter(name: string) {
		activeEntityFilter = {
			name,
			entityType: search.entityCard?.entity_type ?? ''
		};
		const q = `entity:"${name}"`;
		updateUrl(q);
		search.submitSearch(q);
		search.query = '';
		inputEl?.focus();
	}

	function handleBlur() {
		setTimeout(() => search.hideSuggestions(), 200);
	}

	$effect(() => {
		function onKeydown(e: KeyboardEvent) {
			const target = e.target as HTMLElement;
			if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable)
				return;

			const items = search.results;
			const idx = items.findIndex((r) => resultKey(r) === search.selectedId);

			switch (e.key) {
				case 'j':
				case 'ArrowDown': {
					e.preventDefault();
					const next = idx < items.length - 1 ? idx + 1 : idx;
					const nextItem = items[next];
					search.setSelectedId(nextItem ? resultKey(nextItem) : null);
					break;
				}
				case 'k':
				case 'ArrowUp': {
					e.preventDefault();
					const prev = idx > 0 ? idx - 1 : 0;
					const prevItem = items[prev];
					search.setSelectedId(prevItem ? resultKey(prevItem) : null);
					break;
				}
			}
		}

		document.addEventListener('keydown', onKeydown);
		return () => document.removeEventListener('keydown', onKeydown);
	});

	function openResultDetail(id: string) {
		search.setSelectedId(id);
		compactDetailOpen = true;
	}

	const detailItem = $derived(search.selectedEntry);
</script>

{#snippet sidebar()}
	<LibrarySidebar />
{/snippet}

{#snippet content()}
	<div class="search-route-panel">
		<SearchHeader
			query={search.query}
			hasQuery={!!search.submittedQuery}
			{activeEntityFilter}
			suggestionsVisible={search.suggestionsVisible}
			suggestionItems={search.suggestionItems}
			highlightedIndex={search.highlightedIndex}
			filterHints={FILTER_HINTS}
			{tipsVisible}
			detailOpen={compactDetailOpen}
			onInputMount={(node) => {
				inputEl = node;
			}}
			onSubmit={handleSubmit}
			onInput={handleInput}
			onFocus={() => search.showSuggestions()}
			onKeydown={handleKeydown}
			onBlur={handleBlur}
			onClear={handleClear}
			onSuggestionSelect={handleSuggestionSelect}
			onTipsVisibleChange={(visible) => {
				tipsVisible = visible;
			}}
			onToggleDetail={() => (compactDetailOpen = !compactDetailOpen)}
		/>

		{#if search.submittedQuery}
			<SearchResultsPane
				resultCount={search.resultCount}
				entityCard={search.entityCard}
				{activeEntityFilter}
				results={search.results}
				loading={search.loading}
				loadingMore={search.loadingMore}
				hasMore={search.hasMore}
				isEmpty={search.isEmpty}
				selectedId={search.selectedId}
				query={search.submittedQuery}
				onEntityFilter={handleEntityFilter}
				onLoadMore={() => search.loadMore()}
				onSelect={(id) => search.setSelectedId(id)}
				onOpen={openSearchResult}
				onSenderClick={handleSenderClick}
				onDetail={vp.isMobile ? openResultDetail : undefined}
			/>
		{:else}
			<SearchRecentList
				recentLoading={search.recentLoading}
				recentSearches={search.recentSearches}
				onRecentClick={handleRecentClick}
				onClearAll={() => search.clearAllRecent()}
				onDeleteRecent={(id) => search.deleteRecent(id)}
			/>
		{/if}
	</div>

	{#if vp.isCompact}
		{#if compactDetailOpen}
			{#if vp.isMobile}
				<div class="m-detail">
					<div class="m-detailbar">
						<button
							type="button"
							class="m-back"
							onclick={() => (compactDetailOpen = false)}
							aria-label="Back to results"
						>
							<svg
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
								stroke-linecap="round"
								stroke-linejoin="round"
								aria-hidden="true"
							>
								<polyline points="15 18 9 12 15 6" />
							</svg>
						</button>
						<span class="m-dtitle">{detailItem?.title ?? 'Details'}</span>
					</div>
					<DetailPanel item={detailItem} />
				</div>
			{:else}
				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div class="detail-scrim" onclick={() => (compactDetailOpen = false)}></div>
				<div class="detail-overlay">
					<DetailPanel item={detailItem} />
				</div>
			{/if}
		{/if}
	{:else}
		<DetailPanel item={detailItem} />
	{/if}
{/snippet}

<LibraryShell {sidebar} {content} />

<style>
	/* Compact detail surfaces live at the page level, beside the route panel. */
	.detail-scrim {
		position: absolute;
		inset: 0;
		background: rgba(0, 0, 0, 0.1);
		z-index: 20;
	}

	/* Opaque surface: the docked panel's vibrancy blur would let the results
	   bleed through when it floats above them. */
	.detail-overlay {
		position: absolute;
		top: 0;
		right: 0;
		bottom: 0;
		width: 330px;
		z-index: 21;
		display: flex;
		background: var(--bg-elevated);
		box-shadow: -18px 0 56px rgba(0, 0, 0, 0.18);
	}

	.detail-overlay :global(.detail-panel) {
		width: 100%;
		min-width: 0;
		background: var(--bg-elevated);
		backdrop-filter: none;
		-webkit-backdrop-filter: none;
	}

	.m-detail {
		position: absolute;
		inset: 0;
		z-index: 21;
		display: flex;
		flex-direction: column;
		background: var(--bg-content);
	}

	.m-detailbar {
		height: 52px;
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 0 8px;
		border-bottom: 0.5px solid var(--border-primary);
		flex-shrink: 0;
		background: var(--bg-content);
	}

	.m-back {
		width: 34px;
		height: 34px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: var(--radius-sm);
		border: none;
		background: transparent;
		color: var(--text-secondary);
		cursor: pointer;
		flex-shrink: 0;
	}

	.m-back:hover {
		background: var(--fill-hover);
	}

	.m-back svg {
		width: 20px;
		height: 20px;
	}

	.m-dtitle {
		font-size: 15px;
		font-weight: 600;
		letter-spacing: -0.01em;
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		min-width: 0;
	}

	.m-detail :global(.detail-panel) {
		width: 100%;
		min-width: 0;
		flex: 1;
		border-left: none;
		background: var(--bg-content);
		backdrop-filter: none;
		-webkit-backdrop-filter: none;
	}
</style>
