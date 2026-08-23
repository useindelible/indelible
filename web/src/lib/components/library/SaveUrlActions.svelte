<script lang="ts">
	import type { CollectionResponse } from '$lib/api/generated/types.gen';
	import { t } from '$lib/i18n';
	import SaveUrlCollectionPicker from './SaveUrlCollectionPicker.svelte';
	import SaveUrlTagEditor from './SaveUrlTagEditor.svelte';

	interface Props {
		tags: string[];
		addingTag: boolean;
		newTagValue: string;
		submitting: boolean;
		canSave: boolean;
		collectionId: string | null;
		collections: CollectionResponse[];
		selectedCollectionName: string;
		pickerOpen: boolean;
		pickerPos: { top: number; left: number };
		pickerTriggerEl?: HTMLButtonElement;
		pickerDropdownEl?: HTMLDivElement;
		onTogglePicker: () => void;
		onSelectCollection: (id: string | null) => void;
		onAddTag: (raw: string) => void;
		onRemoveTag: (tag: string) => void;
		onTagKeydown: (event: KeyboardEvent) => void;
		onSave: () => void;
	}

	let {
		tags,
		addingTag = $bindable(),
		newTagValue = $bindable(),
		submitting,
		canSave,
		collectionId,
		collections,
		selectedCollectionName,
		pickerOpen,
		pickerPos,
		pickerTriggerEl = $bindable(),
		pickerDropdownEl = $bindable(),
		onTogglePicker,
		onSelectCollection,
		onAddTag,
		onRemoveTag,
		onTagKeydown,
		onSave
	}: Props = $props();
</script>

<div class="cmd-controls">
	<SaveUrlCollectionPicker
		{collectionId}
		{collections}
		{selectedCollectionName}
		{pickerOpen}
		{pickerPos}
		bind:pickerTriggerEl
		bind:pickerDropdownEl
		{onTogglePicker}
		{onSelectCollection}
	/>

	<SaveUrlTagEditor
		{tags}
		bind:addingTag
		bind:newTagValue
		{onAddTag}
		{onRemoveTag}
		{onTagKeydown}
	/>

	<button
		type="button"
		class="cmd-action"
		class:loading={submitting}
		disabled={!canSave}
		onclick={onSave}
	>
		{#if submitting}
			<span class="spinner" aria-hidden="true"></span>
			<span class="sr-only">{$t('common_saving')}</span>
		{:else}
			{$t('common_save')}
		{/if}
	</button>
</div>

<style>
	.cmd-controls {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 10px 16px 14px;
		flex-wrap: wrap;
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
		display: flex;
		align-items: center;
		gap: 6px;
		flex-shrink: 0;
		transition: opacity 120ms ease;
	}

	.cmd-action:hover:not(:disabled) {
		opacity: 0.88;
	}

	.cmd-action:disabled {
		opacity: 0.32;
		cursor: not-allowed;
	}

	.spinner {
		display: inline-block;
		width: 12px;
		height: 12px;
		border: 2px solid rgba(255, 255, 255, 0.35);
		border-top-color: var(--text-on-color);
		border-radius: 50%;
		animation: spin 0.6s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.sr-only {
		position: absolute;
		width: 1px;
		height: 1px;
		padding: 0;
		margin: -1px;
		overflow: hidden;
		clip: rect(0, 0, 0, 0);
		white-space: nowrap;
		border: 0;
	}
</style>
