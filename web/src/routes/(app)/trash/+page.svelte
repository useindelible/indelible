<script lang="ts">
	import * as apiSdk from '$lib/api';
	import type { DocumentListEntry } from '$lib/api';
	import { fetchAllPages } from '$lib/api/pagination';
	import { t } from '$lib/i18n';
	import { getSidebar } from '$lib/stores/sidebar.svelte';
	import { getViewport } from '$lib/stores/viewport.svelte';
	import TrashItemRow from '$lib/components/trash/TrashItemRow.svelte';
	import EmptyTrashDialog from '$lib/components/trash/EmptyTrashDialog.svelte';
	import DeleteItemDialog from '$lib/components/trash/DeleteItemDialog.svelte';

	type TrashItem = DocumentListEntry;

	const sidebar = getSidebar();
	const vp = getViewport();

	let items = $state<TrashItem[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let restoringIds = $state(new Set<string>());
	let deletingId = $state<string | null>(null);
	let showEmptyDialog = $state(false);
	let itemToDelete = $state<TrashItem | null>(null);
	let emptying = $state(false);

	async function load() {
		loading = true;
		error = null;
		try {
			items = await fetchAllPages((cursor) =>
				apiSdk.listTrash({ query: { cursor, limit: 50 } }).then((r) => {
					if (!r.data) return undefined;
					return {
						data: r.data.data as TrashItem[],
						page: { next_cursor: r.data.page.next_cursor ?? null }
					};
				})
			);
		} catch (e) {
			error = e instanceof Error ? e.message : $t('trash_error_load');
		} finally {
			loading = false;
		}
	}

	async function handleRestore(id: string) {
		restoringIds = new Set([...restoringIds, id]);
		try {
			await apiSdk.restoreLibraryEntry({ path: { document_id: id } });
			items = items.filter((i) => i.id !== id);
			await sidebar.refreshTrashCount();
		} catch {
			// Item stays in list on failure
		} finally {
			restoringIds = new Set([...restoringIds].filter((x) => x !== id));
		}
	}

	async function handlePermanentDelete(id: string) {
		deletingId = id;
		try {
			await apiSdk.purgeLibraryEntry({ path: { document_id: id } });
			items = items.filter((i) => i.id !== id);
			itemToDelete = null;
			await sidebar.refreshTrashCount();
		} catch {
			// Item stays in list on failure
		} finally {
			deletingId = null;
		}
	}

	async function handleEmptyTrash() {
		emptying = true;
		try {
			await apiSdk.emptyTrash();
			items = [];
			showEmptyDialog = false;
			await sidebar.refreshTrashCount();
		} catch (e) {
			error = e instanceof Error ? e.message : $t('trash_error_empty');
		} finally {
			emptying = false;
		}
	}

	$effect(() => {
		void load();
	});
</script>

<div class="trash-page">
	<div class="trash-banner">
		<svg viewBox="0 0 24 24" aria-hidden="true">
			<path
				d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"
			/>
			<line x1="12" y1="9" x2="12" y2="13" />
			<line x1="12" y1="17" x2="12.01" y2="17" />
		</svg>
		<span>{$t('trash_retention_notice')}</span>
	</div>

	<div class="trash-header">
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
		<div class="trash-title">
			<svg viewBox="0 0 24 24" aria-hidden="true">
				<polyline points="3 6 5 6 21 6" />
				<path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
			</svg>
			{$t('trash_title')}
		</div>
		{#if items.length > 0}
			<button
				class="empty-trash-btn"
				onclick={() => {
					showEmptyDialog = true;
				}}
				disabled={emptying}
			>
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<polyline points="3 6 5 6 21 6" />
					<path
						d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"
					/>
					<line x1="10" y1="11" x2="10" y2="17" />
					<line x1="14" y1="11" x2="14" y2="17" />
				</svg>
				{$t('trash_empty')}
			</button>
		{/if}
	</div>

	{#if error}
		<p class="error-msg">{error}</p>
	{:else if loading}
		<div class="loading-state">
			<span class="spinner" aria-hidden="true"></span>
			<span>{$t('common_loading')}</span>
		</div>
	{:else if items.length === 0}
		<div class="empty-state">
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.5"
				stroke-linecap="round"
				stroke-linejoin="round"
				aria-hidden="true"
			>
				<polyline points="3 6 5 6 21 6" />
				<path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
			</svg>
			<p>{$t('trash_empty_state')}</p>
		</div>
	{:else}
		<div class="trash-list">
			{#each items as item (item.id)}
				<TrashItemRow
					{item}
					onRestore={handleRestore}
					onDeleteClick={(i) => {
						itemToDelete = i;
					}}
					restoring={restoringIds.has(item.id)}
				/>
			{/each}
		</div>
		<div class="item-count">{$t('trash_item_count', { values: { count: items.length } })}</div>
	{/if}
</div>

{#if showEmptyDialog}
	<EmptyTrashDialog
		itemCount={items.length}
		onConfirm={handleEmptyTrash}
		onClose={() => {
			showEmptyDialog = false;
		}}
		confirming={emptying}
	/>
{/if}

<DeleteItemDialog
	item={itemToDelete}
	onConfirm={handlePermanentDelete}
	onClose={() => {
		itemToDelete = null;
	}}
	deleting={deletingId !== null}
/>

<style>
	.trash-page {
		flex: 1;
		display: flex;
		flex-direction: column;
		overflow-y: auto;
		position: relative;
	}

	.trash-banner {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 12px 20px;
		background: var(--bg-tertiary);
		border-bottom: 0.5px solid var(--border-primary);
	}
	.trash-banner svg {
		width: 18px;
		height: 18px;
		stroke: var(--warning);
		fill: none;
		stroke-width: 1.5;
		stroke-linecap: round;
		stroke-linejoin: round;
		flex-shrink: 0;
	}
	.trash-banner span {
		font-size: 15px;
		font-weight: 400;
		letter-spacing: -0.01em;
		line-height: 1.5;
		color: var(--text-primary);
	}

	.trash-header {
		padding: 16px 20px 12px;
		border-bottom: 0.5px solid var(--border-primary);
		display: flex;
		align-items: center;
		gap: 10px;
	}
	.trash-title {
		display: flex;
		align-items: center;
		gap: 10px;
		flex: 1;
		min-width: 0;
		font-size: 28px;
		font-weight: 700;
		letter-spacing: -0.03em;
		line-height: 1.18;
		color: var(--text-primary);
	}
	.trash-title svg {
		width: 24px;
		height: 24px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.5;
		stroke-linecap: round;
		stroke-linejoin: round;
	}
	.empty-trash-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		font-family: inherit;
		font-weight: 500;
		letter-spacing: -0.01em;
		cursor: pointer;
		white-space: nowrap;
		border: none;
		transition: opacity 120ms ease;
		padding: 5px 12px;
		font-size: 12px;
		border-radius: 6px;
		min-height: 28px;
		background: var(--destructive);
		color: var(--text-on-color);
	}
	.empty-trash-btn:hover:not(:disabled) {
		opacity: 0.88;
	}
	.empty-trash-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}
	.empty-trash-btn svg {
		width: 14px;
		height: 14px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.5;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.error-msg {
		color: var(--destructive);
		font-size: 14px;
		padding: 20px;
	}

	.loading-state {
		display: flex;
		align-items: center;
		gap: 10px;
		color: var(--text-tertiary);
		font-size: 14px;
		padding: 40px 20px;
	}

	.spinner {
		display: inline-block;
		width: 16px;
		height: 16px;
		border: 2px solid var(--border-primary);
		border-top-color: var(--accent);
		border-radius: 50%;
		animation: spin 0.7s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 12px;
		padding: 64px 0;
		color: var(--text-tertiary);
	}

	.empty-state svg {
		width: 32px;
		height: 32px;
	}

	.empty-state p {
		font-size: 14px;
		font-weight: 500;
		margin: 0;
	}

	.trash-list {
		flex: 1;
		overflow-y: auto;
		position: relative;
	}

	.item-count {
		position: absolute;
		bottom: 16px;
		right: 20px;
		font-size: 12px;
		font-weight: 500;
		color: var(--text-tertiary);
	}

	/* ---- Responsive: mobile reflow ---- */

	.menu-btn {
		display: none;
		width: 34px;
		height: 34px;
		align-items: center;
		justify-content: center;
		border-radius: var(--radius-sm);
		border: none;
		background: transparent;
		color: var(--text-secondary);
		cursor: pointer;
		flex-shrink: 0;
		padding: 0;
	}

	.menu-btn:hover {
		background: var(--fill-hover);
	}

	.menu-btn svg {
		width: 20px;
		height: 20px;
	}

	@media (max-width: 599px) {
		.menu-btn {
			display: flex;
		}

		.trash-banner {
			padding: 10px 16px;
		}

		.trash-banner span {
			font-size: 13px;
		}

		.trash-header {
			padding: 12px 16px 10px;
		}

		.trash-title {
			font-size: 22px;
		}

		.trash-title svg {
			width: 20px;
			height: 20px;
		}

		.item-count {
			right: 16px;
		}
	}
</style>
