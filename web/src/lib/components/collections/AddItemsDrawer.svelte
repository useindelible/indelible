<script lang="ts">
	import { onMount } from 'svelte';
	import * as api from '$lib/api';
	import type { DocumentListEntry } from '$lib/api';
	import { SvelteSet } from 'svelte/reactivity';
	import { fetchAllPages } from '$lib/api/pagination';
	import { getCollections } from '$lib/stores/collections.svelte';

	interface Props {
		collectionId: string;
		currentItemIds: Set<string>;
		onClose: () => void;
		onSaved: () => void;
	}

	let { collectionId, currentItemIds, onClose, onSaved }: Props = $props();

	const store = getCollections();

	type ItemType = 'article' | 'video' | 'pdf' | 'epub' | 'tweet';
	const TYPE_FILTERS: { label: string; value: ItemType | null }[] = [
		{ label: 'All', value: null },
		{ label: 'Articles', value: 'article' },
		{ label: 'Books', value: 'epub' },
		{ label: 'PDFs', value: 'pdf' },
		{ label: 'Videos', value: 'video' }
	];

	let allItems = $state<DocumentListEntry[]>([]);
	let loadingItems = $state(true);
	let searchQuery = $state('');
	let activeTypeFilter = $state<ItemType | null>(null);
	let selectedIds = $state<Set<string>>(new Set(currentItemIds));
	let saving = $state(false);

	const originalIds = new Set(currentItemIds);

	const filteredItems = $derived.by(() => {
		let result = allItems;
		if (activeTypeFilter) {
			result = result.filter((item) => item.item_type === activeTypeFilter);
		}
		if (searchQuery.trim()) {
			const q = searchQuery.toLowerCase();
			result = result.filter(
				(item) =>
					item.title.toLowerCase().includes(q) ||
					(item.domain ?? '').toLowerCase().includes(q) ||
					(item.author ?? '').toLowerCase().includes(q)
			);
		}
		return result;
	});

	const pendingCount = $derived.by(() => {
		let count = 0;
		for (const id of selectedIds) {
			if (!originalIds.has(id)) count++;
		}
		for (const id of originalIds) {
			if (!selectedIds.has(id)) count++;
		}
		return count;
	});

	const addingCount = $derived.by(() => {
		let count = 0;
		for (const id of selectedIds) {
			if (!originalIds.has(id)) count++;
		}
		return count;
	});

	const hasChanges = $derived(pendingCount > 0);

	onMount(() => {
		loadAll();
	});

	async function loadAll() {
		loadingItems = true;
		try {
			allItems = await fetchAllPages(async (cursor) => {
				const resp = await api.listLibraryEntries({
					query: { cursor, limit: 100 }
				});
				if (!resp.data) return undefined;
				return {
					data: resp.data.data as DocumentListEntry[],
					page: { next_cursor: resp.data.page?.next_cursor ?? null }
				};
			});
		} finally {
			loadingItems = false;
		}
	}

	function toggleItem(id: string) {
		const next = new SvelteSet(selectedIds);
		if (next.has(id)) {
			next.delete(id);
		} else {
			next.add(id);
		}
		selectedIds = next;
	}

	async function save() {
		if (saving) return;
		saving = true;

		const toAdd = [...selectedIds].filter((id) => !originalIds.has(id));
		const toRemove = [...originalIds].filter((id) => !selectedIds.has(id));

		await Promise.all([
			...toAdd.map((id) => store.addItem(collectionId, id)),
			...toRemove.map((id) => store.removeItem(collectionId, id))
		]);

		saving = false;
		onSaved();
		onClose();
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			e.preventDefault();
			onClose();
		}
	}

	function formatType(type: string): string {
		return type.charAt(0).toUpperCase() + type.slice(1);
	}
</script>

<div
	class="drawer-overlay"
	onkeydown={handleKeydown}
	role="dialog"
	aria-modal="true"
	aria-label="Add items to collection"
>
	<div class="drawer-backdrop" onclick={onClose} role="presentation"></div>
	<div class="drawer-panel">
		<div class="drawer-header">
			<h2 class="drawer-title">Add items</h2>
			<button type="button" class="close-btn" aria-label="Close" onclick={onClose}>
				<svg
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<line x1="18" y1="6" x2="6" y2="18" />
					<line x1="6" y1="6" x2="18" y2="18" />
				</svg>
			</button>
		</div>

		<div class="drawer-search">
			<div class="search-wrap">
				<svg
					class="search-icon"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="1.8"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<circle cx="11" cy="11" r="8" />
					<line x1="21" y1="21" x2="16.65" y2="16.65" />
				</svg>
				<input
					type="search"
					class="search-input"
					placeholder="Search your library..."
					bind:value={searchQuery}
					autofocus
				/>
			</div>
		</div>

		<div class="type-filters">
			{#each TYPE_FILTERS as filter (filter.label)}
				<button
					type="button"
					class="filter-chip"
					class:active={activeTypeFilter === filter.value}
					onclick={() => {
						activeTypeFilter = filter.value;
					}}
				>
					{filter.label}
				</button>
			{/each}
		</div>

		<div class="drawer-list">
			{#if loadingItems}
				<div class="list-state">
					<span class="state-text">Loading your library...</span>
				</div>
			{:else if filteredItems.length === 0}
				<div class="list-state">
					<span class="state-text">No items found</span>
				</div>
			{:else}
				{#each filteredItems as item (item.library_entry_id ?? item.id)}
					{@const libraryEntryId = item.library_entry_id}
					{@const checked = libraryEntryId ? selectedIds.has(libraryEntryId) : false}
					{@const alreadyIn = libraryEntryId ? originalIds.has(libraryEntryId) : false}
					<button
						type="button"
						class="item-row"
						class:checked
						disabled={!libraryEntryId}
						onclick={() => {
							if (libraryEntryId) toggleItem(libraryEntryId);
						}}
					>
						<span
							class="item-check"
							class:checked
							class:already={alreadyIn && checked}
							aria-hidden="true"
						>
							{#if checked}
								<svg
									viewBox="0 0 14 14"
									fill="none"
									stroke="currentColor"
									stroke-width="2.2"
									stroke-linecap="round"
									stroke-linejoin="round"
								>
									<polyline points="2,7 5.5,10.5 12,3.5" />
								</svg>
							{/if}
						</span>
						{#if item.thumbnail_url || item.lead_image_url}
							<img class="item-thumb" src={item.thumbnail_url ?? item.lead_image_url} alt="" />
						{:else}
							<span class="item-thumb-placeholder">
								<svg
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="1.5"
									stroke-linecap="round"
									stroke-linejoin="round"
								>
									<path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" />
									<polyline points="14 2 14 8 20 8" />
								</svg>
							</span>
						{/if}
						<span class="item-info">
							<span class="item-title">{item.title}</span>
							<span class="item-meta">
								{#if item.domain}
									<span class="meta-domain">{item.domain}</span>
								{/if}
								{#if item.item_type && item.item_type !== 'article'}
									<span class="meta-type">{formatType(item.item_type)}</span>
								{/if}
							</span>
						</span>
					</button>
				{/each}
			{/if}
		</div>

		<div class="drawer-footer">
			<span class="footer-count">
				{#if addingCount > 0}
					<strong>{addingCount}</strong> item{addingCount !== 1 ? 's' : ''} selected
				{:else}
					No new items selected
				{/if}
			</span>
			<button type="button" class="btn btn-primary" disabled={!hasChanges || saving} onclick={save}>
				{saving ? 'Saving…' : 'Add to collection'}
			</button>
		</div>
	</div>
</div>

<style>
	.drawer-overlay {
		position: fixed;
		inset: 0;
		display: flex;
		align-items: stretch;
		justify-content: flex-end;
		z-index: 200;
	}

	.drawer-backdrop {
		position: absolute;
		inset: 0;
		background: rgba(0, 0, 0, 0.35);
		backdrop-filter: blur(2px);
	}

	.drawer-panel {
		position: relative;
		width: 460px;
		max-width: 90vw;
		background: var(--bg-primary);
		border-left: 1px solid var(--border-primary);
		display: flex;
		flex-direction: column;
		height: 100%;
		box-shadow: -8px 0 40px rgba(0, 0, 0, 0.12);
	}

	.drawer-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 20px 20px 0;
		flex-shrink: 0;
	}

	.drawer-title {
		font-family: var(--font-sans);
		font-size: 17px;
		font-weight: 700;
		letter-spacing: -0.02em;
		color: var(--text-primary);
		margin: 0;
	}

	.close-btn {
		width: 28px;
		height: 28px;
		border-radius: 8px;
		border: none;
		background: transparent;
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: pointer;
		color: var(--text-tertiary);
		padding: 0;
		transition: background 0.12s ease;
	}

	.close-btn:hover {
		background: var(--fill-hover);
		color: var(--text-primary);
	}

	.close-btn svg {
		width: 16px;
		height: 16px;
	}

	.drawer-search {
		padding: 16px 20px 0;
		flex-shrink: 0;
	}

	.search-wrap {
		position: relative;
		display: flex;
		align-items: center;
	}

	.search-icon {
		position: absolute;
		left: 10px;
		width: 16px;
		height: 16px;
		color: var(--text-tertiary);
		pointer-events: none;
	}

	.search-input {
		width: 100%;
		height: 36px;
		border-radius: 10px;
		border: 1px solid var(--border-primary);
		background: var(--bg-secondary);
		padding: 0 12px 0 34px;
		font-family: var(--font-sans);
		font-size: 14px;
		color: var(--text-primary);
		outline: none;
		transition: border-color 0.15s ease;
	}

	.search-input:focus {
		border-color: var(--accent);
	}

	.search-input::placeholder {
		color: var(--text-quaternary);
	}

	.type-filters {
		display: flex;
		gap: 6px;
		padding: 12px 20px 0;
		flex-shrink: 0;
	}

	.filter-chip {
		height: 26px;
		padding: 0 10px;
		border-radius: 20px;
		border: 1px solid var(--border-primary);
		background: transparent;
		font-family: var(--font-sans);
		font-size: 12px;
		font-weight: 500;
		color: var(--text-secondary);
		cursor: pointer;
		transition:
			background 0.12s ease,
			color 0.12s ease,
			border-color 0.12s ease;
	}

	.filter-chip:hover {
		background: var(--fill-hover);
		color: var(--text-primary);
	}

	.filter-chip.active {
		background: var(--accent);
		border-color: var(--accent);
		color: #fff;
	}

	.drawer-list {
		flex: 1;
		overflow-y: auto;
		padding: 8px 0;
		margin-top: 8px;
	}

	.list-state {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 48px 20px;
	}

	.state-text {
		font-family: var(--font-sans);
		font-size: 14px;
		color: var(--text-tertiary);
	}

	.item-row {
		width: 100%;
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 8px 20px;
		background: transparent;
		border: none;
		cursor: pointer;
		text-align: left;
		transition: background 0.1s ease;
	}

	.item-row:hover {
		background: var(--fill-hover);
	}

	.item-row.checked {
		background: var(--fill-selected);
	}

	.item-check {
		width: 18px;
		height: 18px;
		border-radius: 5px;
		border: 1.5px solid var(--border-primary);
		background: transparent;
		flex-shrink: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		transition:
			background 0.12s ease,
			border-color 0.12s ease;
	}

	.item-check.checked {
		background: var(--accent);
		border-color: var(--accent);
	}

	.item-check.already {
		background: var(--fill-secondary);
		border-color: var(--border-secondary);
	}

	.item-check svg {
		width: 11px;
		height: 11px;
		color: #fff;
	}

	.item-thumb {
		width: 40px;
		height: 40px;
		border-radius: 6px;
		object-fit: cover;
		flex-shrink: 0;
		background: var(--bg-secondary);
	}

	.item-thumb-placeholder {
		width: 40px;
		height: 40px;
		border-radius: 6px;
		background: var(--bg-secondary);
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.item-thumb-placeholder svg {
		width: 18px;
		height: 18px;
		color: var(--text-quaternary);
	}

	.item-info {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
		flex: 1;
	}

	.item-title {
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 500;
		color: var(--text-primary);
		letter-spacing: -0.01em;
		line-height: 1.3;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.item-meta {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.meta-domain {
		font-family: var(--font-sans);
		font-size: 11px;
		color: var(--text-tertiary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.meta-type {
		font-family: var(--font-sans);
		font-size: 11px;
		color: var(--text-quaternary);
		background: var(--bg-secondary);
		border-radius: 4px;
		padding: 0 5px;
		flex-shrink: 0;
	}

	.drawer-footer {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding: 16px 24px;
		border-top: 1px solid var(--border-primary);
		flex-shrink: 0;
	}

	.footer-count {
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--text-secondary);
		flex-shrink: 0;
	}

	.footer-count strong {
		color: var(--accent);
		font-weight: 600;
	}

	.btn {
		height: 36px;
		padding: 0 16px;
		border-radius: 8px;
		font-family: var(--font-sans);
		font-size: 14px;
		font-weight: 500;
		letter-spacing: -0.01em;
		cursor: pointer;
		border: none;
		transition:
			background 0.12s ease,
			opacity 0.12s ease;
	}

	.btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.btn-primary {
		background: var(--accent);
		color: #fff;
	}

	.btn-primary:hover:not(:disabled) {
		opacity: 0.9;
	}

	.btn-secondary {
		background: var(--fill-secondary);
		color: var(--text-primary);
	}

	.btn-secondary:hover {
		background: var(--fill-hover);
	}
</style>
