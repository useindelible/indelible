<script lang="ts">
	import type { SearchResultResponse } from '$lib/api/generated/types.gen';
	import { resultKey } from '$lib/stores/search.svelte';
	import SearchResultRow from './SearchResultRow.svelte';
	import ItemRowSkeleton from '$lib/components/library/ItemRowSkeleton.svelte';

	interface Props {
		results: SearchResultResponse[];
		loading: boolean;
		loadingMore: boolean;
		hasMore: boolean;
		isEmpty: boolean;
		selectedId: string | null;
		query: string;
		onLoadMore: () => void;
		onSelect: (id: string) => void;
		onOpen: (result: SearchResultResponse) => void;
		onSenderClick?: (canonicalAddr: string) => void;
		onDetail?: (id: string) => void;
	}

	let {
		results,
		loading,
		loadingMore,
		hasMore,
		isEmpty,
		selectedId,
		onLoadMore,
		onSelect,
		onOpen,
		onSenderClick,
		onDetail
	}: Props = $props();

	let sentinelEl: HTMLDivElement | undefined = $state();

	$effect(() => {
		if (!sentinelEl) return;
		const observer = new IntersectionObserver(
			(entries) => {
				if (entries[0]?.isIntersecting && hasMore && !loadingMore) {
					onLoadMore();
				}
			},
			{ threshold: 0.1 }
		);
		observer.observe(sentinelEl);
		return () => observer.disconnect();
	});
</script>

<div class="search-result-list" role="list" aria-label="Search results">
	{#if loading}
		<ItemRowSkeleton count={6} />
	{:else if isEmpty}
		<div class="no-results">
			<div class="no-results-icon" aria-hidden="true">
				<svg viewBox="0 0 24 24"
					><circle cx="11" cy="11" r="8" /><line x1="21" y1="21" x2="16.65" y2="16.65" /></svg
				>
			</div>
			<p class="no-results-title">No results found</p>
			<p class="no-results-hint">Try different keywords, remove filters, or check spelling.</p>
			<div class="no-results-suggestions">
				<span>Try:</span>
				<code>tag:</code> <code>type:</code> <code>collection:</code> to narrow your search
			</div>
		</div>
	{:else}
		{#each results as result (resultKey(result) + (result.section?.key ?? ''))}
			{@const key = resultKey(result)}
			<SearchResultRow
				{result}
				selected={selectedId === key}
				onSelect={() => onSelect(key)}
				onOpen={() => onOpen(result)}
				{onSenderClick}
				onDetail={onDetail ? () => onDetail(key) : undefined}
			/>
		{/each}
		{#if hasMore}
			<div bind:this={sentinelEl} class="sentinel" aria-hidden="true"></div>
		{/if}
		{#if loadingMore}
			<ItemRowSkeleton count={3} />
		{/if}
	{/if}
</div>

<style>
	.search-result-list {
		flex: 1;
		overflow-y: auto;
	}

	.no-results {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		padding: 60px 20px;
		text-align: center;
	}

	.no-results-icon {
		color: var(--text-quaternary);
		margin-bottom: 8px;
	}

	.no-results-icon svg {
		width: 40px;
		height: 40px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.4;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.no-results-title {
		font-family: var(--font-sans);
		font-size: 17px;
		font-weight: 600;
		letter-spacing: -0.02em;
		color: var(--text-primary);
	}

	.no-results-hint {
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 400;
		color: var(--text-secondary);
		max-width: 320px;
		line-height: 1.5;
	}

	.no-results-suggestions {
		font-family: var(--font-sans);
		font-size: 12px;
		font-weight: 400;
		color: var(--text-tertiary);
		margin-top: 8px;
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.no-results-suggestions code {
		font-family: 'SF Mono', 'Fira Code', 'Menlo', monospace;
		font-size: 10.5px;
		background: var(--fill-hover);
		padding: 1px 5px;
		border-radius: 4px;
		color: var(--text-secondary);
	}

	.sentinel {
		height: 1px;
	}
</style>
