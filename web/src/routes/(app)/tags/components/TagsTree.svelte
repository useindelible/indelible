<script lang="ts">
	import TagsTreeRow from './TagsTreeRow.svelte';
	import { getTagCountLabel, tagDisplayColor, type TagNode, type TagScope } from '../tag-tree';

	interface Props {
		activeScope: TagScope;
		expandedParents: Set<string>;
		fetchError: string | null;
		isEmpty: boolean;
		loading: boolean;
		nodes: TagNode[];
		rolledUpCounts: Map<string, number>;
		selectedIds: Set<string>;
		totalCount: number;
		onContextMenu: (tagId: string, x: number, y: number) => void;
		onCreate: () => void;
		onOpen: (tagId: string) => void;
		onToggleExpand: (tagId: string) => void;
		onToggleSelect: (tagId: string) => void;
	}

	let {
		activeScope,
		expandedParents,
		fetchError,
		isEmpty,
		loading,
		nodes,
		rolledUpCounts,
		selectedIds,
		totalCount,
		onContextMenu,
		onCreate,
		onOpen,
		onToggleExpand,
		onToggleSelect
	}: Props = $props();
</script>

<div class="tags-list-area">
	{#if loading}
		<div class="empty-state">
			<span class="state-text">Loading tags…</span>
		</div>
	{:else if isEmpty}
		<div class="empty-state">
			<div class="empty-icon" aria-hidden="true">
				<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2">
					<path
						d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"
					/>
					<circle cx="7" cy="7" r="1.5" fill="currentColor" stroke="none" />
				</svg>
			</div>
			<p class="empty-heading">No tags yet</p>
			<p class="empty-sub">Create a tag to start organizing your content.</p>
			<button type="button" class="cmd-action" onclick={onCreate}>Create your first tag</button>
		</div>
	{:else if totalCount === 0}
		<div class="empty-state">
			<p class="empty-heading">No matching tags</p>
			<p class="empty-sub">Try a different search term.</p>
		</div>
	{:else}
		{#each nodes as node (node.tag.id)}
			<TagsTreeRow
				tag={node.tag}
				depth={node.depth}
				hasChildren={node.hasChildren}
				expanded={expandedParents.has(node.tag.id)}
				selected={selectedIds.has(node.tag.id)}
				color={tagDisplayColor(node.tag)}
				countLabel={getTagCountLabel(node.tag, activeScope, rolledUpCounts)}
				{onContextMenu}
				{onOpen}
				{onToggleExpand}
				{onToggleSelect}
			/>
		{/each}
	{/if}
</div>

{#if fetchError}
	<div class="error-banner" role="alert">{fetchError}</div>
{/if}

{#if totalCount > 0}
	<span class="item-count" aria-live="polite">
		{totalCount} tag{totalCount !== 1 ? 's' : ''}
	</span>
{/if}

<style>
	.tags-list-area {
		flex: 1;
		overflow-y: auto;
	}
	.empty-state {
		min-height: 320px;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		text-align: center;
		color: var(--text-tertiary);
		padding: 32px;
	}
	.empty-icon {
		width: 52px;
		height: 52px;
		color: var(--text-quaternary);
		margin-bottom: 12px;
	}
	.empty-icon svg {
		width: 100%;
		height: 100%;
	}
	.empty-heading {
		color: var(--text-primary);
		font-size: 15px;
		font-weight: 600;
		margin: 0 0 4px;
	}
	.empty-sub,
	.state-text {
		margin: 0;
		font-size: 13px;
		color: var(--text-tertiary);
	}
	.cmd-action {
		margin-top: 18px;
		border: 0;
		border-radius: 8px;
		background: var(--accent);
		color: var(--text-on-color);
		font-size: 13px;
		font-weight: 600;
		padding: 8px 13px;
		cursor: pointer;
	}
	.error-banner {
		margin: 12px 16px;
		padding: 10px 12px;
		border-radius: 8px;
		background: var(--fill-danger);
		color: var(--destructive);
		border: 1px solid var(--destructive);
		font-size: 13px;
	}
	.item-count {
		position: absolute;
		right: 20px;
		bottom: 16px;
		font-size: 12px;
		font-weight: 500;
		color: var(--text-tertiary);
		pointer-events: none;
	}
</style>
