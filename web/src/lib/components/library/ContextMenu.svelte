<script lang="ts">
	import type { DocumentListEntry, TriageModeDto } from '$lib/api';
	import { t } from '$lib/i18n';
	import { triageOptionsForMode, type TriageTab } from '$lib/stores/library.svelte';

	interface Props {
		item: DocumentListEntry;
		x: number;
		y: number;
		onClose: () => void;
		triageMode?: TriageModeDto;
		onTriage: (state: TriageTab) => void;
		onDelete?: () => void;
		onAddTags?: () => void;
	}

	let {
		item,
		x,
		y,
		onClose,
		triageMode = 'focus',
		onTriage,
		onDelete,
		onAddTags
	}: Props = $props();

	let menuEl = $state<HTMLDivElement | undefined>(undefined);

	$effect(() => {
		function handleClickOutside(e: MouseEvent) {
			if (menuEl && !menuEl.contains(e.target as Node)) {
				onClose();
			}
		}

		function handleKeydown(e: KeyboardEvent) {
			if (e.key === 'Escape') {
				onClose();
			}
		}

		document.addEventListener('mousedown', handleClickOutside);
		document.addEventListener('keydown', handleKeydown);

		return () => {
			document.removeEventListener('mousedown', handleClickOutside);
			document.removeEventListener('keydown', handleKeydown);
		};
	});

	// Adjust position to keep menu within viewport
	const adjustedX = $derived(Math.min(x, window.innerWidth - 270));
	const adjustedY = $derived(Math.min(y, window.innerHeight - 300));
	const triageActions = $derived(
		triageOptionsForMode(triageMode)
			.filter((option) => item.triage_state !== option.value)
			.map((option) => ({
				state: option.value,
				label:
					option.value === 'archive'
						? $t('library_action_archive')
						: option.value === 'later'
							? $t('library_action_save_for_later')
							: $t('library_action_move_to', {
									values: { label: $t(option.labelKey) }
								})
			}))
	);
</script>

<div
	bind:this={menuEl}
	class="context-menu"
	style="left: {adjustedX}px; top: {adjustedY}px;"
	role="menu"
	aria-label={$t('library_action_item_actions')}
>
	<div class="menu-header">
		<span class="menu-title">{item.title}</span>
	</div>

	<div class="menu-section">
		{#each triageActions as action (action.state)}
			<button
				type="button"
				class="menu-item"
				role="menuitem"
				onclick={() => {
					onTriage(action.state);
					onClose();
				}}
			>
				{#if action.state === 'inbox'}
					<svg viewBox="0 0 24 24" aria-hidden="true">
						<path d="M22 12h-6l-2 3H10l-2-3H2" />
						<path
							d="M5.45 5.11L2 12v6a2 2 0 002 2h16a2 2 0 002-2v-6l-3.45-6.89A2 2 0 0016.76 4H7.24a2 2 0 00-1.79 1.11z"
						/>
					</svg>
				{:else if action.state === 'later'}
					<svg viewBox="0 0 24 24" aria-hidden="true">
						<circle cx="12" cy="12" r="10" />
						<polyline points="12 6 12 12 16 14" />
					</svg>
				{:else}
					<svg viewBox="0 0 24 24" aria-hidden="true">
						<rect x="2" y="3" width="20" height="5" rx="1" />
						<path d="M4 8v10a2 2 0 002 2h12a2 2 0 002-2V8" />
						<line x1="10" y1="12" x2="14" y2="12" />
					</svg>
				{/if}
				{action.label}
			</button>
		{/each}
	</div>

	<div class="menu-divider"></div>

	<div class="menu-section">
		{#if onAddTags}
			<button
				type="button"
				class="menu-item"
				role="menuitem"
				onclick={() => {
					onAddTags();
					onClose();
				}}
			>
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<path d="M20.59 13.41l-7.17 7.17a2 2 0 01-2.83 0L2 12V2h10l8.59 8.59a2 2 0 010 2.82z" />
					<line x1="7" y1="7" x2="7.01" y2="7" />
				</svg>
				{$t('library_action_add_tags')}
			</button>
		{/if}
		<button type="button" class="menu-item menu-item-stub" role="menuitem" disabled>
			<svg viewBox="0 0 24 24" aria-hidden="true">
				<path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z" />
			</svg>
			{$t('library_action_move_to_collection')}
		</button>
		<button type="button" class="menu-item menu-item-stub" role="menuitem" disabled>
			<svg viewBox="0 0 24 24" aria-hidden="true">
				<circle cx="18" cy="5" r="3" />
				<circle cx="6" cy="12" r="3" />
				<circle cx="18" cy="19" r="3" />
				<line x1="8.59" y1="13.51" x2="15.42" y2="17.49" />
				<line x1="15.41" y1="6.51" x2="8.59" y2="10.49" />
			</svg>
			{$t('library_action_share')}
		</button>
	</div>

	<div class="menu-divider"></div>

	<div class="menu-section">
		{#if item.url}
			<button
				type="button"
				class="menu-item"
				role="menuitem"
				onclick={() => {
					window.open(item.url ?? '', '_blank', 'noopener,noreferrer');
					onClose();
				}}
			>
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<path d="M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6" />
					<polyline points="15 3 21 3 21 9" />
					<line x1="10" y1="14" x2="21" y2="3" />
				</svg>
				{$t('library_action_open_original')}
			</button>
		{/if}
		{#if onDelete}
			<button
				type="button"
				class="menu-item menu-item-destructive"
				role="menuitem"
				onclick={() => {
					onDelete?.();
					onClose();
				}}
			>
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<polyline points="3 6 5 6 21 6" />
					<path d="M19 6l-1 14H6L5 6" />
					<path d="M10 11v6M14 11v6" />
					<path d="M9 6V4a1 1 0 011-1h4a1 1 0 011 1v2" />
				</svg>
				{$t('library_action_delete')}
			</button>
		{/if}
	</div>
</div>

<style>
	.context-menu {
		position: fixed;
		z-index: 1000;
		width: 260px;
		background: var(--bg-elevated);
		backdrop-filter: blur(40px) saturate(200%);
		-webkit-backdrop-filter: blur(40px) saturate(200%);
		border: 0.5px solid var(--border-primary);
		border-radius: var(--radius-xl);
		box-shadow: var(--shadow-3);
		overflow: hidden;
		animation: menu-open 0.15s ease-out;
	}

	@keyframes menu-open {
		from {
			opacity: 0;
			transform: scale(0.96) translateY(-4px);
		}
		to {
			opacity: 1;
			transform: scale(1) translateY(0);
		}
	}

	.menu-header {
		padding: 10px 14px 8px;
		border-bottom: 0.5px solid var(--border-primary);
	}

	.menu-title {
		font-family: var(--font-sans);
		font-size: 12px;
		font-weight: 500;
		color: var(--text-secondary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		display: block;
	}

	.menu-section {
		padding: 4px;
	}

	.menu-divider {
		height: 0.5px;
		background: var(--border-primary);
	}

	.menu-item {
		display: flex;
		align-items: center;
		gap: 10px;
		width: 100%;
		padding: 8px 10px;
		border-radius: var(--radius-sm);
		background: none;
		border: none;
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 400;
		letter-spacing: -0.01em;
		color: var(--text-primary);
		cursor: pointer;
		text-decoration: none;
		transition: background 0.1s ease;
		text-align: left;
	}

	.menu-item:hover:not(:disabled) {
		background: var(--fill-hover);
	}

	.menu-item-stub {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.menu-item-destructive {
		color: var(--destructive);
	}

	.menu-item svg {
		width: 16px;
		height: 16px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.5;
		stroke-linecap: round;
		stroke-linejoin: round;
		flex-shrink: 0;
	}
</style>
