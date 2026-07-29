<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import type { DocumentListEntry } from '$lib/api';
	import { getCollections } from '$lib/stores/collections.svelte';
	import { getLibrary } from '$lib/stores/library.svelte';
	import { getSidebar } from '$lib/stores/sidebar.svelte';
	import { getViewport } from '$lib/stores/viewport.svelte';
	import { buildBreadcrumbPath } from '$lib/api/pagination';
	import CollectionBreadcrumbs from '$lib/components/collections/CollectionBreadcrumbs.svelte';
	import CollectionEditModal from '$lib/components/collections/CollectionEditModal.svelte';
	import AddItemsDrawer from '$lib/components/collections/AddItemsDrawer.svelte';
	import DetailPanel from '$lib/components/library/DetailPanel.svelte';
	import ItemList from '$lib/components/library/ItemList.svelte';

	const store = getCollections();
	const lib = getLibrary();
	const sidebar = getSidebar();
	const vp = getViewport();

	let showEditModal = $state(false);
	let showCreateChildModal = $state(false);
	let showDeleteConfirm = $state(false);
	let showAddItems = $state(false);
	let selectedId = $state<string | null>(null);

	// Below the desktop breakpoint the docked detail panel becomes a slide-over
	// (tablet) or full-screen view (mobile); session-only, matching the library list.
	let compactDetailOpen = $state(false);

	function openItemDetail(id: string) {
		selectedId = id;
		compactDetailOpen = true;
	}

	const selectedItem = $derived<DocumentListEntry | null>(
		store.items.find((i) => i.id === selectedId) ??
			(store.items.length > 0 ? store.items[0] : null) ??
			null
	);

	const collectionId = $derived(page.params.id ?? '');
	const breadcrumbs = $derived(
		store.currentCollection
			? buildBreadcrumbPath(store.currentCollection.id, sidebar.allCollections)
			: []
	);
	const colorIndex = $derived(
		store.currentCollection
			? store.currentCollection.id.split('').reduce((acc, c) => acc + c.charCodeAt(0), 0) % 6
			: 0
	);

	$effect(() => {
		const id = collectionId;
		if (id) {
			store.loadCollection(id);
			store.loadChildren(id);
			store.loadItems(id, true);
			selectedId = null;
		}
	});

	function handleLoadMore() {
		if (collectionId) {
			store.loadItems(collectionId);
		}
	}

	function handleSaved() {
		sidebar.refreshCollections();
	}

	async function confirmDelete() {
		if (!store.currentCollection) return;
		const ok = await store.deleteCollection(store.currentCollection.id);
		if (ok) {
			sidebar.refreshCollections();
			goto(resolve('/(app)/collections'));
		}
		showDeleteConfirm = false;
	}
</script>

<div class="detail-layout">
	{#if store.loading && !store.currentCollection}
		<div class="loading-state">
			<span class="loading-text">Loading collection...</span>
		</div>
	{:else if store.currentCollection}
		{@const col = store.currentCollection}

		<div class="detail-main">
			<div class="collection-hero">
				<div class="hero-nav-row">
					<button
						type="button"
						class="menu-btn"
						onclick={() => vp.openMobileNav()}
						aria-label="Open navigation"
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
					<CollectionBreadcrumbs path={breadcrumbs} />
				</div>

				<div class="hero-top">
					<div
						class="hero-badge"
						style:background={`var(--collection-gradient-${colorIndex})`}
						aria-hidden="true"
					>
						{col.icon || '📁'}
					</div>
					<div class="hero-text">
						<h1 class="hero-title">{col.name}</h1>
						{#if col.description}
							<p class="hero-desc">{col.description}</p>
						{/if}
						<div class="hero-stats">
							<span class="stat"
								>{store.items.length} item{store.items.length !== 1 ? 's' : ''}</span
							>
							{#if store.children.length > 0}
								<span class="stat-sep">·</span>
								<span class="stat"
									>{store.children.length} sub-collection{store.children.length !== 1
										? 's'
										: ''}</span
								>
							{/if}
						</div>
					</div>
					<div class="hero-actions">
						<button
							type="button"
							class="btn btn-sm btn-secondary"
							onclick={() => {
								showAddItems = true;
							}}
						>
							Add items
						</button>
						<button
							type="button"
							class="action-btn"
							aria-label="Edit collection"
							onclick={() => {
								showEditModal = true;
							}}
						>
							<svg
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="1.6"
								stroke-linecap="round"
								stroke-linejoin="round"
							>
								<path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7" />
								<path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z" />
							</svg>
						</button>
						<button
							type="button"
							class="action-btn action-btn-danger"
							aria-label="Delete collection"
							onclick={() => {
								showDeleteConfirm = true;
							}}
						>
							<svg
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="1.6"
								stroke-linecap="round"
								stroke-linejoin="round"
							>
								<path d="M3 6h18" />
								<path d="M16 6V4a2 2 0 00-2-2h-4a2 2 0 00-2 2v2" />
								<path d="M19 6l-1 14a2 2 0 01-2 2H8a2 2 0 01-2-2L5 6" />
							</svg>
						</button>
						{#if vp.isCompact && !vp.isMobile}
							<button
								type="button"
								class="action-btn"
								class:panel-active={compactDetailOpen}
								onclick={() => (compactDetailOpen = !compactDetailOpen)}
								aria-label={compactDetailOpen ? 'Hide detail panel' : 'Show detail panel'}
								title={compactDetailOpen ? 'Hide detail panel' : 'Show detail panel'}
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
				</div>
			</div>

			{#if store.children.length > 0}
				<div class="sub-strip">
					{#each store.children as child (child.id)}
						<a href={resolve('/(app)/collections/[id]', { id: child.id })} class="sub-chip">
							<span class="chip-icon">{child.icon || '📁'}</span>
							<span class="chip-name">{child.name}</span>
							<span class="chip-count">{child.item_count}</span>
							<svg
								class="chip-chevron"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
								stroke-linecap="round"
								stroke-linejoin="round"
								aria-hidden="true"
							>
								<polyline points="9 18 15 12 9 6" />
							</svg>
						</a>
					{/each}
					<button
						type="button"
						class="sub-chip sub-chip-add"
						onclick={() => {
							showCreateChildModal = true;
						}}
					>
						<span class="chip-icon" aria-hidden="true">+</span>
						<span class="chip-name">New sub-collection</span>
					</button>
				</div>
			{/if}

			{#if store.itemsEmpty && !store.itemsLoading}
				<div class="empty-state">
					<div class="empty-icon" aria-hidden="true">
						<svg
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="1.5"
							stroke-linecap="round"
							stroke-linejoin="round"
						>
							<path d="M19 21l-7-5-7 5V5a2 2 0 012-2h10a2 2 0 012 2z" />
						</svg>
					</div>
					<p class="empty-title">No items yet</p>
					<p class="empty-desc">Add items from your library to this collection.</p>
					<button
						type="button"
						class="btn btn-primary"
						onclick={() => {
							showAddItems = true;
						}}
					>
						Add items
					</button>
				</div>
			{:else}
				<ItemList
					items={store.items}
					loading={store.itemsLoading}
					loadingMore={store.itemsLoadingMore}
					hasMore={store.itemsHasMore}
					isEmpty={store.itemsEmpty}
					selectedId={selectedItem?.id ?? null}
					triageTab="inbox"
					triageMode={lib.triageMode}
					onLoadMore={handleLoadMore}
					onSelect={(id) => {
						selectedId = id;
					}}
					onOpen={(id) => {
						goto(resolve('/(app)/reader/[documentId]', { documentId: id }));
					}}
					onTriage={() => {}}
					onDetail={vp.isMobile ? openItemDetail : undefined}
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
								aria-label="Back to collection"
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
							<span class="m-dtitle">{selectedItem?.title ?? 'Details'}</span>
						</div>
						<DetailPanel item={selectedItem} collectionId={col.id} collectionName={col.name} />
					</div>
				{:else}
					<!-- svelte-ignore a11y_click_events_have_key_events -->
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div class="detail-scrim" onclick={() => (compactDetailOpen = false)}></div>
					<div class="detail-overlay">
						<DetailPanel item={selectedItem} collectionId={col.id} collectionName={col.name} />
					</div>
				{/if}
			{/if}
		{:else}
			<DetailPanel item={selectedItem} collectionId={col.id} collectionName={col.name} />
		{/if}
	{:else if store.fetchError}
		<div class="error-state">
			<p class="error-text">{store.fetchError}</p>
			<a href={resolve('/(app)/collections')} class="btn btn-secondary">Back to collections</a>
		</div>
	{/if}
</div>

{#if showAddItems && store.currentCollection}
	<AddItemsDrawer
		collectionId={store.currentCollection.id}
		currentItemIds={new Set(
			store.items.flatMap((i) => (i.library_entry_id ? [i.library_entry_id] : []))
		)}
		onClose={() => {
			showAddItems = false;
		}}
		onSaved={() => {
			store.loadItems(collectionId, true);
		}}
	/>
{/if}

{#if showEditModal && store.currentCollection}
	<CollectionEditModal
		collection={store.currentCollection}
		allCollections={sidebar.allCollections}
		onClose={() => {
			showEditModal = false;
		}}
		onSaved={handleSaved}
	/>
{/if}

{#if showCreateChildModal}
	<CollectionEditModal
		parentId={collectionId}
		allCollections={sidebar.allCollections}
		onClose={() => {
			showCreateChildModal = false;
		}}
		onSaved={() => {
			handleSaved();
			store.loadChildren(collectionId);
		}}
	/>
{/if}

{#if showDeleteConfirm}
	<div class="modal-overlay" role="dialog" aria-modal="true" aria-label="Delete collection">
		<div
			class="modal-backdrop"
			onclick={() => {
				showDeleteConfirm = false;
			}}
			role="presentation"
		></div>
		<div class="modal-panel">
			<h2 class="modal-title">Delete "{store.currentCollection?.name}"?</h2>
			<p class="modal-desc">
				This collection will be permanently deleted. Items inside will not be deleted.
			</p>
			<div class="modal-actions">
				<button
					type="button"
					class="btn btn-secondary"
					onclick={() => {
						showDeleteConfirm = false;
					}}>Cancel</button
				>
				<button type="button" class="btn btn-danger" onclick={confirmDelete}>Delete</button>
			</div>
		</div>
	</div>
{/if}

<style>
	/* Layout */
	.detail-layout {
		display: flex;
		flex: 1;
		overflow: hidden;
		position: relative;
	}

	.detail-main {
		flex: 1;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		background: var(--bg-content);
	}

	/* Hero */
	.collection-hero {
		padding: 24px 32px 20px;
		display: flex;
		flex-direction: column;
		gap: 14px;
		border-bottom: 1px solid var(--border-primary);
	}

	.hero-top {
		display: flex;
		align-items: flex-start;
		gap: 14px;
	}

	.hero-badge {
		width: 52px;
		height: 52px;
		border-radius: var(--radius-lg);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 24px;
		line-height: 1;
		flex-shrink: 0;
		filter: drop-shadow(0 2px 6px rgba(0, 0, 0, 0.15));
	}

	.hero-text {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.hero-title {
		font-family: var(--font-sans);
		font-size: 20px;
		font-weight: 700;
		letter-spacing: -0.02em;
		color: var(--text-primary);
		margin: 0;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.hero-desc {
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--text-secondary);
		margin: 0;
		line-height: 1.5;
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}

	.hero-stats {
		display: flex;
		align-items: center;
		gap: 4px;
		margin-top: 2px;
	}

	.stat {
		font-family: var(--font-sans);
		font-size: 12px;
		color: var(--text-tertiary);
	}

	.stat-sep {
		font-size: 12px;
		color: var(--text-quaternary);
	}

	.hero-actions {
		display: flex;
		align-items: center;
		gap: 6px;
		flex-shrink: 0;
	}

	/* Sub-collections strip */
	.sub-strip {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 12px 32px;
		overflow-x: auto;
		border-bottom: 1px solid var(--border-primary);
		scrollbar-width: none;
	}

	.sub-strip::-webkit-scrollbar {
		display: none;
	}

	.sub-chip {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 10px;
		border-radius: var(--radius-lg);
		background: var(--bg-secondary);
		border: 1px solid var(--border-primary);
		text-decoration: none;
		color: var(--text-primary);
		white-space: nowrap;
		flex-shrink: 0;
		font-family: var(--font-sans);
		font-size: 12px;
		font-weight: 500;
		transition: background 0.12s ease;
		cursor: pointer;
	}

	.sub-chip:hover {
		background: var(--fill-hover);
	}

	.chip-icon {
		font-size: 14px;
		line-height: 1;
	}

	.chip-name {
		color: var(--text-primary);
	}

	.chip-count {
		font-size: 11px;
		color: var(--text-tertiary);
		font-weight: 400;
	}

	.chip-chevron {
		width: 12px;
		height: 12px;
		color: var(--text-quaternary);
	}

	.sub-chip-add {
		border: 1px dashed var(--border-secondary);
		background: transparent;
		color: var(--text-tertiary);
	}

	.sub-chip-add:hover {
		border-color: var(--border-secondary);
		background: var(--fill-hover);
		color: var(--text-secondary);
	}

	/* Loading / error states */
	.loading-state,
	.error-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 12px;
		padding: 80px 20px;
		flex: 1;
	}

	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 10px;
		padding: 80px 20px;
		flex: 1;
	}

	.empty-icon {
		width: 48px;
		height: 48px;
		border-radius: 14px;
		background: var(--bg-secondary);
		display: flex;
		align-items: center;
		justify-content: center;
		margin-bottom: 4px;
	}

	.empty-icon svg {
		width: 24px;
		height: 24px;
		color: var(--text-tertiary);
	}

	.empty-title {
		font-family: var(--font-sans);
		font-size: 15px;
		font-weight: 600;
		color: var(--text-primary);
		letter-spacing: -0.01em;
		margin: 0;
	}

	.empty-desc {
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--text-tertiary);
		margin: 0 0 6px;
		text-align: center;
		max-width: 260px;
		line-height: 1.5;
	}

	.loading-text,
	.error-text {
		font-family: var(--font-sans);
		font-size: 14px;
		color: var(--text-secondary);
		margin: 0;
	}

	/* Buttons */
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
		text-decoration: none;
		display: inline-flex;
		align-items: center;
		transition:
			background 0.12s ease,
			opacity 0.12s ease;
	}

	.btn-sm {
		height: 30px;
		padding: 0 12px;
		font-size: 13px;
	}

	.btn-primary {
		background: var(--accent);
		color: var(--text-on-color);
	}

	.btn-primary:hover {
		opacity: 0.9;
	}

	.btn-secondary {
		background: var(--fill-secondary);
		color: var(--text-primary);
	}

	.btn-secondary:hover {
		background: var(--fill-hover);
	}

	.btn-danger {
		background: var(--destructive);
		color: var(--text-on-color);
	}

	.btn-danger:hover {
		opacity: 0.9;
	}

	.action-btn {
		width: 32px;
		height: 32px;
		border-radius: 8px;
		border: none;
		background: transparent;
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: pointer;
		color: var(--text-tertiary);
		padding: 0;
		transition:
			background 0.12s ease,
			color 0.12s ease;
	}

	.action-btn:hover {
		background: var(--fill-hover);
		color: var(--text-primary);
	}

	.action-btn-danger:hover {
		color: var(--destructive);
	}

	.action-btn svg {
		width: 18px;
		height: 18px;
	}

	/* Delete confirmation modal */
	.modal-overlay {
		position: fixed;
		inset: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 100;
	}

	.modal-backdrop {
		position: absolute;
		inset: 0;
		background: rgba(0, 0, 0, 0.4);
		backdrop-filter: blur(4px);
	}

	.modal-panel {
		position: relative;
		background: var(--bg-primary);
		border: 1px solid var(--border-primary);
		border-radius: 16px;
		padding: 24px;
		width: 380px;
		max-width: 90vw;
		box-shadow: 0 20px 60px rgba(0, 0, 0, 0.15);
	}

	.modal-title {
		font-family: var(--font-sans);
		font-size: 17px;
		font-weight: 700;
		letter-spacing: -0.02em;
		color: var(--text-primary);
		margin: 0 0 8px;
	}

	.modal-desc {
		font-family: var(--font-sans);
		font-size: 14px;
		color: var(--text-secondary);
		margin: 0 0 20px;
		line-height: 1.5;
	}

	.modal-actions {
		display: flex;
		justify-content: flex-end;
		gap: 10px;
	}

	/* ---- Responsive: tablet slide-over + mobile reflow ---- */

	.hero-nav-row {
		display: flex;
		align-items: center;
		gap: 8px;
		min-width: 0;
	}

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
	}

	.menu-btn:hover {
		background: var(--fill-hover);
	}

	.menu-btn svg {
		width: 20px;
		height: 20px;
	}

	.action-btn.panel-active {
		color: var(--accent);
		background: var(--fill-selected);
	}

	.detail-scrim {
		position: absolute;
		inset: 0;
		background: rgba(0, 0, 0, 0.1);
		z-index: 20;
	}

	/* Opaque surface: the docked panel's vibrancy blur would let the list rows
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

	@media (max-width: 1099px) {
		.collection-hero {
			padding: 20px 24px 16px;
		}

		.sub-strip {
			padding: 12px 24px;
		}
	}

	@media (max-width: 599px) {
		.menu-btn {
			display: flex;
		}

		.collection-hero {
			padding: 12px 16px 14px;
		}

		.sub-strip {
			padding: 12px 16px;
		}

		/* Actions drop below the badge + text block instead of squeezing the title. */
		.hero-top {
			flex-wrap: wrap;
		}

		.hero-actions {
			width: 100%;
			justify-content: flex-end;
		}
	}
</style>
