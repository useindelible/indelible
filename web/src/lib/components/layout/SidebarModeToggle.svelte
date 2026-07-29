<script lang="ts">
	import type { NavMode } from '$lib/stores/sidebar.svelte';

	interface Props {
		mode: NavMode;
		onToggle: () => void;
	}

	let { mode, onToggle }: Props = $props();
</script>

<div class="mode-toggle" role="radiogroup" aria-label="Navigation mode">
	<button
		type="button"
		class="mode-btn"
		class:active={mode === 'content-type'}
		role="radio"
		aria-checked={mode === 'content-type'}
		aria-label="Browse by content type"
		onclick={() => {
			if (mode !== 'content-type') onToggle();
		}}
	>
		<svg
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="1.6"
			stroke-linecap="round"
			stroke-linejoin="round"
			aria-hidden="true"
		>
			<rect x="3" y="3" width="7" height="7" rx="1" />
			<rect x="14" y="3" width="7" height="7" rx="1" />
			<rect x="3" y="14" width="7" height="7" rx="1" />
			<rect x="14" y="14" width="7" height="7" rx="1" />
		</svg>
	</button>
	<button
		type="button"
		class="mode-btn"
		class:active={mode === 'collections'}
		role="radio"
		aria-checked={mode === 'collections'}
		aria-label="Browse by collections"
		onclick={() => {
			if (mode !== 'collections') onToggle();
		}}
	>
		<svg
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="1.6"
			stroke-linecap="round"
			stroke-linejoin="round"
			aria-hidden="true"
		>
			<path d="M3 7v10a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-6l-2-2H5a2 2 0 0 0-2 2z" />
		</svg>
	</button>
</div>

<style>
	.mode-toggle {
		display: flex;
		align-items: center;
		gap: 2px;
		background: var(--fill-secondary);
		border-radius: 7px;
		padding: 2px;
	}

	.mode-btn {
		width: 26px;
		height: 24px;
		border-radius: 5px;
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

	.mode-btn:hover {
		color: var(--text-secondary);
	}

	.mode-btn.active {
		background: var(--bg-primary);
		color: var(--text-primary);
		box-shadow: 0 1px 2px rgba(0, 0, 0, 0.06);
	}

	.mode-btn svg {
		width: 14px;
		height: 14px;
	}
</style>
