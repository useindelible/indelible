<script lang="ts">
	import type {
		SearchEntityCardResponse,
		SearchResultResponse
	} from '$lib/api/generated/types.gen';
	import SearchResultList from '$lib/components/search/SearchResultList.svelte';
	import type { ActiveEntityFilter } from '../search-page-model';
	import SearchEntityRail from './SearchEntityRail.svelte';

	interface Props {
		resultCount: string | null;
		entityCard: SearchEntityCardResponse | null;
		activeEntityFilter: ActiveEntityFilter | null;
		results: SearchResultResponse[];
		loading: boolean;
		loadingMore: boolean;
		hasMore: boolean;
		isEmpty: boolean;
		selectedId: string | null;
		query: string;
		onEntityFilter: (name: string) => void;
		onLoadMore: () => void | Promise<void>;
		onSelect: (id: string | null) => void;
		onOpen: (result: SearchResultResponse) => void | Promise<void>;
		onSenderClick: (canonicalAddr: string) => void;
		onDetail?: (id: string) => void;
	}

	let {
		resultCount,
		entityCard,
		activeEntityFilter,
		results,
		loading,
		loadingMore,
		hasMore,
		isEmpty,
		selectedId,
		query,
		onEntityFilter,
		onLoadMore,
		onSelect,
		onOpen,
		onSenderClick,
		onDetail
	}: Props = $props();
</script>

{#if resultCount}
	<div class="results-header">
		<span class="results-count">{resultCount}</span>
	</div>
{/if}

<SearchEntityRail {entityCard} filterActive={!!activeEntityFilter} onFilter={onEntityFilter} />

<div class="results-body">
	<SearchResultList
		{results}
		{loading}
		{loadingMore}
		{hasMore}
		{isEmpty}
		{selectedId}
		{query}
		{onLoadMore}
		{onSelect}
		{onOpen}
		{onSenderClick}
		{onDetail}
	/>
</div>
