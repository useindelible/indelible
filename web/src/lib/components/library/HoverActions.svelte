<script lang="ts">
	import type { TriageModeDto } from '$lib/api/generated/types.gen';
	import { triageOptionsForMode, type TriageTab } from '$lib/stores/library.svelte';

	interface Props {
		currentTriage: string;
		triageMode?: TriageModeDto;
		onTriage: (state: TriageTab) => void;
		onMore: (e: MouseEvent) => void;
	}

	let { currentTriage, triageMode = 'focus', onTriage, onMore }: Props = $props();

	const triageButtons = $derived(
		triageOptionsForMode(triageMode).map((option) => ({
			state: option.value,
			label: option.label
		}))
	);
</script>

<div class="hover-actions">
	<div class="triage-pill">
		{#each triageButtons as btn (btn.state)}
			<button
				type="button"
				class="triage-btn"
				class:current={currentTriage === btn.state}
				aria-label="Move to {btn.label}"
				onclick={(e) => {
					e.stopPropagation();
					if (currentTriage !== btn.state) {
						onTriage(btn.state);
					}
				}}
				aria-pressed={currentTriage === btn.state}
			>
				{#if btn.state === 'inbox'}
					<svg viewBox="0 0 24 24" aria-hidden="true">
						<path d="M22 12h-6l-2 3H10l-2-3H2" />
						<path
							d="M5.45 5.11L2 12v6a2 2 0 002 2h16a2 2 0 002-2v-6l-3.45-6.89A2 2 0 0016.76 4H7.24a2 2 0 00-1.79 1.11z"
						/>
					</svg>
				{:else if btn.state === 'later'}
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
			</button>
		{/each}
	</div>

	<button
		type="button"
		class="more-btn"
		aria-label="More options"
		onclick={(e) => {
			e.stopPropagation();
			onMore(e);
		}}
	>
		<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
			<circle cx="12" cy="5" r="1" fill="currentColor" stroke="none" />
			<circle cx="12" cy="12" r="1" fill="currentColor" stroke="none" />
			<circle cx="12" cy="19" r="1" fill="currentColor" stroke="none" />
		</svg>
	</button>
</div>

<style>
	.hover-actions {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.triage-pill {
		display: flex;
		align-items: center;
		background: var(--bg-elevated);
		backdrop-filter: blur(20px) saturate(180%);
		-webkit-backdrop-filter: blur(20px) saturate(180%);
		border-radius: 8px;
		padding: 2px;
		box-shadow: var(--shadow-1);
	}

	.triage-btn {
		width: 30px;
		height: 28px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 6px;
		border: none;
		background: none;
		cursor: pointer;
		color: var(--text-primary);
		transition: background 0.12s ease;
		padding: 0;
	}

	.triage-btn.current {
		color: var(--text-quaternary);
		cursor: default;
	}

	.triage-btn:not(.current):hover {
		background: var(--fill-hover);
	}

	.triage-btn svg {
		width: 15px;
		height: 15px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.5;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.more-btn {
		width: 30px;
		height: 30px;
		border-radius: 15px;
		background: var(--bg-elevated);
		backdrop-filter: blur(20px) saturate(180%);
		-webkit-backdrop-filter: blur(20px) saturate(180%);
		box-shadow: var(--shadow-1);
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: pointer;
		border: none;
		color: var(--text-primary);
		padding: 0;
		transition: background 0.12s ease;
	}

	.more-btn:hover {
		background: var(--fill-hover);
	}

	.more-btn svg {
		width: 16px;
		height: 16px;
	}
</style>
