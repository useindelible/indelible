<script lang="ts">
	import type { TagScope } from '../tag-tree';

	interface Props {
		activeScope: TagScope;
		bulkMode: boolean;
		searchQuery: string;
		selectedCount: number;
		onBulkDelete: () => void;
		onBulkMerge: () => void;
		onClearSelection: () => void;
		onCreate: () => void;
		onScopeClick: (scope: Exclude<TagScope, 'all'>) => void;
		onSearch: (value: string) => void;
		onMenuClick?: () => void;
	}

	let {
		activeScope,
		bulkMode,
		searchQuery,
		selectedCount,
		onBulkDelete,
		onBulkMerge,
		onClearSelection,
		onCreate,
		onScopeClick,
		onSearch,
		onMenuClick
	}: Props = $props();
</script>

<div class="tags-header">
	<div class="tags-header-top">
		<div class="tags-title-row">
			{#if onMenuClick}
				<button type="button" class="menu-btn" onclick={onMenuClick} aria-label="Open navigation">
					<svg
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="1.7"
						stroke-linecap="round"
						stroke-linejoin="round"
						aria-hidden="true"
					>
						<line x1="3" y1="6" x2="21" y2="6" />
						<line x1="3" y1="12" x2="21" y2="12" />
						<line x1="3" y1="18" x2="21" y2="18" />
					</svg>
				</button>
			{/if}
			<div class="tags-title">Tags</div>
		</div>
		<div class="tags-header-controls">
			<button type="button" class="add-tag-btn" onclick={onCreate}>
				<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
					<line x1="12" y1="5" x2="12" y2="19" />
					<line x1="5" y1="12" x2="19" y2="12" />
				</svg>
				New Tag
			</button>
			<div class="segmented-control" role="group" aria-label="Filter by scope">
				<button
					type="button"
					class="segment"
					class:active={activeScope === 'document'}
					onclick={() => onScopeClick('document')}
				>
					Document
				</button>
				<button
					type="button"
					class="segment"
					class:active={activeScope === 'highlight'}
					onclick={() => onScopeClick('highlight')}
				>
					Highlight
				</button>
			</div>
			<div class="search-field-wrap">
				<svg
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="1.8"
					aria-hidden="true"
				>
					<circle cx="11" cy="11" r="8" />
					<line x1="21" y1="21" x2="16.65" y2="16.65" />
				</svg>
				<input
					type="text"
					class="search-input"
					placeholder="Find…"
					value={searchQuery}
					oninput={(event) => onSearch(event.currentTarget.value)}
				/>
			</div>
		</div>
	</div>
</div>

{#if bulkMode}
	<div class="bulk-action-bar">
		<button
			type="button"
			class="bulk-deselect"
			aria-label="Deselect all"
			onclick={onClearSelection}
		>
			<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<line x1="18" y1="6" x2="6" y2="18" />
				<line x1="6" y1="6" x2="18" y2="18" />
			</svg>
		</button>
		<span
			><span class="bulk-count">{selectedCount} tag{selectedCount !== 1 ? 's' : ''}</span> selected</span
		>
		<span class="bulk-spacer"></span>
		{#if selectedCount >= 2}
			<button type="button" class="bulk-btn" onclick={onBulkMerge}>
				<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
					<path d="M8 3H5a2 2 0 0 0-2 2v3" />
					<path d="M21 8V5a2 2 0 0 0-2-2h-3" />
					<path d="M3 16v3a2 2 0 0 0 2 2h3" />
					<path d="M16 21h3a2 2 0 0 0 2-2v-3" />
				</svg>
				Merge Selected
			</button>
		{/if}
		<button type="button" class="bulk-btn bulk-btn-destructive" onclick={onBulkDelete}>
			<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
				<polyline points="3 6 5 6 21 6" />
				<path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
			</svg>
			Delete
		</button>
	</div>
{/if}

<style>
	.tags-header {
		padding: 16px 20px 12px;
		background: var(--bg-content);
		border-bottom: 0.5px solid var(--border-primary);
	}
	.tags-header-top,
	.tags-header-controls,
	.bulk-action-bar {
		display: flex;
		align-items: center;
		gap: 12px;
	}
	.tags-header-top {
		justify-content: space-between;
		flex-wrap: wrap;
	}
	.tags-title {
		font-size: 28px;
		font-weight: 700;
		letter-spacing: -0.03em;
		line-height: 1.18;
		color: var(--text-primary);
	}
	button,
	input {
		font: inherit;
	}
	.add-tag-btn,
	.bulk-btn,
	.bulk-deselect,
	.segment {
		border: 1px solid var(--border-hairline);
		background: var(--bg-elevated);
		color: var(--text-primary);
		cursor: pointer;
	}
	.add-tag-btn,
	.bulk-btn {
		display: inline-flex;
		align-items: center;
		gap: 7px;
		border-radius: 980px;
		padding: 5px 14px;
		font-size: 13px;
		font-weight: 500;
		letter-spacing: -0.01em;
		min-height: 32px;
	}
	.add-tag-btn svg,
	.bulk-deselect svg,
	.search-field-wrap svg {
		width: 15px;
		height: 15px;
	}
	.segmented-control {
		display: inline-flex;
		border: 0;
		border-radius: 8px;
		padding: 2px;
		background: var(--seg-bg);
		gap: 1px;
	}
	.segment {
		border: 0;
		border-radius: 7px;
		background: transparent;
		padding: 5px 16px;
		font-size: 13px;
		font-weight: 500;
		letter-spacing: -0.01em;
		color: var(--text-secondary);
	}
	.segment.active {
		background: var(--seg-on);
		color: var(--text-primary);
		box-shadow:
			0 1px 3px rgba(0, 0, 0, 0.06),
			0 0 0 0.5px var(--border-hairline);
	}
	.search-field-wrap {
		display: flex;
		align-items: center;
		gap: 7px;
		border: 1px solid var(--border-hairline);
		border-radius: 8px;
		padding: 0 9px;
		color: var(--text-tertiary);
		background: var(--fill-hover);
	}
	.search-input {
		width: 160px;
		height: 30px;
		border: 0;
		outline: 0;
		background: transparent;
		color: var(--text-primary);
		font-size: 13px;
		letter-spacing: -0.01em;
	}
	.bulk-action-bar {
		padding: 8px 20px;
		background: var(--accent);
		border-bottom: 0;
		color: var(--text-on-color);
		font-size: 13px;
		font-weight: 500;
		letter-spacing: -0.01em;
	}
	.bulk-deselect {
		width: 24px;
		height: 24px;
		border-radius: 12px;
		background: color-mix(in oklab, var(--text-on-color) 20%, transparent);
		border: none;
		color: var(--text-on-color);
		display: inline-flex;
		align-items: center;
		justify-content: center;
	}
	.bulk-count {
		color: var(--text-primary);
		font-weight: 600;
	}
	.bulk-spacer {
		flex: 1;
	}
	.bulk-btn {
		padding: 5px 14px;
		border-radius: 6px;
		font-size: 12px;
		min-height: 28px;
		border-color: color-mix(in oklab, var(--text-on-color) 30%, transparent);
		background: color-mix(in oklab, var(--text-on-color) 15%, transparent);
		color: var(--text-on-color);
	}
	.bulk-btn-destructive {
		background: color-mix(in oklab, var(--destructive) 36%, transparent);
		border-color: color-mix(in oklab, var(--text-on-color) 20%, transparent);
		color: var(--text-on-color);
	}
	.tags-title-row {
		display: flex;
		align-items: center;
		gap: 8px;
		min-width: 0;
	}
	.menu-btn {
		display: none;
		width: 34px;
		height: 34px;
		align-items: center;
		justify-content: center;
		border-radius: var(--radius-sm);
		border: none;
		background: transparent;
		color: var(--text-secondary);
		cursor: pointer;
		flex-shrink: 0;
		padding: 0;
	}
	.menu-btn:hover {
		background: var(--fill-hover);
	}
	.menu-btn svg {
		width: 20px;
		height: 20px;
	}
	@media (max-width: 760px) {
		.tags-header-top,
		.tags-header-controls {
			align-items: stretch;
			flex-direction: column;
		}
		.search-input {
			width: 100%;
		}
	}
	@media (max-width: 599px) {
		.menu-btn {
			display: flex;
		}
		.tags-header {
			padding: 12px 16px 10px;
		}
		.tags-title {
			font-size: 22px;
		}
		.bulk-action-bar {
			padding: 8px 16px;
			flex-wrap: wrap;
		}
	}
</style>
