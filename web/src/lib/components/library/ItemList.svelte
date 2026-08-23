<script lang="ts">
	import { t } from '$lib/i18n';
	import type { DocumentListEntry, TriageModeDto } from '$lib/api';
	import type { TriageTab } from '$lib/stores/library.svelte';
	import ItemRow from './ItemRow.svelte';
	import ItemRowSkeleton from './ItemRowSkeleton.svelte';

	interface Props {
		items: DocumentListEntry[];
		loading: boolean;
		loadingMore: boolean;
		hasMore: boolean;
		isEmpty: boolean;
		selectedId: string | null;
		triageTab: TriageTab;
		triageMode?: TriageModeDto;
		onLoadMore: () => void;
		onSelect: (id: string) => void;
		onOpen: (id: string) => void;
		onTriage: (id: string, state: TriageTab) => void;
		onDelete?: (id: string) => void;
		onDetail?: (id: string) => void;
		showFeedBadge?: boolean;
	}

	let {
		items,
		loading,
		loadingMore,
		hasMore,
		isEmpty,
		selectedId,
		triageTab,
		triageMode = 'focus',
		onLoadMore,
		onSelect,
		onOpen,
		onTriage,
		onDelete,
		onDetail,
		showFeedBadge = false
	}: Props = $props();

	let sentinel = $state<HTMLDivElement | undefined>(undefined);

	$effect(() => {
		if (!sentinel) return;

		const observer = new IntersectionObserver(
			(entries) => {
				if (entries[0]?.isIntersecting && hasMore && !loadingMore) {
					onLoadMore();
				}
			},
			{ threshold: 0.1 }
		);

		observer.observe(sentinel);
		return () => observer.disconnect();
	});

	const emptyLabels = $derived<Record<TriageTab, { heading: string; sub: string }>>(
		triageMode === 'manual'
			? {
					inbox: {
						heading: $t('library_empty_saved_heading'),
						sub: $t('library_empty_start_sub')
					},
					later: {
						heading: $t('library_empty_saved_heading'),
						sub: $t('library_empty_start_sub')
					},
					archive: {
						heading: $t('library_empty_archive_heading'),
						sub: $t('library_empty_archive_sub')
					}
				}
			: {
					inbox: {
						heading: $t('library_empty_inbox_heading'),
						sub: $t('library_empty_start_sub')
					},
					later: {
						heading: $t('library_empty_later_heading'),
						sub: $t('library_empty_later_sub')
					},
					archive: {
						heading: $t('library_empty_archive_heading'),
						sub: $t('library_empty_archive_sub')
					}
				}
	);
</script>

<div class="item-list" role="listbox" aria-label={$t('library_item_list')}>
	{#if loading && items.length === 0}
		<ItemRowSkeleton count={6} />
	{:else if isEmpty}
		<div class="empty-state">
			<p class="empty-heading">{emptyLabels[triageTab].heading}</p>
			<p class="empty-sub">{emptyLabels[triageTab].sub}</p>
		</div>
	{:else}
		{#key items[0]?.id}
			<div class="item-list-body">
				{#each items as item (item.id)}
					<ItemRow
						{item}
						selected={selectedId === item.id}
						onSelect={() => onSelect(item.id)}
						onOpen={() => onOpen(item.id)}
						onTriage={(state) => onTriage(item.id, state)}
						onDelete={onDelete ? () => onDelete(item.id) : undefined}
						onDetail={onDetail ? () => onDetail(item.id) : undefined}
						{triageMode}
						{showFeedBadge}
					/>
				{/each}
			</div>
		{/key}

		{#if loadingMore}
			<ItemRowSkeleton count={3} />
		{/if}

		<!-- IntersectionObserver sentinel -->
		<div bind:this={sentinel} class="sentinel" aria-hidden="true"></div>
	{/if}
</div>

<style>
	.item-list {
		flex: 1;
		overflow-y: auto;
		position: relative;
	}

	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 6px;
		height: 200px;
		padding: 40px 20px;
	}

	.empty-heading {
		font-family: var(--font-sans);
		font-size: 15px;
		font-weight: 600;
		letter-spacing: -0.01em;
		color: var(--text-primary);
		margin: 0;
	}

	.empty-sub {
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 400;
		color: var(--text-secondary);
		margin: 0;
	}

	.item-list-body {
		animation: list-fade-in 0.18s ease both;
	}

	@keyframes list-fade-in {
		from {
			opacity: 0;
		}
		to {
			opacity: 1;
		}
	}

	.sentinel {
		height: 1px;
	}
</style>
