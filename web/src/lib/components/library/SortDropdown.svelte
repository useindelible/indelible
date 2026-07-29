<script lang="ts">
	import { getLibrary, type SortOrder } from '$lib/stores/library.svelte';

	const lib = getLibrary();

	let open = $state(false);
	let buttonEl = $state<HTMLButtonElement | undefined>(undefined);

	const options: { value: SortOrder; label: string }[] = [
		{ value: 'date_saved_desc', label: 'Date saved' },
		{ value: 'date_published_desc', label: 'Date published' },
		{ value: 'title_asc', label: 'Title A–Z' },
		{ value: 'title_desc', label: 'Title Z–A' },
		{ value: 'reading_progress', label: 'Reading progress' },
		{ value: 'reading_time', label: 'Reading time' }
	];

	const currentLabel = $derived(
		options.find((o) => o.value === lib.sortOrder)?.label ?? 'Date saved'
	);

	$effect(() => {
		if (!open) return;

		function handleClickOutside(e: MouseEvent) {
			if (buttonEl && !buttonEl.closest('.sort-dropdown-wrapper')?.contains(e.target as Node)) {
				open = false;
			}
		}

		function handleKeydown(e: KeyboardEvent) {
			if (e.key === 'Escape') open = false;
		}

		document.addEventListener('mousedown', handleClickOutside);
		document.addEventListener('keydown', handleKeydown);

		return () => {
			document.removeEventListener('mousedown', handleClickOutside);
			document.removeEventListener('keydown', handleKeydown);
		};
	});
</script>

<div class="sort-dropdown-wrapper">
	<button
		bind:this={buttonEl}
		type="button"
		class="sort-trigger"
		onclick={() => (open = !open)}
		aria-haspopup="listbox"
		aria-expanded={open}
	>
		<span>{currentLabel}</span>
		<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
			<polyline points="6 9 12 15 18 9" />
		</svg>
	</button>

	{#if open}
		<div class="sort-popover" role="listbox" aria-label="Sort order">
			{#each options as opt (opt.value)}
				<button
					type="button"
					class="sort-option"
					role="option"
					aria-selected={lib.sortOrder === opt.value}
					class:selected={lib.sortOrder === opt.value}
					onclick={() => {
						lib.setSortOrder(opt.value);
						open = false;
					}}
				>
					{opt.label}
					{#if lib.sortOrder === opt.value}
						<svg
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2.5"
							aria-hidden="true"
						>
							<polyline points="20 6 9 17 4 12" />
						</svg>
					{/if}
				</button>
			{/each}
		</div>
	{/if}
</div>

<style>
	.sort-dropdown-wrapper {
		position: relative;
	}

	.sort-trigger {
		display: flex;
		align-items: center;
		gap: 4px;
		background: none;
		border: none;
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 400;
		color: var(--text-secondary);
		cursor: pointer;
		padding: 4px 6px;
		border-radius: var(--radius-sm);
		transition: color 0.12s ease;
	}

	.sort-trigger:hover {
		color: var(--text-primary);
	}

	.sort-trigger svg {
		width: 14px;
		height: 14px;
	}

	.sort-popover {
		position: absolute;
		right: 0;
		top: calc(100% + 6px);
		z-index: 100;
		width: 200px;
		background: var(--bg-elevated);
		backdrop-filter: blur(40px) saturate(200%);
		-webkit-backdrop-filter: blur(40px) saturate(200%);
		border: 0.5px solid var(--border-primary);
		border-radius: var(--radius-xl);
		box-shadow: var(--shadow-3);
		padding: 4px;
		animation: popover-open 0.15s ease-out;
	}

	@keyframes popover-open {
		from {
			opacity: 0;
			transform: scale(0.96) translateY(-4px);
		}
		to {
			opacity: 1;
			transform: scale(1) translateY(0);
		}
	}

	.sort-option {
		display: flex;
		align-items: center;
		justify-content: space-between;
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
		text-align: left;
		transition: background 0.1s ease;
	}

	.sort-option:hover {
		background: var(--fill-hover);
	}

	.sort-option.selected {
		font-weight: 500;
	}

	.sort-option svg {
		width: 14px;
		height: 14px;
		color: var(--accent);
		flex-shrink: 0;
	}
</style>
