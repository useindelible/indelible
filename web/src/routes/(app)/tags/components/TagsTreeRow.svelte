<script lang="ts">
	import type { TagResponse } from '$lib/api/generated/types.gen';
	import { t } from '$lib/i18n';

	interface Props {
		tag: TagResponse;
		depth: number;
		hasChildren: boolean;
		expanded: boolean;
		selected: boolean;
		color: string;
		countLabel: string;
		onContextMenu: (tagId: string, x: number, y: number) => void;
		onOpen: (tagId: string) => void;
		onToggleExpand: (tagId: string) => void;
		onToggleSelect: (tagId: string) => void;
	}

	let {
		tag,
		depth,
		hasChildren,
		expanded,
		selected,
		color,
		countLabel,
		onContextMenu,
		onOpen,
		onToggleExpand,
		onToggleSelect
	}: Props = $props();
</script>

<div
	class="tag-row"
	class:selected
	role="button"
	tabindex="0"
	style:--depth={depth}
	onclick={() => onOpen(tag.id)}
	oncontextmenu={(event) => {
		event.preventDefault();
		event.stopPropagation();
		onContextMenu(tag.id, event.clientX, event.clientY);
	}}
	onkeydown={(event) => {
		if (event.key === 'Enter' || event.key === ' ') {
			event.preventDefault();
			onOpen(tag.id);
		}
	}}
>
	<button
		type="button"
		class="tag-checkbox"
		class:checked={selected}
		aria-label={$t(selected ? 'tag_deselect_named' : 'tag_select_named', {
			values: { name: tag.name }
		})}
		onclick={(event) => {
			event.stopPropagation();
			onToggleSelect(tag.id);
		}}
	>
		{#if selected}
			<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
				<polyline points="20 6 9 17 4 12" />
			</svg>
		{/if}
	</button>

	{#if hasChildren}
		<button
			type="button"
			class="tag-disclosure"
			class:expanded
			aria-label={$t(expanded ? 'tag_collapse_children' : 'tag_expand_children')}
			onclick={(event) => {
				event.stopPropagation();
				onToggleExpand(tag.id);
			}}
		>
			<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<polyline points="9 18 15 12 9 6" />
			</svg>
		</button>
	{:else}
		<div class="tag-disclosure-spacer" aria-hidden="true"></div>
	{/if}

	<div class="tag-dot" style:background={color} aria-hidden="true"></div>

	<div class="tag-info">
		<span class="tag-name">{tag.name}</span>
		{#if tag.aliases.length > 0}
			<span class="tag-aka-badge" title={tag.aliases.join(', ')}>{$t('tag_aka')}</span>
		{/if}
	</div>

	<span class="tag-count">{countLabel}</span>

	<button
		type="button"
		class="tag-more-btn"
		aria-label={$t('tag_more_actions', { values: { name: tag.name } })}
		onclick={(event) => {
			event.stopPropagation();
			const rect = event.currentTarget.getBoundingClientRect();
			onContextMenu(tag.id, rect.right - 264, rect.bottom);
		}}
	>
		<svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
			<circle cx="12" cy="5" r="1.5" />
			<circle cx="12" cy="12" r="1.5" />
			<circle cx="12" cy="19" r="1.5" />
		</svg>
	</button>

	<span class="tag-chevron" aria-hidden="true">
		<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
			<polyline points="9 18 15 12 9 6" />
		</svg>
	</span>
</div>

<style>
	.tag-row {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 10px 20px;
		padding-left: calc(12px + var(--depth) * 24px);
		border-bottom: 0.5px solid var(--border-primary);
		color: var(--text-primary);
		cursor: pointer;
		position: relative;
		transition: background 120ms ease;
		user-select: none;
	}
	.tag-row:hover {
		background: var(--fill-hover);
	}
	.tag-row.selected {
		background: var(--fill-selected);
	}
	.tag-checkbox,
	.tag-disclosure,
	.tag-more-btn {
		border: 0;
		background: transparent;
		color: var(--text-tertiary);
		cursor: pointer;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		padding: 0;
		flex-shrink: 0;
	}
	.tag-checkbox {
		width: 18px;
		height: 18px;
		border-radius: 4px;
		border: 1.5px solid var(--text-quaternary);
		background: transparent;
		transition: all 150ms ease;
	}
	.tag-checkbox.checked {
		background: var(--accent);
		color: var(--text-on-color);
		border-color: var(--accent);
	}
	.tag-checkbox svg {
		width: 11px;
		height: 11px;
	}
	.tag-disclosure {
		width: 20px;
		height: 20px;
		border-radius: 4px;
		transition: background 120ms ease;
	}
	.tag-disclosure:hover {
		background: var(--fill-hover);
	}
	.tag-disclosure svg {
		width: 12px;
		height: 12px;
		transition: transform 200ms ease;
	}
	.tag-disclosure.expanded svg {
		transform: rotate(90deg);
	}
	.tag-disclosure-spacer {
		width: 20px;
		flex-shrink: 0;
	}
	.tag-dot {
		width: 12px;
		height: 12px;
		border-radius: 50%;
		flex-shrink: 0;
	}
	.tag-info {
		flex: 1;
		overflow: hidden;
		display: flex;
		align-items: center;
		gap: 8px;
		min-width: 0;
	}
	.tag-name {
		font-size: 14px;
		font-weight: 600;
		letter-spacing: -0.01em;
		line-height: 1.4;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.tag-aka-badge {
		font-size: 10px;
		color: var(--text-tertiary);
		border: 1px solid var(--border-hairline);
		border-radius: 999px;
		padding: 1px 5px;
	}
	.tag-count {
		font-size: 12px;
		letter-spacing: -0.005em;
		color: var(--text-tertiary);
		white-space: nowrap;
		flex-shrink: 0;
	}
	.tag-more-btn {
		width: 28px;
		height: 28px;
		border-radius: 7px;
		opacity: 0;
		transition:
			opacity 120ms ease,
			background 120ms ease;
	}
	.tag-more-btn:hover {
		background: var(--fill-hover);
	}
	.tag-row:hover .tag-more-btn,
	.tag-more-btn:focus-visible {
		opacity: 1;
	}
	.tag-more-btn svg {
		width: 16px;
		height: 16px;
	}
	.tag-chevron {
		color: var(--text-quaternary);
		flex-shrink: 0;
		display: flex;
		align-items: center;
	}
	.tag-chevron svg {
		width: 14px;
		height: 14px;
	}
</style>
