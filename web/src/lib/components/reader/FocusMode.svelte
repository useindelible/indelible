<script lang="ts">
	import { onDestroy } from 'svelte';
	import { beforeNavigate } from '$app/navigation';

	type FocusModeState = 'selecting' | 'active' | 'paused' | 'completed';

	interface Props {
		focusState: FocusModeState;
		startProgress: number;
		currentProgress: number;
		highlightsCreated: number;
		onStart: () => void;
		onPause: () => void;
		onResume: () => void;
		onComplete: () => void;
		onExit: () => void;
	}

	let {
		focusState,
		startProgress,
		currentProgress,
		highlightsCreated,
		onStart,
		onPause,
		onResume,
		onComplete,
		onExit
	}: Props = $props();

	const PRESETS = [
		{ minutes: 15, label: '15' },
		{ minutes: 25, label: '25' },
		{ minutes: 45, label: '45' },
		{ minutes: 60, label: '60' }
	];

	let totalMs = $state(0);
	let remainingMs = $state(0);
	let intervalId: ReturnType<typeof setInterval> | undefined;
	let showNavConfirm = $state(false);

	const elapsed = $derived(totalMs - remainingMs);
	const progressFraction = $derived(totalMs > 0 ? elapsed / totalMs : 0);
	const progressGained = $derived(Math.max(0, currentProgress - startProgress));

	const displayMinutes = $derived(Math.floor(remainingMs / 60000));
	const displaySeconds = $derived(Math.floor((remainingMs % 60000) / 1000));
	const timeDisplay = $derived(
		`${String(displayMinutes).padStart(2, '0')}:${String(displaySeconds).padStart(2, '0')}`
	);

	const elapsedMinutes = $derived(Math.floor(elapsed / 60000));
	const elapsedSeconds = $derived(Math.floor((elapsed % 60000) / 1000));
	const elapsedDisplay = $derived(
		`${String(elapsedMinutes).padStart(2, '0')}:${String(elapsedSeconds).padStart(2, '0')}`
	);

	const CIRCLE_R = 20;
	const CIRCLE_C = 2 * Math.PI * CIRCLE_R;
	const strokeDashoffset = $derived(CIRCLE_C * (1 - progressFraction));

	function startTimer(minutes: number) {
		totalMs = minutes * 60000;
		remainingMs = totalMs;
		onStart();
		startInterval();
	}

	function startInterval() {
		if (intervalId) clearInterval(intervalId);
		intervalId = setInterval(() => {
			remainingMs -= 1000;
			if (remainingMs <= 0) {
				remainingMs = 0;
				clearInterval(intervalId);
				intervalId = undefined;
				onComplete();
			}
		}, 1000);
	}

	function handlePause() {
		if (intervalId) clearInterval(intervalId);
		intervalId = undefined;
		onPause();
	}

	function handleResume() {
		startInterval();
		onResume();
	}

	function handleKeydown(e: KeyboardEvent) {
		const tag = (e.target as HTMLElement)?.tagName;
		if (tag === 'INPUT' || tag === 'TEXTAREA' || (e.target as HTMLElement)?.isContentEditable)
			return;

		if (e.key === ' ' && (focusState === 'active' || focusState === 'paused')) {
			e.preventDefault();
			if (focusState === 'active') {
				handlePause();
			} else {
				handleResume();
			}
		}
	}

	beforeNavigate(({ cancel }) => {
		if (focusState === 'active' || focusState === 'paused') {
			cancel();
			showNavConfirm = true;
		}
	});

	function confirmExit() {
		if (intervalId) clearInterval(intervalId);
		intervalId = undefined;
		showNavConfirm = false;
		onExit();
	}

	onDestroy(() => {
		if (intervalId) clearInterval(intervalId);
	});
</script>

<svelte:window onkeydown={handleKeydown} />

{#if focusState === 'selecting'}
	<div class="focus-selector">
		<span class="focus-label">Focus for</span>
		<div class="preset-row">
			{#each PRESETS as preset (preset.minutes)}
				<button type="button" class="preset-btn" onclick={() => startTimer(preset.minutes)}>
					{preset.label} min
				</button>
			{/each}
		</div>
		<button type="button" class="focus-cancel" onclick={onExit}> Cancel </button>
	</div>
{/if}

{#if focusState === 'active' || focusState === 'paused'}
	<div class="focus-timer-bar" aria-live="polite">
		<div class="timer-display" role="timer" aria-label="Focus timer">
			<svg viewBox="0 0 48 48" class="timer-ring">
				<circle
					cx="24"
					cy="24"
					r={CIRCLE_R}
					fill="none"
					stroke="var(--border-primary)"
					stroke-width="3"
				/>
				<circle
					cx="24"
					cy="24"
					r={CIRCLE_R}
					fill="none"
					stroke="var(--accent)"
					stroke-width="3"
					stroke-linecap="round"
					stroke-dasharray={CIRCLE_C}
					stroke-dashoffset={strokeDashoffset}
					transform="rotate(-90 24 24)"
				/>
			</svg>
			<span class="timer-text">{timeDisplay}</span>
		</div>

		{#if focusState === 'active'}
			<button type="button" class="focus-control-btn" onclick={handlePause} aria-label="Pause">
				<svg viewBox="0 0 24 24" fill="currentColor">
					<rect x="6" y="4" width="4" height="16" rx="1" />
					<rect x="14" y="4" width="4" height="16" rx="1" />
				</svg>
			</button>
		{:else}
			<button type="button" class="focus-control-btn" onclick={handleResume} aria-label="Resume">
				<svg viewBox="0 0 24 24" fill="currentColor">
					<polygon points="5,3 19,12 5,21" />
				</svg>
			</button>
		{/if}

		<button
			type="button"
			class="focus-control-btn exit-btn"
			onclick={onExit}
			aria-label="Exit focus mode"
		>
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.5"
				stroke-linecap="round"
			>
				<line x1="18" y1="6" x2="6" y2="18" />
				<line x1="6" y1="6" x2="18" y2="18" />
			</svg>
		</button>
	</div>
{/if}

{#if focusState === 'completed'}
	<div class="focus-completion-overlay">
		<div class="completion-card">
			<h3 class="completion-title">Session Complete</h3>

			<div class="completion-stats">
				<div class="stat">
					<span class="stat-value">{elapsedDisplay}</span>
					<span class="stat-label">Time Focused</span>
				</div>
				<div class="stat">
					<span class="stat-value">{progressGained}%</span>
					<span class="stat-label">Progress</span>
				</div>
				<div class="stat">
					<span class="stat-value">{highlightsCreated}</span>
					<span class="stat-label">Highlights</span>
				</div>
			</div>

			<button type="button" class="completion-done-btn" onclick={onExit}> Done </button>
		</div>
	</div>
{/if}

{#if showNavConfirm}
	<div class="focus-completion-overlay">
		<div class="completion-card">
			<h3 class="completion-title">Leave Focus Session?</h3>
			<p class="nav-confirm-text">Your focus session is still in progress.</p>
			<div class="nav-confirm-actions">
				<button
					type="button"
					class="nav-confirm-cancel"
					onclick={() => {
						showNavConfirm = false;
					}}
				>
					Stay
				</button>
				<button type="button" class="nav-confirm-leave" onclick={confirmExit}> Leave </button>
			</div>
		</div>
	</div>
{/if}

<style>
	.focus-selector {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 8px 20px;
		background: var(--bg-content);
		border-bottom: 0.5px solid var(--border-primary);
		flex-shrink: 0;
	}

	.focus-label {
		font-size: 13px;
		font-weight: 500;
		color: var(--text-secondary);
		font-family: var(--font-sans);
		white-space: nowrap;
	}

	.preset-row {
		display: flex;
		gap: 6px;
	}

	.preset-btn {
		padding: 5px 14px;
		border-radius: 980px;
		border: 1px solid var(--border-primary);
		background: transparent;
		font-size: 13px;
		font-weight: 500;
		color: var(--text-primary);
		cursor: pointer;
		font-family: var(--font-sans);
		transition: all 120ms ease;
	}

	.preset-btn:hover {
		background: var(--fill-selected);
		border-color: var(--accent);
		color: var(--accent);
	}

	.focus-cancel {
		font-size: 13px;
		font-weight: 500;
		color: var(--text-tertiary);
		background: none;
		border: none;
		cursor: pointer;
		font-family: var(--font-sans);
		padding: 4px 8px;
		margin-left: auto;
	}

	.focus-cancel:hover {
		color: var(--text-primary);
	}

	.focus-timer-bar {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 6px 20px;
		background: var(--bg-content);
		border-bottom: 0.5px solid var(--border-primary);
		flex-shrink: 0;
	}

	.timer-display {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.timer-ring {
		width: 32px;
		height: 32px;
	}

	.timer-text {
		font-size: 15px;
		font-weight: 600;
		color: var(--text-primary);
		font-family: var(--font-sans);
		font-variant-numeric: tabular-nums;
		letter-spacing: -0.01em;
	}

	.focus-control-btn {
		width: 28px;
		height: 28px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 7px;
		cursor: pointer;
		color: var(--text-primary);
		background: transparent;
		border: none;
		transition: background 120ms ease;
	}

	.focus-control-btn:hover {
		background: var(--fill-hover);
	}

	.focus-control-btn :global(svg) {
		width: 14px;
		height: 14px;
	}

	.exit-btn {
		margin-left: auto;
		color: var(--text-tertiary);
	}

	.exit-btn:hover {
		color: var(--destructive);
	}

	.focus-completion-overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.5);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 50;
	}

	.completion-card {
		background: var(--bg-elevated);
		border-radius: 16px;
		padding: 32px;
		width: 320px;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 24px;
		box-shadow: var(--shadow-3);
	}

	.completion-title {
		font-size: 20px;
		font-weight: 700;
		color: var(--text-primary);
		font-family: var(--font-sans);
		margin: 0;
		letter-spacing: -0.02em;
	}

	.completion-stats {
		display: flex;
		gap: 24px;
	}

	.stat {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 4px;
	}

	.stat-value {
		font-size: 22px;
		font-weight: 700;
		color: var(--accent);
		font-family: var(--font-sans);
		font-variant-numeric: tabular-nums;
	}

	.stat-label {
		font-size: 11px;
		font-weight: 500;
		color: var(--text-tertiary);
		font-family: var(--font-sans);
		text-transform: uppercase;
		letter-spacing: 0.06em;
	}

	.completion-done-btn {
		padding: 10px 32px;
		border-radius: 980px;
		background: var(--accent);
		color: #fff;
		font-size: 14px;
		font-weight: 600;
		border: none;
		cursor: pointer;
		font-family: var(--font-sans);
		transition: opacity 120ms ease;
	}

	.completion-done-btn:hover {
		opacity: 0.9;
	}

	.nav-confirm-text {
		font-size: 14px;
		color: var(--text-secondary);
		font-family: var(--font-sans);
		margin: 0;
		text-align: center;
	}

	.nav-confirm-actions {
		display: flex;
		gap: 12px;
	}

	.nav-confirm-cancel {
		padding: 10px 24px;
		border-radius: 980px;
		border: 1px solid var(--border-primary);
		background: transparent;
		color: var(--text-primary);
		font-size: 14px;
		font-weight: 500;
		cursor: pointer;
		font-family: var(--font-sans);
	}

	.nav-confirm-cancel:hover {
		background: var(--fill-hover);
	}

	.nav-confirm-leave {
		padding: 10px 24px;
		border-radius: 980px;
		background: var(--destructive);
		color: #fff;
		font-size: 14px;
		font-weight: 500;
		border: none;
		cursor: pointer;
		font-family: var(--font-sans);
	}

	.nav-confirm-leave:hover {
		opacity: 0.9;
	}
</style>
