<script lang="ts">
	import { onMount } from 'svelte';
	import type { TagResponse } from '$lib/api/generated/types.gen';
	import { SvelteSet } from 'svelte/reactivity';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { getTags } from '$lib/stores/tags.svelte';
	import { getViewport } from '$lib/stores/viewport.svelte';
	import MergeTagsDialog from '$lib/components/tags/MergeTagsDialog.svelte';
	import CreateTagDialog from './components/CreateTagDialog.svelte';
	import DeleteTagDialog from './components/DeleteTagDialog.svelte';
	import RenameTagDialog from './components/RenameTagDialog.svelte';
	import SetParentDialog from './components/SetParentDialog.svelte';
	import TagContextMenu from './components/TagContextMenu.svelte';
	import TagsToolbar from './components/TagsToolbar.svelte';
	import TagsTree from './components/TagsTree.svelte';
	import { buildTagTree, parentOptions, rolledUpItemCounts, type TagScope } from './tag-tree';

	const store = getTags();
	const vp = getViewport();

	onMount(() => {
		store.loadAllTags();
	});

	let expandedParents = new SvelteSet<string>();
	let contextMenuTagId = $state<string | null>(null);
	let contextMenuShowColors = $state(false);
	let contextMenuLeft = $state(0);
	let contextMenuTop = $state(0);
	let contextMenuEl: HTMLElement | undefined = $state();
	let showCreateModal = $state(false);
	let createName = $state('');
	let createColor = $state<string | null>(null);
	let createParentId = $state<string | null>(null);
	let renameTag = $state<TagResponse | null>(null);
	let renameValue = $state('');
	let setParentForTag = $state<TagResponse | null>(null);
	let selectedParentId = $state('');
	let deletingTag = $state<TagResponse | null>(null);
	let mergeSourceTag = $state<TagResponse | null>(null);
	let showBulkMergeDialog = $state(false);

	const CTX_PALETTE = [
		'#0A84FF',
		'#30D158',
		'#FF9F0A',
		'#FF453A',
		'#BF5AF2',
		'#FF2D55',
		'#64D2FF',
		'#FFD60A'
	];

	const contextMenuTag = $derived(
		contextMenuTagId ? (store.allTags.find((tag) => tag.id === contextMenuTagId) ?? null) : null
	);
	const selectedTags = $derived(store.allTags.filter((tag) => store.selectedIds.has(tag.id)));
	const bulkMode = $derived(store.selectedIds.size > 0);
	const tagTree = $derived(buildTagTree(store.filteredTags, expandedParents));
	const rolledUpCount = $derived(rolledUpItemCounts(store.allTags));
	const totalCount = $derived(store.filteredTags.length);

	$effect(() => {
		if (!contextMenuTagId) return;
		function onPointerDown(event: PointerEvent) {
			if (contextMenuEl && !contextMenuEl.contains(event.target as Node)) {
				closeContextMenu();
			}
		}
		document.addEventListener('pointerdown', onPointerDown, true);
		return () => document.removeEventListener('pointerdown', onPointerDown, true);
	});

	function toggleExpand(id: string) {
		if (expandedParents.has(id)) expandedParents.delete(id);
		else expandedParents.add(id);
	}

	function openContextMenuAt(tagId: string, x: number, y: number) {
		const menuWidth = 264;
		const menuHeight = 420;
		contextMenuLeft = Math.max(16, Math.min(x, window.innerWidth - menuWidth - 16));
		contextMenuTop = Math.min(y + 4, window.innerHeight - menuHeight);
		contextMenuTagId = tagId;
		contextMenuShowColors = false;
	}

	function closeContextMenu() {
		contextMenuTagId = null;
		contextMenuShowColors = false;
	}

	function openCreate(parentId: string | null = null) {
		createName = '';
		createColor = null;
		createParentId = parentId;
		showCreateModal = true;
	}

	async function submitCreate() {
		if (!createName.trim()) return;
		const created = await store.createTag({
			name: createName.trim(),
			color: createColor,
			parent_id: createParentId
		});
		if (created) {
			showCreateModal = false;
			await store.loadAllTags();
			goto(resolve('/(app)/tags/[id]', { id: created.id }));
		}
	}

	function startRename(tag: TagResponse) {
		renameTag = tag;
		renameValue = tag.name;
		closeContextMenu();
	}

	async function submitRename() {
		if (!renameTag || !renameValue.trim()) return;
		await store.updateTag(renameTag.id, { name: renameValue.trim() });
		renameTag = null;
	}

	async function applyColor(color: string | null) {
		if (!contextMenuTagId) return;
		await store.updateTag(contextMenuTagId, { color });
		closeContextMenu();
	}

	function startSetParent(tag: TagResponse) {
		setParentForTag = tag;
		selectedParentId = tag.parent_id ?? '';
		closeContextMenu();
	}

	async function submitSetParent() {
		if (!setParentForTag) return;
		await store.updateTag(setParentForTag.id, { parent_id: selectedParentId || null });
		await store.loadAllTags();
		setParentForTag = null;
	}

	function startDelete(tag: TagResponse) {
		deletingTag = tag;
		closeContextMenu();
	}

	async function confirmDelete() {
		if (!deletingTag) return;
		await store.deleteTag(deletingTag.id);
		deletingTag = null;
	}

	function startMerge(tag: TagResponse) {
		mergeSourceTag = tag;
		closeContextMenu();
	}

	async function handleSingleMerge(sourceIds: string[], targetId: string) {
		const ok = await store.mergeTags(sourceIds, targetId);
		if (ok) mergeSourceTag = null;
	}

	async function handleBulkMerge(sourceIds: string[], targetId: string) {
		const ok = await store.mergeTags(sourceIds, targetId);
		if (ok) {
			showBulkMergeDialog = false;
			store.clearSelection();
		}
	}

	async function bulkDelete() {
		const ids = [...store.selectedIds];
		for (const id of ids) {
			await store.deleteTag(id);
		}
		store.clearSelection();
	}

	function handleScopeClick(scope: Exclude<TagScope, 'all'>) {
		store.setScope(store.activeScope === scope ? 'all' : scope);
	}

	function openTag(tagId: string) {
		goto(resolve('/(app)/tags/[id]', { id: tagId }));
	}
</script>

<div class="tags-page">
	<TagsToolbar
		activeScope={store.activeScope}
		{bulkMode}
		searchQuery={store.searchQuery}
		selectedCount={store.selectedIds.size}
		onBulkDelete={bulkDelete}
		onBulkMerge={() => (showBulkMergeDialog = true)}
		onClearSelection={() => store.clearSelection()}
		onCreate={() => openCreate()}
		onScopeClick={handleScopeClick}
		onSearch={(value) => store.setSearchQuery(value)}
		onMenuClick={() => vp.openMobileNav()}
	/>

	<TagsTree
		activeScope={store.activeScope}
		{expandedParents}
		fetchError={store.fetchError}
		isEmpty={store.isEmpty}
		loading={store.loading}
		nodes={tagTree}
		rolledUpCounts={rolledUpCount}
		selectedIds={store.selectedIds}
		{totalCount}
		onContextMenu={openContextMenuAt}
		onCreate={() => openCreate()}
		onOpen={openTag}
		onToggleExpand={toggleExpand}
		onToggleSelect={(tagId) => store.toggleSelection(tagId)}
	/>
</div>

{#if contextMenuTagId && contextMenuTag}
	<TagContextMenu
		bind:menuEl={contextMenuEl}
		tag={contextMenuTag}
		showColors={contextMenuShowColors}
		left={contextMenuLeft}
		top={contextMenuTop}
		palette={CTX_PALETTE}
		onApplyColor={applyColor}
		onCreateChild={(parentId) => {
			closeContextMenu();
			openCreate(parentId);
		}}
		onDelete={startDelete}
		onMerge={startMerge}
		onRename={startRename}
		onSetParent={startSetParent}
		onToggleColors={() => (contextMenuShowColors = !contextMenuShowColors)}
	/>
{/if}

{#if showCreateModal}
	<CreateTagDialog
		name={createName}
		color={createColor}
		parentId={createParentId}
		onClose={() => (showCreateModal = false)}
		onColorChange={(color) => (createColor = color)}
		onNameChange={(name) => (createName = name)}
		onSubmit={submitCreate}
	/>
{/if}

{#if renameTag}
	<RenameTagDialog
		value={renameValue}
		onClose={() => (renameTag = null)}
		onSubmit={submitRename}
		onValueChange={(value) => (renameValue = value)}
	/>
{/if}

{#if setParentForTag}
	<SetParentDialog
		options={parentOptions(store.allTags, setParentForTag)}
		{selectedParentId}
		onClose={() => (setParentForTag = null)}
		onParentChange={(id) => (selectedParentId = id)}
		onSubmit={submitSetParent}
	/>
{/if}

{#if deletingTag}
	<DeleteTagDialog onClose={() => (deletingTag = null)} onConfirm={confirmDelete} />
{/if}

{#if mergeSourceTag}
	<MergeTagsDialog
		sourceTags={[mergeSourceTag]}
		allTags={store.allTags}
		onMerge={handleSingleMerge}
		onClose={() => {
			mergeSourceTag = null;
		}}
	/>
{/if}

{#if showBulkMergeDialog}
	<MergeTagsDialog
		sourceTags={selectedTags}
		allTags={store.allTags}
		onMerge={handleBulkMerge}
		onClose={() => {
			showBulkMergeDialog = false;
		}}
	/>
{/if}

<style>
	.tags-page {
		flex: 1;
		overflow: hidden;
		display: flex;
		flex-direction: column;
		position: relative;
		background: var(--bg-content);
	}
</style>
