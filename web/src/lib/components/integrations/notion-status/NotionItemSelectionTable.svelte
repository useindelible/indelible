<script lang="ts">
	import type { NotionExportItemDto } from '$lib/api';
	import { formatExportedAt, formatItemType } from './notion-status-model';
	import { t } from '$lib/i18n';

	interface Props {
		visible?: boolean;
		items?: NotionExportItemDto[];
		itemsLoading?: boolean;
		itemsError?: string | null;
		itemsQuery?: string;
		itemsHasNext?: boolean;
		savingItemId?: string | null;
		refreshingItemId?: string | null;
		refreshNotice?: { message: string; archivedPageUrl?: string | null } | null;
		selectedCount: number;
		exportItemsMeta: string;
		onItemsSearch: (query: string) => void;
		onItemsLoadMore: () => void;
		onItemSelection: (itemId: string, selected: boolean) => void;
		onRefreshItem: (item: NotionExportItemDto) => void;
	}

	let {
		visible = false,
		items = [],
		itemsLoading = false,
		itemsError = null,
		itemsQuery = '',
		itemsHasNext = false,
		savingItemId = null,
		refreshingItemId = null,
		refreshNotice = null,
		selectedCount,
		exportItemsMeta,
		onItemsSearch,
		onItemsLoadMore,
		onItemSelection,
		onRefreshItem
	}: Props = $props();

	let loadMoreSentinel = $state<HTMLDivElement | null>(null);

	$effect(() => {
		if (!loadMoreSentinel || !itemsHasNext || itemsLoading) return;

		const observer = new IntersectionObserver(
			([entry]) => {
				if (entry?.isIntersecting) onItemsLoadMore();
			},
			{ rootMargin: '240px 0px' }
		);
		observer.observe(loadMoreSentinel);

		return () => observer.disconnect();
	});
</script>

<div class="items-section" class:visible>
	<div class="group-label">{$t('integrations_notion_documents_to_export')}</div>
	<div class="group-desc">
		{$t('integrations_notion_documents_to_export_description')}
	</div>
	<div class="group-card">
		{#if refreshNotice}
			<div class="refresh-notice" role="status">
				<span>{refreshNotice.message}</span>
				{#if refreshNotice.archivedPageUrl}
					<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- Notion returns this external page URL. -->
					<a href={refreshNotice.archivedPageUrl} target="_blank" rel="noopener noreferrer">
						{$t('integrations_notion_open_archived_page')}
					</a>
				{/if}
			</div>
		{/if}
		<div class="items-toolbar">
			<div class="search-wrap">
				<span class="search-icon">
					<svg viewBox="0 0 24 24"
						><circle cx="11" cy="11" r="7" /><path d="M21 21l-4.35-4.35" /></svg
					>
				</span>
				<input
					class="search-input"
					type="search"
					placeholder={$t('integrations_notion_search_documents')}
					value={itemsQuery}
					oninput={(event) => onItemsSearch(event.currentTarget.value)}
				/>
			</div>
			<div class="toolbar-meta" data-testid="notion-items-meta">
				{$t('integrations_notion_selected_meta', {
					values: { count: selectedCount, meta: exportItemsMeta }
				})}
			</div>
		</div>

		{#if itemsError}
			<div class="callout error" role="alert">
				<div class="callout-body">
					<div class="callout-title">{$t('integrations_notion_update_documents_failed')}</div>
					<div class="callout-detail">{itemsError}</div>
				</div>
			</div>
		{/if}

		<table class="items-table">
			<thead>
				<tr>
					<th class="col-check"></th>
					<th>{$t('integrations_notion_document')}</th>
					<th class="col-type">{$t('common_type')}</th>
					<th class="col-last">{$t('integrations_notion_last_exported')}</th>
					<th class="col-action"></th>
				</tr>
			</thead>
			<tbody>
				{#if itemsLoading && items.length === 0}
					<tr>
						<td colspan="5" class="empty-row">{$t('integrations_notion_loading_documents')}</td>
					</tr>
				{:else if items.length === 0}
					<tr>
						<td colspan="5" class="empty-row">{$t('integrations_notion_no_matching_documents')}</td>
					</tr>
				{:else}
					{#each items as item (item.library_entry_id)}
						<tr>
							<td class="col-check">
								<button
									type="button"
									class="row-checkbox"
									class:checked={item.selected}
									aria-label={$t(
										item.selected ? 'integrations_notion_deselect' : 'integrations_notion_select'
									)}
									aria-pressed={item.selected}
									disabled={savingItemId === item.library_entry_id}
									onclick={() => onItemSelection(item.library_entry_id, !item.selected)}
								></button>
							</td>
							<td>
								<div class="item-cell-title">{item.title || $t('reader_untitled')}</div>
								{#if item.url}
									<div class="item-cell-url">{item.url}</div>
								{/if}
								{#if item.last_error}
									<div class="item-cell-error">{item.last_error}</div>
								{/if}
							</td>
							<td>
								<span class="type-pill">{formatItemType(item.item_type, $t)}</span>
							</td>
							<td class="col-last">{formatExportedAt(item.last_synced_at)}</td>
							<td class="col-action">
								<button
									class="text-action"
									type="button"
									disabled={refreshingItemId === item.library_entry_id || !item.exported_page_id}
									onclick={() => onRefreshItem(item)}
								>
									{refreshingItemId === item.library_entry_id
										? $t('integrations_notion_refreshing')
										: item.exported_page_id
											? $t('integrations_notion_refresh')
											: $t('integrations_notion_not_exported')}
								</button>
							</td>
						</tr>
					{/each}
				{/if}
			</tbody>
		</table>

		<div bind:this={loadMoreSentinel} class="load-more-sentinel" aria-hidden="true"></div>

		<div class="pager">
			<span class="pager-page">{exportItemsMeta}</span>
			<button
				class="btn"
				type="button"
				disabled={!itemsHasNext || itemsLoading}
				onclick={onItemsLoadMore}
			>
				{itemsLoading && items.length > 0
					? $t('common_loading')
					: $t('integrations_notion_load_more')}
			</button>
		</div>
	</div>
</div>

<style>
	.items-section {
		display: none;
		margin-bottom: 28px;
	}

	.items-section.visible {
		display: block;
	}

	.group-label {
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		color: var(--text-tertiary);
		padding: 0 4px 4px;
	}

	.group-desc {
		font-size: 12.5px;
		color: var(--text-secondary);
		padding: 0 4px 10px;
		line-height: 1.45;
	}

	.group-card {
		background: var(--bg-elevated);
		border-radius: 14px;
		overflow: hidden;
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		container-type: inline-size;
		container-name: settings-card;
	}

	.refresh-notice {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
		padding: 12px 18px;
		border-bottom: 0.5px solid var(--border-primary);
		font-size: 12px;
		color: var(--text-secondary);
	}

	.refresh-notice a {
		color: var(--accent);
		white-space: nowrap;
	}

	.items-toolbar,
	.pager {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
		padding: 14px 18px;
	}

	.items-toolbar {
		border-bottom: 0.5px solid var(--border-primary);
	}

	.search-wrap {
		position: relative;
		flex: 1;
		min-width: 220px;
	}

	.search-icon {
		position: absolute;
		left: 10px;
		top: 50%;
		transform: translateY(-50%);
		color: var(--text-tertiary);
	}

	.search-icon svg {
		width: 14px;
		height: 14px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.8;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.search-input {
		width: 100%;
		border: none;
		border-radius: 8px;
		background: var(--bg-secondary);
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		color: var(--text-primary);
		font: inherit;
		font-size: 13px;
		padding: 8px 10px 8px 32px;
	}

	.toolbar-meta,
	.pager-page {
		font-size: 12px;
		color: var(--text-secondary);
		white-space: nowrap;
	}

	.items-table {
		width: 100%;
		border-collapse: collapse;
		font-size: 12.5px;
	}

	th,
	td {
		text-align: left;
		padding: 12px 14px;
		border-bottom: 0.5px solid var(--border-primary);
		vertical-align: top;
	}

	th {
		font-size: 10.5px;
		font-weight: 600;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		color: var(--text-tertiary);
	}

	.col-check {
		width: 38px;
	}

	.col-type,
	.col-last,
	.col-action {
		white-space: nowrap;
	}

	.item-cell-title {
		font-size: 13px;
		font-weight: 500;
		color: var(--text-primary);
	}

	.item-cell-url,
	.item-cell-error {
		margin-top: 4px;
		font-size: 11.5px;
		color: var(--text-tertiary);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		max-width: 420px;
	}

	.item-cell-error {
		color: var(--destructive);
	}

	.type-pill {
		display: inline-flex;
		border-radius: 999px;
		padding: 4px 8px;
		background: var(--bg-secondary);
		color: var(--text-secondary);
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
	}

	.row-checkbox {
		width: 18px;
		height: 18px;
		border-radius: 5px;
		border: 1px solid var(--border-secondary);
		background: transparent;
		cursor: pointer;
	}

	.row-checkbox.checked {
		background: var(--accent);
		border-color: var(--accent);
	}

	.text-action,
	.btn {
		border: none;
		cursor: pointer;
		font: inherit;
	}

	.text-action {
		background: transparent;
		color: var(--accent);
		font-size: 12.5px;
		padding: 0;
	}

	.btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 8px 14px;
		border-radius: 8px;
		font-size: 13px;
		font-weight: 500;
		color: var(--text-primary);
		background: var(--bg-secondary);
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
	}

	.btn:disabled,
	.text-action:disabled,
	.row-checkbox:disabled {
		opacity: 0.55;
		cursor: not-allowed;
	}

	.empty-row {
		text-align: center;
		color: var(--text-secondary);
		padding: 28px;
	}

	.callout {
		margin: 14px 18px;
		padding: 12px 14px;
		border-radius: 10px;
		border: 0.5px solid var(--destructive);
		background: var(--fill-danger);
	}

	.callout-title {
		font-size: 13px;
		font-weight: 600;
		color: var(--text-primary);
	}

	.callout-detail {
		font-size: 12px;
		color: var(--text-secondary);
		line-height: 1.4;
		margin-top: 3px;
	}

	.load-more-sentinel {
		height: 1px;
	}

	/* The fixed-column table's minimum width is ~800px; below that it
	   scrolls inside its card instead of widening the page. */
	@container settings-card (max-width: 799px) {
		.items-table {
			display: block;
			overflow-x: auto;
		}
	}

	@container settings-card (max-width: 539px) {
		.items-toolbar {
			flex-wrap: wrap;
		}

		.search-wrap {
			flex-basis: 100%;
		}

		.items-table th,
		.items-table td {
			padding-left: 12px;
			padding-right: 12px;
		}
	}
</style>
