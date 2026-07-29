<script lang="ts">
	import type { DocumentListEntry } from '$lib/api';
	import type { BookMetadata } from './book-source';
	import MorphSwitcher from '$lib/components/ui/MorphSwitcher.svelte';
	import BookInfoPanel from './BookInfoPanel.svelte';
	import NotebookTab from '$lib/components/library/NotebookTab.svelte';
	import ChatTab from '$lib/components/library/ChatTab.svelte';

	export type DetailTab = 'info' | 'notebook' | 'chat';

	interface Props {
		item: DocumentListEntry;
		bookMetadata: BookMetadata;
		progress: number;
	}

	let { item, bookMetadata, progress }: Props = $props();

	let activeTab = $state<DetailTab>('info');

	const tabOptions = [
		{ value: 'info', label: 'Info' },
		{ value: 'notebook', label: 'Notebook' },
		{ value: 'chat', label: 'Chat' }
	];

	function onTabChange(value: string) {
		activeTab = value as DetailTab;
	}
</script>

<div class="right-panel" class:chat-mode={activeTab === 'chat'}>
	<div class="right-header">
		<span class="tabs-eyebrow">Details</span>
		<MorphSwitcher options={tabOptions} value={activeTab} onchange={onTabChange} size="sm" />
	</div>
	{#if activeTab === 'info'}
		<div class="right-body">
			<BookInfoPanel {item} {bookMetadata} {progress} />
		</div>
	{:else if activeTab === 'notebook'}
		<NotebookTab {item} />
	{:else if activeTab === 'chat'}
		<ChatTab scope={{ type: 'single_document', documentId: item.id }} label={item.title} />
	{/if}
</div>

<style>
	.right-panel {
		width: 320px;
		min-width: 320px;
		display: flex;
		flex-direction: column;
		background: var(--detail-bg);
		backdrop-filter: blur(40px) saturate(180%);
		-webkit-backdrop-filter: blur(40px) saturate(180%);
		border-left: 0.5px solid var(--border-primary);
		position: relative;
		z-index: 2;
		overflow-y: auto;
	}

	.right-panel.chat-mode {
		overflow: hidden;
	}

	/* Mirrors the list header grammar: quiet label left, switcher right.
	   44px matches the reader toolbar so the two hairlines connect. */
	.right-header {
		flex-shrink: 0;
		height: 44px;
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 10px;
		padding: 0 16px;
		border-bottom: 0.5px solid var(--border-primary);
	}

	.tabs-eyebrow {
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--text-tertiary);
		white-space: nowrap;
	}

	.right-body {
		flex: 1;
		overflow-y: auto;
		overflow-x: hidden;
		padding: 16px;
	}

	.right-body::-webkit-scrollbar {
		width: 4px;
	}

	.right-body::-webkit-scrollbar-track {
		background: transparent;
	}

	.right-body::-webkit-scrollbar-thumb {
		background: var(--text-quaternary);
		border-radius: 2px;
	}
</style>
