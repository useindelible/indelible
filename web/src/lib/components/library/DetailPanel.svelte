<script lang="ts">
	import type { DocumentListEntry } from '$lib/api';
	import MorphSwitcher from '$lib/components/ui/MorphSwitcher.svelte';
	import DetailInfo from './DetailInfo.svelte';
	import NotebookTab from './NotebookTab.svelte';
	import ChatTab from './ChatTab.svelte';
	import EditMetadataPanel from './EditMetadataPanel.svelte';

	interface Props {
		item: DocumentListEntry | null;
		collectionId?: string | null;
		collectionName?: string | null;
	}

	let { item, collectionId = null, collectionName = null }: Props = $props();

	const displayItem = $derived(item);

	type Tab = 'info' | 'notebook' | 'chat';
	type TabOption = { value: Tab; label: string };

	let activeTab: Tab = $state('info');
	let editing = $state(false);
	const currentItemId = $derived(item?.id ?? null);
	const currentCollectionId = $derived(collectionId ?? null);
	const hasCollectionChat = $derived(Boolean(currentCollectionId && collectionName));
	let trackedItemId = $state<string | null>(null);

	$effect(() => {
		if (currentItemId !== trackedItemId) {
			trackedItemId = currentItemId;
			editing = false;
		}
	});

	$effect(() => {
		if (hasCollectionChat && activeTab === 'notebook') {
			activeTab = 'info';
		}
		if (hasCollectionChat && !displayItem && activeTab === 'info') {
			activeTab = 'chat';
		}
	});

	const itemTabOptions: TabOption[] = [
		{ value: 'info', label: 'Info' },
		{ value: 'notebook', label: 'Notebook' },
		{ value: 'chat', label: 'Chat' }
	];

	const collectionTabOptions: TabOption[] = [
		{ value: 'info', label: 'Info' },
		{ value: 'chat', label: 'Chat' }
	];

	const tabOptions = $derived(hasCollectionChat ? collectionTabOptions : itemTabOptions);

	function onTabChange(value: string) {
		activeTab = value as Tab;
	}
</script>

<aside class="detail-panel" class:chat-mode={activeTab === 'chat' && !editing}>
	<div class="detail-tabs">
		<span class="tabs-eyebrow">Details</span>
		<MorphSwitcher options={tabOptions} value={activeTab} onchange={onTabChange} size="sm" />
	</div>

	{#if editing && displayItem}
		<EditMetadataPanel
			item={displayItem}
			onClose={() => {
				editing = false;
			}}
		/>
	{:else if activeTab === 'info' && displayItem}
		<DetailInfo
			item={displayItem}
			onEditMetadata={() => {
				editing = true;
			}}
		/>
	{:else if activeTab === 'notebook' && displayItem}
		<NotebookTab item={displayItem} />
	{:else if activeTab === 'chat'}
		<div class="chat-shell">
			{#if hasCollectionChat && currentCollectionId && collectionName}
				<ChatTab
					scope={{ type: 'collection', collectionId: currentCollectionId }}
					label={collectionName}
				/>
			{:else if displayItem}
				<ChatTab
					scope={{ type: 'single_document', documentId: displayItem.id }}
					label={displayItem.title}
				/>
			{/if}
		</div>
	{/if}
</aside>

<style>
	.detail-panel {
		width: 300px;
		min-width: 300px;
		background: var(--vibrancy-sidebar);
		backdrop-filter: blur(60px) saturate(220%);
		-webkit-backdrop-filter: blur(60px) saturate(220%);
		border-left: 0.5px solid var(--border-primary);
		display: flex;
		flex-direction: column;
		overflow-y: auto;
	}

	.detail-panel.chat-mode {
		overflow: hidden;
	}

	/* Mirrors the list header grammar: quiet label left, switcher right.
	   Height is overridable so the row's hairline can align with whatever
	   header sits beside it (60px library list, 44px reader toolbar). */
	.detail-tabs {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 10px;
		height: var(--detail-tabs-height, 60px);
		padding: 0 16px;
		border-bottom: 0.5px solid var(--border-primary);
		flex-shrink: 0;
	}

	.tabs-eyebrow {
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--text-tertiary);
		white-space: nowrap;
	}

	.chat-shell {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-height: 0;
	}
</style>
