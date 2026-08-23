<script lang="ts">
	import { onMount } from 'svelte';
	import type { SearchSuggestionResponse } from '$lib/api/generated/types.gen';
	import SearchAutocomplete from '$lib/components/search/SearchAutocomplete.svelte';
	import { getViewport } from '$lib/stores/viewport.svelte';
	import { t } from '$lib/i18n';
	import type { ActiveEntityFilter } from '../search-page-model';
	import SearchTipsPanel from './SearchTipsPanel.svelte';

	interface Props {
		query: string;
		hasQuery: boolean;
		activeEntityFilter: ActiveEntityFilter | null;
		suggestionsVisible: boolean;
		suggestionItems: SearchSuggestionResponse[];
		highlightedIndex: number;
		filterHints: string[];
		tipsVisible: boolean;
		detailOpen: boolean;
		onInputMount: (node: HTMLInputElement | undefined) => void;
		onSubmit: (event: SubmitEvent) => void;
		onInput: (event: Event) => void;
		onFocus: () => void;
		onKeydown: (event: KeyboardEvent) => void;
		onBlur: () => void;
		onClear: () => void;
		onSuggestionSelect: (suggestion: SearchSuggestionResponse) => void;
		onTipsVisibleChange: (visible: boolean) => void;
		onToggleDetail: () => void;
	}

	let {
		query,
		hasQuery,
		activeEntityFilter,
		suggestionsVisible,
		suggestionItems,
		highlightedIndex,
		filterHints,
		tipsVisible,
		detailOpen,
		onInputMount,
		onSubmit,
		onInput,
		onFocus,
		onKeydown,
		onBlur,
		onClear,
		onSuggestionSelect,
		onTipsVisibleChange,
		onToggleDetail
	}: Props = $props();

	const vp = getViewport();

	let inputEl = $state<HTMLInputElement | undefined>(undefined);

	onMount(() => {
		onInputMount(inputEl);
		return () => onInputMount(undefined);
	});
</script>

<div class="search-header" class:has-query={hasQuery}>
	<div class="search-bar-row">
		<button
			type="button"
			class="menu-btn"
			onclick={() => vp.openMobileNav()}
			aria-label={$t('common_open_navigation')}
		>
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.7"
				stroke-linecap="round"
				stroke-linejoin="round"
				aria-hidden="true"
			>
				<line x1="3" y1="6" x2="21" y2="6" />
				<line x1="3" y1="12" x2="21" y2="12" />
				<line x1="3" y1="18" x2="21" y2="18" />
			</svg>
		</button>
		<form class="search-input-container" class:focused={true} onsubmit={onSubmit}>
			<div class="search-icon" aria-hidden="true">
				<svg viewBox="0 0 24 24"
					><circle cx="11" cy="11" r="8" /><line x1="21" y1="21" x2="16.65" y2="16.65" /></svg
				>
			</div>
			<input
				bind:this={inputEl}
				type="text"
				class="search-input"
				placeholder={activeEntityFilter
					? $t('search_placeholder_within', { values: { name: activeEntityFilter.name } })
					: $t('search_placeholder_library')}
				value={query}
				oninput={onInput}
				onfocus={onFocus}
				onkeydown={onKeydown}
				onblur={onBlur}
				aria-label={$t('search_library_aria')}
				aria-autocomplete="list"
				autocomplete="off"
				spellcheck="false"
			/>
			{#if query || activeEntityFilter}
				<button
					type="button"
					class="search-clear"
					onclick={onClear}
					aria-label={$t('common_clear')}
				>
					<svg viewBox="0 0 24 24"
						><line x1="6" y1="6" x2="18" y2="18" /><line x1="18" y1="6" x2="6" y2="18" /></svg
					>
				</button>
			{/if}

			<button
				type="submit"
				class="search-submit"
				aria-label={$t('common_search')}
				title={$t('common_search')}
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
					<circle cx="11" cy="11" r="8" />
					<line x1="21" y1="21" x2="16.65" y2="16.65" />
				</svg>
			</button>

			{#if suggestionsVisible}
				<SearchAutocomplete
					suggestions={suggestionItems}
					{highlightedIndex}
					onSelect={onSuggestionSelect}
				/>
			{/if}
		</form>

		{#if vp.isCompact && !vp.isMobile}
			<button
				type="button"
				class="panel-toggle"
				class:active={detailOpen}
				onclick={onToggleDetail}
				aria-label={$t(detailOpen ? 'common_hide_detail_panel' : 'common_show_detail_panel')}
				title={$t(detailOpen ? 'common_hide_detail_panel' : 'common_show_detail_panel')}
			>
				<svg
					viewBox="0 0 20 20"
					fill="none"
					stroke="currentColor"
					stroke-width="1.5"
					stroke-linecap="round"
					stroke-linejoin="round"
					aria-hidden="true"
				>
					<rect x="3" y="4" width="14" height="12" rx="1.5" />
					<line x1="13" y1="4" x2="13" y2="16" />
				</svg>
			</button>
		{/if}
	</div>

	<SearchTipsPanel hints={filterHints} {tipsVisible} {onTipsVisibleChange} />

	{#if activeEntityFilter}
		<div class="filter-chips">
			<span class="filter-chip-label">{$t('search_filter_applied')}</span>
			<div class="filter-chip {activeEntityFilter.entityType.toLowerCase()}">
				<svg
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
					aria-hidden="true"
				>
					<path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
					<circle cx="12" cy="7" r="4" />
				</svg>
				entity: {activeEntityFilter.name}
				<button
					type="button"
					class="filter-chip-remove"
					aria-label={$t('search_filter_remove_entity')}
					onclick={onClear}
				>
					<svg
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2.5"
						stroke-linecap="round"
						aria-hidden="true"
					>
						<line x1="18" y1="6" x2="6" y2="18" />
						<line x1="6" y1="6" x2="18" y2="18" />
					</svg>
				</button>
			</div>
		</div>
	{/if}
</div>
