<script lang="ts">
	import type { TagResponse } from '$lib/api/generated/types.gen';
	import { t } from '$lib/i18n';

	interface Props {
		menuEl?: HTMLElement;
		showColors: boolean;
		tag: TagResponse;
		left: number;
		top: number;
		palette: string[];
		onApplyColor: (color: string | null) => void;
		onCreateChild: (parentId: string) => void;
		onDelete: (tag: TagResponse) => void;
		onMerge: (tag: TagResponse) => void;
		onRename: (tag: TagResponse) => void;
		onSetParent: (tag: TagResponse) => void;
		onToggleColors: () => void;
	}

	let {
		menuEl = $bindable(),
		showColors,
		tag,
		left,
		top,
		palette,
		onApplyColor,
		onCreateChild,
		onDelete,
		onMerge,
		onRename,
		onSetParent,
		onToggleColors
	}: Props = $props();
</script>

<div
	bind:this={menuEl}
	class="context-menu"
	style:left={`${left}px`}
	style:top={`${top}px`}
	role="menu"
>
	<button type="button" class="ctx-item" role="menuitem" onclick={() => onRename(tag)}>
		{$t('tag_rename')} <span class="ctx-shortcut">R</span>
	</button>
	<button type="button" class="ctx-item" role="menuitem" onclick={() => onCreateChild(tag.id)}>
		{$t('tag_new_child')}
	</button>
	<button type="button" class="ctx-item" role="menuitem" onclick={onToggleColors}>
		{$t('tag_color')}
	</button>

	{#if showColors}
		<div class="ctx-colors" role="group" aria-label={$t('tag_color_options')}>
			<button
				type="button"
				class="ctx-color-swatch ctx-color-none"
				title={$t('tag_color_none')}
				aria-label={$t('tag_color_none')}
				onclick={() => onApplyColor(null)}
			>
				<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
					<line x1="18" y1="6" x2="6" y2="18" />
					<line x1="6" y1="6" x2="18" y2="18" />
				</svg>
			</button>
			{#each palette as color (color)}
				<button
					type="button"
					class="ctx-color-swatch"
					class:active={tag.color === color}
					style:background={color}
					title={color}
					aria-label={color}
					onclick={() => onApplyColor(color)}
				></button>
			{/each}
		</div>
	{/if}

	<div class="ctx-divider"></div>
	<button type="button" class="ctx-item" role="menuitem" onclick={() => onSetParent(tag)}>
		{$t('tag_set_parent')}
	</button>
	<button type="button" class="ctx-item ctx-item-disabled" role="menuitem" aria-disabled="true">
		{$t('tag_add_description')} <span class="ctx-badge-soon">{$t('tag_soon')}</span>
	</button>
	<div class="ctx-divider"></div>
	<button type="button" class="ctx-item" role="menuitem" onclick={() => onMerge(tag)}>
		{$t('tag_merge_into')}…
	</button>
	<div class="ctx-divider"></div>
	<button
		type="button"
		class="ctx-item ctx-item-destructive"
		role="menuitem"
		onclick={() => onDelete(tag)}
	>
		{$t('common_delete')}
		<span class="ctx-shortcut ctx-shortcut-destructive">{$t('common_delete')}</span>
	</button>
</div>

<style>
	.context-menu {
		position: fixed;
		z-index: 200;
		width: 264px;
		padding: 4px 0;
		border-radius: 12px;
		border: 0.5px solid var(--border-secondary);
		background: var(--bg-elevated);
		backdrop-filter: blur(50px) saturate(200%);
		-webkit-backdrop-filter: blur(50px) saturate(200%);
		box-shadow:
			0 8px 40px rgba(0, 0, 0, 0.14),
			0 0 0 0.5px rgba(0, 0, 0, 0.06);
		animation: menuIn 150ms ease-out;
	}
	@keyframes menuIn {
		from {
			opacity: 0;
			transform: scale(0.96) translateY(-4px);
		}
		to {
			opacity: 1;
			transform: scale(1) translateY(0);
		}
	}
	.ctx-item {
		width: 100%;
		display: flex;
		align-items: center;
		gap: 10px;
		border: 0;
		background: transparent;
		color: var(--text-primary);
		font: 400 13px/1.2 var(--font-sans);
		letter-spacing: -0.01em;
		padding: 8px 14px;
		cursor: pointer;
		text-align: left;
	}
	.ctx-item:hover {
		background: var(--fill-selected);
	}
	.ctx-item-destructive {
		color: var(--destructive);
	}
	.ctx-item-disabled {
		color: var(--text-quaternary);
		cursor: default;
	}
	.ctx-divider {
		height: 0.5px;
		background: var(--border-primary);
		margin: 4px 0;
	}
	.ctx-shortcut,
	.ctx-badge-soon {
		margin-left: auto;
		font-size: 11px;
		color: var(--text-tertiary);
	}
	.ctx-shortcut-destructive {
		color: var(--destructive);
	}
	.ctx-badge-soon {
		border: 1px solid var(--border-hairline);
		border-radius: 999px;
		padding: 1px 6px;
	}
	.ctx-colors {
		display: flex;
		gap: 6px;
		padding: 4px 14px 8px;
		align-items: center;
		flex-wrap: wrap;
	}
	.ctx-color-swatch {
		width: 20px;
		height: 20px;
		border-radius: 50%;
		border: 2px solid transparent;
		cursor: pointer;
		padding: 0;
		transition:
			border-color 120ms ease,
			transform 100ms ease;
	}
	.ctx-color-swatch:hover {
		transform: scale(1.15);
	}
	.ctx-color-swatch.active {
		border-color: var(--text-primary);
	}
	.ctx-color-none {
		background: var(--bg-secondary);
		color: var(--text-tertiary);
	}
	.ctx-color-none svg {
		width: 15px;
		height: 15px;
	}
</style>
