<script lang="ts">
	import { resolve } from '$app/paths';
	import type { TagResponse } from '$lib/api/generated/types.gen';
	import { sanitizeColor } from '$lib/utils/color';

	interface Props {
		tag: TagResponse;
		selected?: boolean;
		selectable?: boolean;
		onSelect?: (id: string) => void;
		onEdit?: (tag: TagResponse) => void;
		onDelete?: (tag: TagResponse) => void;
	}

	let { tag, selected = false, selectable = false, onSelect, onEdit, onDelete }: Props = $props();

	const totalCount = $derived(tag.item_count + tag.highlight_count);
</script>

<div class="tag-card" class:selected>
	{#if selectable}
		<button
			type="button"
			class="select-check"
			class:checked={selected}
			aria-label={selected ? `Deselect ${tag.name}` : `Select ${tag.name}`}
			onclick={(e) => {
				e.stopPropagation();
				onSelect?.(tag.id);
			}}
		>
			{#if selected}
				<svg
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2.5"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<polyline points="20 6 9 17 4 12" />
				</svg>
			{/if}
		</button>
	{/if}

	<a href={resolve('/(app)/tags/[id]', { id: tag.id })} class="tag-link">
		<span
			class="tag-dot"
			style="background: {sanitizeColor(tag.color) ?? 'var(--text-tertiary)'}"
			aria-hidden="true"
		></span>
		<span class="tag-name">{tag.name}</span>
		<span class="tag-count">{totalCount}</span>
	</a>

	{#if onEdit || onDelete}
		<div class="tag-actions">
			{#if onEdit}
				<button
					type="button"
					class="action-btn"
					aria-label="Edit {tag.name}"
					onclick={(e) => {
						e.stopPropagation();
						onEdit?.(tag);
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
			{/if}
			{#if onDelete}
				<button
					type="button"
					class="action-btn action-btn-danger"
					aria-label="Delete {tag.name}"
					onclick={(e) => {
						e.stopPropagation();
						onDelete?.(tag);
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
			{/if}
		</div>
	{/if}
</div>

<style>
	.tag-card {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 14px;
		border-radius: 10px;
		border: 1px solid var(--border-primary);
		background: var(--bg-secondary);
		transition:
			border-color 0.15s ease,
			box-shadow 0.15s ease;
	}

	.tag-card:hover {
		border-color: var(--border-secondary);
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
	}

	.tag-card.selected {
		border-color: var(--accent);
		background: var(--fill-selected);
	}

	.select-check {
		width: 20px;
		height: 20px;
		border-radius: 5px;
		border: 1.5px solid var(--border-secondary);
		background: transparent;
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: pointer;
		padding: 0;
		flex-shrink: 0;
		transition:
			background 0.12s ease,
			border-color 0.12s ease;
	}

	.select-check.checked {
		background: var(--accent);
		border-color: var(--accent);
	}

	.select-check svg {
		width: 12px;
		height: 12px;
		color: var(--text-on-color);
	}

	.tag-link {
		display: flex;
		align-items: center;
		gap: 10px;
		flex: 1;
		min-width: 0;
		text-decoration: none;
		color: inherit;
	}

	.tag-dot {
		width: 10px;
		height: 10px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.tag-name {
		font-family: var(--font-sans);
		font-size: 14px;
		font-weight: 500;
		letter-spacing: -0.01em;
		color: var(--text-primary);
		flex: 1;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.tag-count {
		font-family: var(--font-sans);
		font-size: 12px;
		font-weight: 400;
		color: var(--text-tertiary);
		flex-shrink: 0;
	}

	.tag-actions {
		display: flex;
		align-items: center;
		gap: 4px;
		opacity: 0;
		transition: opacity 0.12s ease;
	}

	.tag-card:hover .tag-actions {
		opacity: 1;
	}

	.action-btn {
		width: 24px;
		height: 24px;
		border-radius: 6px;
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
		color: var(--red);
	}

	.action-btn svg {
		width: 14px;
		height: 14px;
	}
</style>
