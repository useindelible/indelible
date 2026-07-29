<script lang="ts">
	import type { HighlightColorOption } from './highlight-toolbar-model';

	interface Props {
		x: number;
		y: number;
		colors: HighlightColorOption[];
		activeColor: string | null;
		hasTags: boolean;
		onColorChange: (color: string) => void;
		onCopy: () => void;
		onTag: () => void;
		onDelete: () => void;
		onMouseEnter: () => void;
		onMouseLeave: () => void;
	}

	let {
		x,
		y,
		colors,
		activeColor,
		hasTags,
		onColorChange,
		onCopy,
		onTag,
		onDelete,
		onMouseEnter,
		onMouseLeave
	}: Props = $props();
</script>

<div
	class="highlight-context-menu highlight-toolbar"
	style:left="{x}px"
	style:top="{y - 50}px"
	style:position="fixed"
	style:transform="translateX(-50%)"
	onmouseenter={onMouseEnter}
	onmouseleave={onMouseLeave}
>
	{#each colors as color (color.name)}
		<button
			type="button"
			class="hl-color-btn"
			class:active-color={activeColor === color.name}
			style:background={color.hex}
			aria-label="Change to {color.name}"
			onclick={() => onColorChange(color.name)}
		></button>
	{/each}
	<div class="hl-divider"></div>
	<button type="button" class="hl-action-btn" onclick={onCopy}> Copy </button>
	<button type="button" class="hl-action-btn hl-tag-btn" class:has-tags={hasTags} onclick={onTag}>
		<svg
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
			><path d="M20.59 13.41l-7.17 7.17a2 2 0 01-2.83 0L2 12V2h10l8.59 8.59a2 2 0 010 2.82z" /><line
				x1="7"
				y1="7"
				x2="7.01"
				y2="7"
			/></svg
		>
		Tag
	</button>
	<button type="button" class="hl-action-btn hl-delete-btn" onclick={onDelete}>Delete</button>
</div>

<style>
	.highlight-toolbar {
		position: absolute;
		transform: translateX(-50%);
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 10px;
		background: var(--bg-elevated);
		backdrop-filter: blur(20px) saturate(180%);
		-webkit-backdrop-filter: blur(20px) saturate(180%);
		border-radius: 10px;
		box-shadow: var(--shadow-3);
		z-index: 20;
	}

	.hl-color-btn {
		width: 22px;
		height: 22px;
		border-radius: 50%;
		cursor: pointer;
		border: 2px solid transparent;
		transition: transform 150ms ease;
		padding: 0;
	}

	.hl-color-btn:hover {
		transform: scale(1.15);
	}

	.hl-color-btn.active-color {
		border-color: var(--text-primary);
	}

	.hl-divider {
		width: 0.5px;
		height: 20px;
		background: var(--border-secondary);
		margin: 0 2px;
	}

	.hl-action-btn {
		padding: 4px 10px;
		border-radius: 6px;
		font-size: 12px;
		font-weight: 500;
		color: var(--text-primary);
		cursor: pointer;
		transition: background 120ms ease;
		letter-spacing: -0.005em;
		white-space: nowrap;
		border: none;
		background: transparent;
		font-family: var(--font-sans);
	}

	.hl-action-btn:hover {
		background: var(--fill-hover);
	}

	.hl-delete-btn {
		color: var(--destructive);
	}

	.hl-tag-btn {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.hl-tag-btn svg {
		width: 12px;
		height: 12px;
		flex-shrink: 0;
	}

	.hl-tag-btn.has-tags {
		color: var(--accent);
	}
</style>
