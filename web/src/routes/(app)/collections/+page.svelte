<script lang="ts">
	import { onMount } from 'svelte';
	import type { CollectionResponse } from '$lib/api/generated/types.gen';
	import { getCollections } from '$lib/stores/collections.svelte';
	import { getSidebar } from '$lib/stores/sidebar.svelte';
	import { getViewport } from '$lib/stores/viewport.svelte';
	import CollectionCard from '$lib/components/collections/CollectionCard.svelte';
	import CollectionEditModal from '$lib/components/collections/CollectionEditModal.svelte';
	import { t } from '$lib/i18n';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';

	const store = getCollections();
	const sidebar = getSidebar();
	const vp = getViewport();
	let showCreateModal = $state(false);
	let editingCollection = $state<CollectionResponse | null>(null);
	let deletingCollection = $state<CollectionResponse | null>(null);

	onMount(() => {
		store.loadAllCollections();
	});

	function subCountFor(id: string): number {
		return store.allCollections.filter((c) => c.parent_id === id).length;
	}

	function handleSaved(col: CollectionResponse) {
		sidebar.refreshCollections();
		if (!editingCollection) {
			goto(resolve('/(app)/collections/[id]', { id: col.id }));
		}
	}

	async function confirmDelete() {
		if (!deletingCollection) return;
		const ok = await store.deleteCollection(deletingCollection.id);
		if (ok) {
			sidebar.refreshCollections();
		}
		deletingCollection = null;
	}
</script>

<div class="collections-page">
	<div class="page-header">
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
		<div class="header-text">
			<h1 class="page-title">{$t('collection_title')}</h1>
			{#if !store.loading && !store.isEmpty}
				<p class="page-sub">
					{$t('collection_count', { values: { count: store.rootCollections.length } })}
				</p>
			{/if}
		</div>
		<button
			type="button"
			class="btn btn-primary"
			onclick={() => {
				showCreateModal = true;
			}}
		>
			{$t('collection_new')}
		</button>
	</div>

	{#if store.loading}
		<div class="loading-state">
			<span class="loading-text">{$t('collection_loading_all')}</span>
		</div>
	{:else if store.isEmpty}
		<div class="empty-state">
			<div class="empty-icon" aria-hidden="true">
				<svg
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="1.2"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<path
						d="M3 7v10a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-6l-2-2H5a2 2 0 0 0-2 2z"
					/>
				</svg>
			</div>
			<p class="empty-heading">{$t('collection_no_collections_title')}</p>
			<p class="empty-sub">{$t('collection_no_collections_body')}</p>
			<button
				type="button"
				class="btn btn-primary"
				onclick={() => {
					showCreateModal = true;
				}}
			>
				{$t('collection_create_first')}
			</button>
		</div>
	{:else}
		<div class="collections-grid">
			{#each store.rootCollections as col (col.id)}
				<CollectionCard
					collection={col}
					subCount={subCountFor(col.id)}
					onEdit={(c) => {
						editingCollection = c;
					}}
					onDelete={(c) => {
						deletingCollection = c;
					}}
				/>
			{/each}
		</div>
	{/if}

	{#if store.fetchError}
		<div class="error-banner" role="alert">{store.fetchError}</div>
	{/if}
</div>

{#if showCreateModal}
	<CollectionEditModal
		onClose={() => {
			showCreateModal = false;
		}}
		onSaved={handleSaved}
	/>
{/if}

{#if editingCollection}
	<CollectionEditModal
		collection={editingCollection}
		onClose={() => {
			editingCollection = null;
		}}
		onSaved={handleSaved}
	/>
{/if}

{#if deletingCollection}
	<div
		class="cmd-backdrop"
		role="dialog"
		aria-modal="true"
		aria-label={$t('collection_delete_dialog')}
		tabindex="-1"
		onclick={() => {
			deletingCollection = null;
		}}
		onkeydown={(e) => {
			if (e.key === 'Escape') deletingCollection = null;
		}}
	>
		<div class="cmd-card" role="none" onclick={(e) => e.stopPropagation()} onkeydown={() => {}}>
			<div class="cmd-body cmd-body-form">
				<p class="cmd-form-desc">{$t('collection_delete_body')}</p>
			</div>
			<div class="cmd-controls">
				<button
					type="button"
					class="cmd-secondary"
					onclick={() => {
						deletingCollection = null;
					}}>{$t('common_cancel')}</button
				>
				<button type="button" class="cmd-action cmd-action-danger" onclick={confirmDelete}
					>{$t('common_delete')}</button
				>
			</div>
		</div>
	</div>
{/if}

<style>
	.collections-page {
		flex: 1;
		overflow-y: auto;
		padding: 32px 40px;
		background: var(--bg-content);
	}

	.page-header {
		display: flex;
		align-items: center;
		gap: 12px;
		margin-bottom: 28px;
	}

	.header-text {
		display: flex;
		flex-direction: column;
		gap: 2px;
		flex: 1;
		min-width: 0;
	}

	.page-title {
		font-family: var(--font-sans);
		font-size: 24px;
		font-weight: 700;
		letter-spacing: -0.03em;
		color: var(--text-primary);
		margin: 0;
	}

	.page-sub {
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--text-tertiary);
		margin: 0;
	}

	.collections-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
		gap: 16px;
	}

	.loading-state,
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 12px;
		padding: 80px 20px;
	}

	.loading-text {
		font-family: var(--font-sans);
		font-size: 14px;
		color: var(--text-secondary);
	}

	.empty-icon {
		width: 48px;
		height: 48px;
		color: var(--text-tertiary);
	}

	.empty-icon svg {
		width: 48px;
		height: 48px;
	}

	.empty-heading {
		font-family: var(--font-sans);
		font-size: 17px;
		font-weight: 600;
		color: var(--text-primary);
		margin: 0;
	}

	.empty-sub {
		font-family: var(--font-sans);
		font-size: 14px;
		color: var(--text-secondary);
		margin: 0;
	}

	.error-banner {
		margin-top: 16px;
		padding: 10px 14px;
		border-radius: 8px;
		background: var(--fill-danger);
		color: var(--destructive);
		font-size: 13px;
		font-weight: 500;
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

	.btn-primary {
		background: var(--accent);
		color: var(--text-on-color);
	}

	.btn-primary:hover {
		opacity: 0.9;
	}

	/* ---- Responsive: tablet + mobile reflow ---- */

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

	@media (max-width: 1099px) {
		.collections-page {
			padding: 24px 28px;
		}
	}

	@media (max-width: 599px) {
		.menu-btn {
			display: flex;
		}

		.collections-page {
			padding: 16px;
		}

		.page-header {
			margin-bottom: 20px;
		}

		.page-title {
			font-size: 20px;
		}

		.btn {
			height: 34px;
			padding: 0 12px;
			font-size: 13px;
		}
	}

	/* Delete confirmation modal */
	.cmd-backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.4);
		backdrop-filter: blur(4px);
		-webkit-backdrop-filter: blur(4px);
		display: flex;
		align-items: flex-start;
		justify-content: center;
		padding-top: 80px;
		z-index: 300;
		box-sizing: border-box;
	}

	:global([data-theme='dark']) .cmd-backdrop {
		background: rgba(0, 0, 0, 0.6);
	}

	.cmd-card {
		width: 460px;
		max-width: calc(100vw - 32px);
		background: var(--bg-elevated);
		border-radius: 14px;
		box-shadow:
			0 24px 80px rgba(0, 0, 0, 0.22),
			0 0 0 0.5px rgba(0, 0, 0, 0.06);
	}

	:global([data-theme='dark']) .cmd-card {
		box-shadow:
			0 24px 80px rgba(0, 0, 0, 0.55),
			0 0 0 0.5px rgba(255, 255, 255, 0.08);
	}

	.cmd-body {
		display: flex;
		flex-direction: column;
	}

	.cmd-body-form {
		padding: 16px 16px 4px;
	}

	.cmd-form-desc {
		font-family: var(--font-sans);
		font-size: 14px;
		color: var(--text-secondary);
		margin: 0;
		line-height: 1.5;
	}

	.cmd-controls {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 10px 16px 14px;
	}

	.cmd-secondary {
		padding: 6px 14px;
		border-radius: 980px;
		border: 1px solid var(--border-primary);
		background: transparent;
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 500;
		color: var(--text-secondary);
		cursor: pointer;
		transition: background 120ms ease;
		letter-spacing: -0.01em;
	}

	.cmd-secondary:hover {
		background: var(--fill-hover);
	}

	.cmd-action {
		margin-left: auto;
		padding: 6px 16px;
		border-radius: 980px;
		border: none;
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 600;
		cursor: pointer;
		letter-spacing: -0.01em;
		color: var(--text-on-color);
		background: var(--accent);
		flex-shrink: 0;
		transition: opacity 120ms ease;
	}

	.cmd-action:hover:not(:disabled) {
		opacity: 0.88;
	}

	.cmd-action-danger {
		background: var(--destructive);
	}
</style>
