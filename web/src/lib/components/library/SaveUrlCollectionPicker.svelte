<script lang="ts">
	import { t } from '$lib/i18n';
	import type { CollectionResponse } from '$lib/api/generated/types.gen';

	interface Props {
		collectionId: string | null;
		collections: CollectionResponse[];
		selectedCollectionName: string;
		pickerOpen: boolean;
		pickerPos: { top: number; left: number };
		pickerTriggerEl?: HTMLButtonElement;
		pickerDropdownEl?: HTMLDivElement;
		onTogglePicker: () => void;
		onSelectCollection: (id: string | null) => void;
	}

	let {
		collectionId,
		collections,
		selectedCollectionName,
		pickerOpen,
		pickerPos,
		pickerTriggerEl = $bindable(),
		pickerDropdownEl = $bindable(),
		onTogglePicker,
		onSelectCollection
	}: Props = $props();
</script>

<div class="collection-wrap">
	<button
		bind:this={pickerTriggerEl}
		type="button"
		class="cmd-collection"
		aria-label={$t('library_choose_collection')}
		aria-expanded={pickerOpen}
		onclick={onTogglePicker}
	>
		<svg viewBox="0 0 24 24" aria-hidden="true">
			<path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
		</svg>
		{selectedCollectionName}
		<svg class="chev" viewBox="0 0 24 24" aria-hidden="true">
			<polyline points="9 6 15 12 9 18" />
		</svg>
	</button>

	{#if pickerOpen}
		<div
			bind:this={pickerDropdownEl}
			class="collection-dropdown"
			style="top: {pickerPos.top}px; left: {pickerPos.left}px;"
			role="listbox"
			aria-label={$t('library_collections')}
		>
			<button
				type="button"
				class="collection-option"
				class:active={collectionId === null}
				role="option"
				aria-selected={collectionId === null}
				onclick={() => onSelectCollection(null)}
			>
				{$t('library_triage_inbox')}
			</button>
			{#each collections as collection (collection.id)}
				<button
					type="button"
					class="collection-option"
					class:active={collectionId === collection.id}
					role="option"
					aria-selected={collectionId === collection.id}
					onclick={() => onSelectCollection(collection.id)}
				>
					{collection.icon ? collection.icon + ' ' : ''}{collection.name}
				</button>
			{/each}
		</div>
	{/if}
</div>

<style>
	.collection-wrap {
		flex-shrink: 0;
	}

	.cmd-collection {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 5px 8px 5px 7px;
		border-radius: 7px;
		background: var(--bg-secondary);
		border: none;
		cursor: pointer;
		font-family: var(--font-sans);
		font-size: 12px;
		font-weight: 500;
		color: var(--text-primary);
		flex-shrink: 0;
	}

	.cmd-collection:hover {
		background: var(--bg-tertiary);
	}

	.cmd-collection svg {
		width: 13px;
		height: 13px;
		stroke: var(--text-secondary);
		fill: none;
		stroke-width: 1.5;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.cmd-collection .chev {
		width: 10px;
		height: 10px;
		stroke: var(--text-quaternary);
		stroke-width: 2;
		margin-left: 2px;
	}

	.collection-dropdown {
		position: fixed;
		min-width: 180px;
		max-height: 220px;
		overflow-y: auto;
		background: var(--bg-elevated);
		border-radius: 10px;
		box-shadow:
			0 8px 32px rgba(0, 0, 0, 0.18),
			0 0 0 0.5px rgba(0, 0, 0, 0.08);
		z-index: 10;
		padding: 4px;
	}

	.collection-option {
		display: block;
		width: 100%;
		padding: 7px 10px;
		border: none;
		background: transparent;
		text-align: left;
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--text-primary);
		border-radius: 6px;
		cursor: pointer;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.collection-option:hover {
		background: var(--fill-hover);
	}

	.collection-option.active {
		background: var(--fill-selected);
		font-weight: 500;
		color: var(--accent);
	}
</style>
