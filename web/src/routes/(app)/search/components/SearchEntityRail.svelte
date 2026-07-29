<script lang="ts">
	import type { SearchEntityCardResponse } from '$lib/api/generated/types.gen';
	import SearchEntityCard from '$lib/components/search/SearchEntityCard.svelte';

	interface Props {
		entityCard: SearchEntityCardResponse | null;
		filterActive: boolean;
		onFilter: (name: string) => void;
	}

	let { entityCard, filterActive, onFilter }: Props = $props();
</script>

{#if entityCard}
	<SearchEntityCard {entityCard} {filterActive} {onFilter} />
{/if}
