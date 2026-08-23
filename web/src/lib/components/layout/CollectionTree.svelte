<script lang="ts">
	import { page } from '$app/state';
	import { resolve } from '$app/paths';
	import { sanitizeColor } from '$lib/utils/color';
	import type { CollectionNode } from '$lib/api/pagination';
	import { t } from '$lib/i18n';

	interface Props {
		nodes: CollectionNode[];
		depth?: number;
		onToggle: (id: string) => void;
	}

	let { nodes, depth = 0, onToggle }: Props = $props();

	const INDENT_PX = 12;

	function isActive(collectionId: string): boolean {
		return page.url.pathname === `/collections/${collectionId}`;
	}
</script>

<ul class="tree-list" role="group">
	{#each nodes as node (node.collection.id)}
		{@const hasChildren = node.children.length > 0}
		<li>
			<div
				class="tree-item"
				class:active={isActive(node.collection.id)}
				style="padding-left: {12 + depth * INDENT_PX}px"
			>
				<button
					type="button"
					class="tree-chevron"
					class:has-children={hasChildren}
					class:expanded={node.expanded}
					aria-label={hasChildren
						? node.expanded
							? $t('common_collapse')
							: $t('common_expand')
						: undefined}
					onclick={(e) => {
						e.stopPropagation();
						if (hasChildren) onToggle(node.collection.id);
					}}
					tabindex={hasChildren ? 0 : -1}
				>
					{#if hasChildren}
						<svg
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
							stroke-linecap="round"
							stroke-linejoin="round"
						>
							<polyline points="9 6 15 12 9 18" />
						</svg>
					{/if}
				</button>

				<a
					href={resolve('/(app)/collections/[id]', { id: node.collection.id })}
					class="tree-link"
					aria-current={isActive(node.collection.id) ? 'page' : undefined}
				>
					{#if node.collection.icon}
						<span class="tree-icon-custom" aria-hidden="true">{node.collection.icon}</span>
					{:else}
						<span
							class="tree-icon"
							aria-hidden="true"
							style={sanitizeColor(node.collection.color)
								? `color: ${sanitizeColor(node.collection.color)}`
								: ''}
						>
							<svg
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="1.6"
								stroke-linecap="round"
								stroke-linejoin="round"
							>
								<path
									d="M3 7v10a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-6l-2-2H5a2 2 0 0 0-2 2z"
								/>
							</svg>
						</span>
					{/if}
					<span class="tree-name">{node.collection.name}</span>
					{#if node.collection.item_count > 0}
						<span class="tree-badge">{node.collection.item_count}</span>
					{/if}
				</a>
			</div>

			{#if hasChildren && node.expanded}
				<svelte:self nodes={node.children} depth={depth + 1} {onToggle} />
			{/if}
		</li>
	{/each}
</ul>

<style>
	.tree-list {
		list-style: none;
		margin: 0;
		padding: 0;
	}

	.tree-item {
		display: flex;
		align-items: center;
		gap: 2px;
		padding-right: 12px;
		border-radius: 8px;
		transition: background 0.12s ease;
	}

	.tree-item:hover {
		background: var(--fill-hover);
	}

	.tree-item.active {
		background: var(--fill-selected-strong);
	}

	.tree-item.active .tree-link {
		color: var(--accent);
		font-weight: 500;
	}

	.tree-chevron {
		width: 16px;
		height: 16px;
		display: flex;
		align-items: center;
		justify-content: center;
		background: none;
		border: none;
		padding: 0;
		color: var(--text-tertiary);
		flex-shrink: 0;
		transition: transform 0.15s ease;
	}

	.tree-chevron.has-children {
		cursor: pointer;
	}

	.tree-chevron.has-children:hover {
		color: var(--text-secondary);
	}

	.tree-chevron.expanded {
		transform: rotate(90deg);
	}

	.tree-chevron svg {
		width: 12px;
		height: 12px;
	}

	.tree-link {
		display: flex;
		align-items: center;
		gap: 8px;
		flex: 1;
		min-width: 0;
		padding: 6px 4px;
		text-decoration: none;
		color: var(--text-primary);
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 400;
		letter-spacing: -0.01em;
		line-height: 1.45;
	}

	.tree-icon {
		width: 16px;
		height: 16px;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.tree-icon svg {
		width: 16px;
		height: 16px;
	}

	.tree-icon-custom {
		width: 16px;
		height: 16px;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
		font-size: 14px;
		line-height: 1;
	}

	.tree-name {
		flex: 1;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.tree-badge {
		font-size: 11px;
		font-weight: 500;
		color: var(--text-tertiary);
		flex-shrink: 0;
	}
</style>
