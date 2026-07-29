<script lang="ts">
	import * as apiSdk from '$lib/api';
	import type { CollectionResponse } from '$lib/api/generated/types.gen';
	import { fetchAllPages } from '$lib/api/pagination';
	import { getModalStore } from '$lib/stores/addItemModal.svelte';
	import DuplicateUrlPreview from './DuplicateUrlPreview.svelte';
	import SaveUrlActions from './SaveUrlActions.svelte';
	import SaveUrlInputZone from './SaveUrlInputZone.svelte';
	import {
		addSaveUrlTag,
		duplicateFromConflictError,
		getSelectedCollectionName,
		messageForSaveUrlProblem,
		messageForUrlValidation,
		removeSaveUrlTag,
		validateSaveUrl,
		type DuplicateUrlInfo
	} from './save-url-model';

	const modal = getModalStore();

	let dialogEl = $state<HTMLDialogElement | undefined>(undefined);

	let url = $state('');
	let tags = $state<string[]>([]);
	let submitting = $state(false);
	let submitError = $state('');
	let addingTag = $state(false);
	let newTagValue = $state('');
	let duplicate = $state<DuplicateUrlInfo | null>(null);

	let collectionId = $state<string | null>(null);
	let collections = $state<CollectionResponse[]>([]);
	let collectionsLoaded = $state(false);
	let pickerOpen = $state(false);
	let pickerTriggerEl = $state<HTMLButtonElement | undefined>(undefined);
	let pickerDropdownEl = $state<HTMLDivElement | undefined>(undefined);
	let pickerPos = $state({ top: 0, left: 0 });

	let selectedCollectionName = $derived(getSelectedCollectionName(collectionId, collections));
	let isOpen = $derived(modal.active === 'url');
	let hasUrl = $derived(url.trim().length > 0);
	let canSave = $derived(hasUrl && !submitting);

	$effect(() => {
		if (!dialogEl) return;
		if (isOpen) {
			url = '';
			tags = [];
			submitting = false;
			submitError = '';
			duplicate = null;
			addingTag = false;
			newTagValue = '';
			collectionId = null;
			pickerOpen = false;
			dialogEl.showModal();
		} else {
			dialogEl.close();
		}
	});

	$effect(() => {
		if (!pickerOpen) return;
		function handleClickOutside(event: MouseEvent) {
			const target = event.target as Node;
			const inTrigger = pickerTriggerEl?.contains(target) ?? false;
			const inDropdown = pickerDropdownEl?.contains(target) ?? false;
			if (!inTrigger && !inDropdown) pickerOpen = false;
		}
		document.addEventListener('click', handleClickOutside, true);
		return () => document.removeEventListener('click', handleClickOutside, true);
	});

	async function loadCollections() {
		if (collectionsLoaded) return;
		try {
			const results = await fetchAllPages(async (cursor) => {
				const resp = await apiSdk.listCollections({ query: { cursor, limit: 100 } });
				if (!resp.data) return undefined;
				return {
					data: resp.data.data as CollectionResponse[],
					page: { next_cursor: resp.data.page.next_cursor ?? null }
				};
			});
			collections = results;
			collectionsLoaded = true;
		} catch {
			// Silently fail; clicking again retries because loaded stays false.
		}
	}

	function togglePicker() {
		pickerOpen = !pickerOpen;
		if (pickerOpen) {
			void loadCollections();
			if (pickerTriggerEl) {
				const rect = pickerTriggerEl.getBoundingClientRect();
				pickerPos = {
					top: rect.bottom + 4,
					left: rect.left
				};
			}
		}
	}

	function selectCollection(id: string | null) {
		collectionId = id;
		pickerOpen = false;
	}

	function close() {
		modal.close();
	}

	function handleBackdropClick(event: MouseEvent) {
		if (event.target === dialogEl) close();
	}

	async function handleSave() {
		if (!canSave) return;
		const validation = validateSaveUrl(url);
		if (validation) {
			submitError = messageForUrlValidation(validation);
			return;
		}
		submitError = '';
		duplicate = null;
		submitting = true;

		try {
			const {
				data,
				error: apiError,
				response
			} = await apiSdk.createDocumentEntry({
				body: { url: url.trim() }
			});

			if (data) {
				close();
				return;
			}

			if (response.status === 409) {
				const conflictDuplicate = duplicateFromConflictError(apiError);
				if (conflictDuplicate) {
					duplicate = conflictDuplicate;
				} else {
					submitError = 'This URL is already in your library.';
				}
				return;
			}

			submitError = messageForSaveUrlProblem(apiError);
		} catch {
			submitError = 'An unexpected error occurred.';
		} finally {
			submitting = false;
		}
	}

	async function handleSaveAsNew() {
		duplicate = null;
		await handleSave();
	}

	function handleRefresh() {
		if (duplicate) {
			close();
		}
	}

	function addTag(raw: string) {
		tags = addSaveUrlTag(tags, raw);
		newTagValue = '';
		addingTag = false;
	}

	function removeTag(tag: string) {
		tags = removeSaveUrlTag(tags, tag);
	}

	function handleTagKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter' || event.key === ',') {
			event.preventDefault();
			addTag(newTagValue);
		} else if (event.key === 'Escape') {
			newTagValue = '';
			addingTag = false;
		} else if (event.key === 'Backspace' && newTagValue === '') {
			addingTag = false;
		}
	}
</script>

<dialog
	bind:this={dialogEl}
	class="modal-backdrop"
	aria-label="Save URL"
	onclick={handleBackdropClick}
	onclose={close}
>
	<div class="cmd-card" role="document">
		<SaveUrlInputZone bind:url hasDuplicate={!!duplicate} onSave={handleSave} onClose={close} />

		{#if duplicate || submitError}
			<DuplicateUrlPreview
				{duplicate}
				{submitError}
				{submitting}
				onRefresh={handleRefresh}
				onSaveAsNew={() => {
					void handleSaveAsNew();
				}}
			/>
		{/if}

		<SaveUrlActions
			{tags}
			bind:addingTag
			bind:newTagValue
			{submitting}
			{canSave}
			{collectionId}
			{collections}
			{selectedCollectionName}
			{pickerOpen}
			{pickerPos}
			bind:pickerTriggerEl
			bind:pickerDropdownEl
			onTogglePicker={togglePicker}
			onSelectCollection={selectCollection}
			onAddTag={addTag}
			onRemoveTag={removeTag}
			onTagKeydown={handleTagKeydown}
			onSave={handleSave}
		/>
	</div>
</dialog>

<style>
	.modal-backdrop {
		position: fixed;
		inset: 0;
		width: 100%;
		height: 100%;
		max-width: 100%;
		max-height: 100%;
		margin: 0;
		padding: 0;
		border: none;
		background: transparent;
		display: flex;
		align-items: flex-start;
		justify-content: center;
		padding-top: 80px;
		box-sizing: border-box;
	}

	.modal-backdrop::backdrop {
		background: rgba(0, 0, 0, 0.4);
		backdrop-filter: blur(4px);
		-webkit-backdrop-filter: blur(4px);
	}

	.cmd-card {
		width: 440px;
		max-width: calc(100vw - 32px);
		background: var(--bg-elevated);
		border-radius: 14px;
		box-shadow:
			0 24px 80px rgba(0, 0, 0, 0.22),
			0 0 0 0.5px rgba(0, 0, 0, 0.06);
		overflow: hidden;
	}

	[data-theme='dark'] .cmd-card {
		box-shadow:
			0 24px 80px rgba(0, 0, 0, 0.55),
			0 0 0 0.5px rgba(255, 255, 255, 0.08);
	}
</style>
