<script lang="ts">
	import type { VoicePersonaResponse } from '$lib/api/generated/types.gen';
	import { t } from '$lib/i18n';

	interface Props {
		playing: boolean;
		loading: boolean;
		currentTime: number;
		duration: number;
		speed: number;
		personas: VoicePersonaResponse[];
		selectedPersonaId: string | null;
		onPlay: () => void;
		onPause: () => void;
		onStop: () => void;
		onSkipBack: () => void;
		onSkipForward: () => void;
		onSeek: (time: number) => void;
		onSpeedChange: (speed: number) => void;
		onPersonaChange: (personaId: string) => void;
	}

	const SPEEDS = [0.5, 0.75, 1, 1.25, 1.5, 2, 2.5, 3];

	let {
		playing,
		loading,
		currentTime,
		duration,
		speed,
		personas,
		selectedPersonaId,
		onPlay,
		onPause,
		onStop,
		onSkipBack,
		onSkipForward,
		onSeek,
		onSpeedChange,
		onPersonaChange
	}: Props = $props();

	let showSpeedPopover = $state(false);
	let showVoicePopover = $state(false);

	let speedWrapperEl = $state<HTMLDivElement | undefined>(undefined);
	let voiceWrapperEl = $state<HTMLDivElement | undefined>(undefined);
	let progressTrackEl = $state<HTMLDivElement | undefined>(undefined);

	const selectedPersona = $derived(personas.find((p) => p.id === selectedPersonaId) ?? personas[0]);
	const builtInPersonas = $derived(personas.filter((p) => p.is_builtin));
	const customPersonas = $derived(personas.filter((p) => !p.is_builtin));
	const speedLabel = $derived(`${speed}x`);
	const progressRatio = $derived(
		duration > 0 ? Math.min(1, Math.max(0, currentTime / duration)) : 0
	);
	const progressPercent = $derived(`${(progressRatio * 100).toFixed(2)}%`);
	const timeLabel = $derived(`${formatTime(currentTime)} / ${formatTime(duration)}`);

	function formatTime(seconds: number): string {
		if (!Number.isFinite(seconds) || seconds < 0) return '0:00';
		const total = Math.floor(seconds);
		const m = Math.floor(total / 60);
		const s = total % 60;
		return `${m}:${s.toString().padStart(2, '0')}`;
	}

	function handleSeekClick(event: MouseEvent) {
		if (!progressTrackEl || duration <= 0) return;
		const rect = progressTrackEl.getBoundingClientRect();
		const ratio = Math.min(1, Math.max(0, (event.clientX - rect.left) / rect.width));
		onSeek(ratio * duration);
	}

	$effect(() => {
		if (!showSpeedPopover && !showVoicePopover) return;
		function handleClickOutside(e: MouseEvent) {
			const target = e.target as Node;
			if (showSpeedPopover && speedWrapperEl && !speedWrapperEl.contains(target)) {
				showSpeedPopover = false;
			}
			if (showVoicePopover && voiceWrapperEl && !voiceWrapperEl.contains(target)) {
				showVoicePopover = false;
			}
		}
		document.addEventListener('click', handleClickOutside, true);
		return () => document.removeEventListener('click', handleClickOutside, true);
	});
</script>

<div class="tts-strip" role="toolbar" aria-label={$t('reader_tts_controls')}>
	<div class="tts-strip-left">
		<button
			type="button"
			class="tts-btn"
			aria-label={$t('reader_tts_skip_back')}
			disabled={loading}
			onclick={onSkipBack}
		>
			<svg viewBox="0 0 24 24" fill="currentColor" stroke="none" aria-hidden="true">
				<path
					d="M12 5V1L7 6l5 5V7c3.31 0 6 2.69 6 6s-2.69 6-6 6-6-2.69-6-6H4c0 4.42 3.58 8 8 8s8-3.58 8-8-3.58-8-8-8z"
				/>
				<text
					x="12"
					y="16.5"
					text-anchor="middle"
					font-size="7.5"
					font-weight="700"
					letter-spacing="-0.3"
					font-family="-apple-system, BlinkMacSystemFont, system-ui, sans-serif">15</text
				>
			</svg>
		</button>

		<button
			type="button"
			class="tts-btn primary"
			aria-label={$t(playing ? 'reader_pause' : 'reader_play')}
			disabled={loading}
			onclick={playing ? onPause : onPlay}
		>
			{#if loading}
				<svg viewBox="0 0 24 24" fill="currentColor" stroke="none" class="spin" aria-hidden="true">
					<path
						d="M12 2a10 10 0 110 20A10 10 0 0112 2zm0 2a8 8 0 100 16A8 8 0 0012 4z"
						opacity="0.3"
					/>
					<path d="M12 2a10 10 0 0110 10h-2A8 8 0 0012 4V2z" />
				</svg>
			{:else if playing}
				<svg viewBox="0 0 24 24" fill="currentColor" stroke="none" aria-hidden="true">
					<rect x="6" y="5" width="4" height="14" rx="1" />
					<rect x="14" y="5" width="4" height="14" rx="1" />
				</svg>
			{:else}
				<svg viewBox="0 0 24 24" fill="currentColor" stroke="none" aria-hidden="true">
					<polygon points="5,3 19,12 5,21" />
				</svg>
			{/if}
		</button>

		<button
			type="button"
			class="tts-btn"
			aria-label={$t('reader_tts_skip_forward')}
			disabled={loading}
			onclick={onSkipForward}
		>
			<svg viewBox="0 0 24 24" fill="currentColor" stroke="none" aria-hidden="true">
				<path
					d="M12 5V1l5 5-5 5V7c-3.31 0-6 2.69-6 6s2.69 6 6 6 6-2.69 6-6h2c0 4.42-3.58 8-8 8s-8-3.58-8-8 3.58-8 8-8z"
				/>
				<text
					x="12"
					y="16.5"
					text-anchor="middle"
					font-size="7.5"
					font-weight="700"
					letter-spacing="-0.3"
					font-family="-apple-system, BlinkMacSystemFont, system-ui, sans-serif">15</text
				>
			</svg>
		</button>

		{#if playing}
			<span class="tts-wave-group" aria-hidden="true">
				<span class="tts-wave-bar"></span>
				<span class="tts-wave-bar"></span>
				<span class="tts-wave-bar"></span>
				<span class="tts-wave-bar"></span>
			</span>
		{/if}
	</div>

	<div class="tts-strip-center">
		<div
			class="tts-progress-track"
			bind:this={progressTrackEl}
			role="slider"
			tabindex="0"
			aria-label={$t('reader_tts_playback_position')}
			aria-valuemin={0}
			aria-valuemax={duration > 0 ? duration : 0}
			aria-valuenow={currentTime}
			onclick={handleSeekClick}
			onkeydown={(e) => {
				if (e.key === 'ArrowLeft') {
					onSeek(Math.max(0, currentTime - 5));
				} else if (e.key === 'ArrowRight') {
					onSeek(Math.min(duration, currentTime + 5));
				}
			}}
		>
			<div class="tts-progress-fill" style:width={progressPercent}></div>
			<div class="tts-progress-thumb" style:left={progressPercent}></div>
		</div>
		<div class="tts-time">{timeLabel}</div>
	</div>

	<div class="tts-strip-right">
		<div class="tts-chip-wrapper speed-wrapper" bind:this={speedWrapperEl}>
			<button
				type="button"
				class="tts-chip"
				class:active={showSpeedPopover}
				onclick={() => {
					showSpeedPopover = !showSpeedPopover;
					showVoicePopover = false;
				}}
				aria-haspopup="listbox"
				aria-expanded={showSpeedPopover}
				aria-label={$t('reader_tts_playback_speed_value', { values: { speed: speedLabel } })}
			>
				{speedLabel}
				<svg
					class="chev"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
					aria-hidden="true"
				>
					<polyline points="6 9 12 15 18 9" />
				</svg>
			</button>

			{#if showSpeedPopover}
				<div class="tts-popover speed" role="listbox" aria-label={$t('reader_tts_playback_speed')}>
					{#each SPEEDS as s (s)}
						<button
							type="button"
							role="option"
							aria-selected={speed === s}
							class="tts-speed-row"
							class:selected={speed === s}
							onclick={() => {
								onSpeedChange(s);
								showSpeedPopover = false;
							}}
						>
							{s}x
							{#if speed === s}
								<svg
									class="tts-speed-check"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="2.5"
									stroke-linecap="round"
									stroke-linejoin="round"
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

		{#if personas.length > 0}
			<div class="tts-chip-wrapper" bind:this={voiceWrapperEl}>
				<button
					type="button"
					class="tts-chip"
					class:active={showVoicePopover}
					onclick={() => {
						showVoicePopover = !showVoicePopover;
						showSpeedPopover = false;
					}}
					aria-haspopup="listbox"
					aria-expanded={showVoicePopover}
					aria-label={$t('reader_tts_voice_value', {
						values: { voice: selectedPersona?.display_name ?? $t('reader_tts_default_voice') }
					})}
				>
					<span class="tts-voice-dot" class:custom={selectedPersona && !selectedPersona.is_builtin}
					></span>
					{selectedPersona?.display_name ?? $t('reader_tts_default_voice')}
					<svg
						class="chev"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						stroke-linecap="round"
						stroke-linejoin="round"
						aria-hidden="true"
					>
						<polyline points="6 9 12 15 18 9" />
					</svg>
				</button>

				{#if showVoicePopover}
					<div class="tts-popover voice" role="listbox" aria-label={$t('reader_tts_voices')}>
						{#if builtInPersonas.length > 0}
							<div class="tts-persona-group-label">{$t('reader_tts_builtin')}</div>
							{#each builtInPersonas as persona (persona.id)}
								<button
									type="button"
									role="option"
									aria-selected={selectedPersonaId === persona.id}
									class="tts-persona-row"
									class:selected={selectedPersonaId === persona.id}
									onclick={() => {
										onPersonaChange(persona.id);
										showVoicePopover = false;
									}}
								>
									<span class="tts-persona-radio" class:selected={selectedPersonaId === persona.id}
									></span>
									<span class="tts-persona-name">{persona.display_name}</span>
									<span class="tts-persona-provider">{persona.provider}</span>
								</button>
							{/each}
						{/if}
						{#if customPersonas.length > 0}
							<div class="tts-persona-group-label">{$t('reader_tts_custom')}</div>
							{#each customPersonas as persona (persona.id)}
								<button
									type="button"
									role="option"
									aria-selected={selectedPersonaId === persona.id}
									class="tts-persona-row"
									class:selected={selectedPersonaId === persona.id}
									onclick={() => {
										onPersonaChange(persona.id);
										showVoicePopover = false;
									}}
								>
									<span class="tts-persona-radio" class:selected={selectedPersonaId === persona.id}
									></span>
									<span class="tts-persona-name">{persona.display_name}</span>
									<span class="tts-persona-provider">{persona.provider}</span>
								</button>
							{/each}
						{/if}
					</div>
				{/if}
			</div>
		{/if}

		<button type="button" class="tts-btn stop" aria-label={$t('reader_tts_stop')} onclick={onStop}>
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
				stroke-linecap="round"
				stroke-linejoin="round"
				aria-hidden="true"
			>
				<line x1="18" y1="6" x2="6" y2="18" />
				<line x1="6" y1="6" x2="18" y2="18" />
			</svg>
		</button>
	</div>
</div>

<style>
	.tts-strip {
		height: 64px;
		min-height: 64px;
		flex-shrink: 0;
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 0 20px;
		background: var(--tts-strip-bg);
		border-bottom: 0.5px solid var(--tts-strip-border);
		position: relative;
		z-index: 3;
	}

	.tts-strip-left {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.tts-strip-center {
		flex: 1;
		display: flex;
		align-items: center;
		gap: 12px;
		min-width: 0;
	}

	.tts-strip-right {
		display: flex;
		align-items: center;
		gap: 6px;
		position: relative;
	}

	.tts-btn {
		width: 36px;
		height: 36px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 8px;
		border: none;
		background: transparent;
		color: var(--text-primary);
		cursor: pointer;
		transition:
			background 120ms ease,
			transform 120ms ease;
		padding: 0;
		font-family: var(--font-sans);
	}

	.tts-btn:hover:not(:disabled) {
		background: var(--fill-hover);
	}

	.tts-btn:active:not(:disabled) {
		transform: scale(0.96);
	}

	.tts-btn:disabled {
		opacity: 0.3;
		cursor: default;
	}

	.tts-btn :global(svg) {
		width: 16px;
		height: 16px;
	}

	.tts-btn.primary {
		width: 40px;
		height: 40px;
		background: var(--accent);
		color: var(--text-on-color);
		box-shadow: var(--tts-primary-shadow);
	}

	.tts-btn.primary:hover:not(:disabled) {
		background: var(--accent-hover);
	}

	.tts-btn.primary :global(svg) {
		width: 17px;
		height: 17px;
	}

	.tts-btn.stop {
		color: var(--text-secondary);
	}

	.tts-btn.stop:hover {
		color: var(--destructive);
	}

	.tts-progress-track {
		flex: 1;
		height: 4px;
		background: var(--tts-quota-bar-bg);
		border-radius: 2px;
		position: relative;
		min-width: 120px;
		cursor: pointer;
		outline: none;
	}

	.tts-progress-track:focus-visible {
		box-shadow: 0 0 0 2px var(--fill-selected);
	}

	.tts-progress-fill {
		position: absolute;
		left: 0;
		top: 0;
		bottom: 0;
		border-radius: 2px;
		background: var(--accent);
	}

	.tts-progress-thumb {
		position: absolute;
		top: 50%;
		transform: translate(-50%, -50%);
		width: 12px;
		height: 12px;
		border-radius: 50%;
		background: var(--accent);
		box-shadow: var(--shadow-1);
		pointer-events: none;
	}

	.tts-time {
		font-variant-numeric: tabular-nums;
		color: var(--text-tertiary);
		font-size: 11px;
		font-weight: 500;
		letter-spacing: 0;
		font-family: var(--font-sans);
		white-space: nowrap;
	}

	.tts-wave-group {
		display: inline-flex;
		align-items: center;
		gap: 2px;
		height: 16px;
		margin-left: 2px;
	}

	.tts-wave-bar {
		width: 2px;
		background: var(--accent);
		border-radius: 1px;
		transform-origin: center;
		animation: ttsWave 900ms ease-in-out infinite;
	}

	.tts-wave-bar:nth-child(1) {
		height: 6px;
		animation-delay: 0ms;
	}
	.tts-wave-bar:nth-child(2) {
		height: 12px;
		animation-delay: 120ms;
	}
	.tts-wave-bar:nth-child(3) {
		height: 16px;
		animation-delay: 240ms;
	}
	.tts-wave-bar:nth-child(4) {
		height: 10px;
		animation-delay: 360ms;
	}

	@keyframes ttsWave {
		0%,
		100% {
			transform: scaleY(0.35);
			opacity: 0.6;
		}
		50% {
			transform: scaleY(1);
			opacity: 1;
		}
	}

	.tts-chip-wrapper {
		position: relative;
	}

	.tts-chip {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 6px 10px;
		border-radius: 8px;
		background: var(--bg-elevated);
		border: 0.5px solid var(--border-primary);
		box-shadow: var(--shadow-1);
		cursor: pointer;
		font-size: 12px;
		font-weight: 500;
		color: var(--text-primary);
		letter-spacing: -0.005em;
		transition:
			background 120ms ease,
			border-color 120ms ease;
		white-space: nowrap;
		font-family: var(--font-sans);
	}

	.tts-chip:hover {
		background: var(--fill-hover);
		border-color: var(--border-secondary);
	}

	.tts-chip.active {
		border-color: var(--accent);
		box-shadow: 0 0 0 2px var(--fill-selected);
	}

	.tts-chip .chev {
		width: 10px;
		height: 10px;
		color: var(--text-tertiary);
		flex-shrink: 0;
	}

	.tts-voice-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--accent);
		flex-shrink: 0;
	}

	.tts-voice-dot.custom {
		background: var(--success);
	}

	.tts-popover {
		position: absolute;
		top: calc(100% + 6px);
		right: 0;
		background: var(--bg-elevated);
		border-radius: 10px;
		box-shadow: var(--tts-popover-shadow);
		border: 0.5px solid var(--border-primary);
		z-index: 30;
		overflow: hidden;
	}

	.tts-popover.speed {
		width: 110px;
		padding: 4px;
	}

	.tts-popover.voice {
		width: 260px;
		padding: 4px;
	}

	.tts-speed-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 7px 10px;
		border-radius: 6px;
		cursor: pointer;
		font-size: 13px;
		font-weight: 500;
		color: var(--text-primary);
		letter-spacing: -0.01em;
		font-variant-numeric: tabular-nums;
		transition: background 120ms ease;
		font-family: var(--font-sans);
		width: 100%;
		background: transparent;
		border: none;
		text-align: left;
	}

	.tts-speed-row:hover {
		background: var(--fill-hover);
	}

	.tts-speed-row.selected {
		background: var(--fill-selected);
		color: var(--accent);
	}

	.tts-speed-check {
		width: 12px;
		height: 12px;
		color: var(--accent);
	}

	.tts-persona-group-label {
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		color: var(--text-tertiary);
		padding: 8px 10px 4px;
		font-family: var(--font-sans);
	}

	.tts-persona-row {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 7px 10px;
		border-radius: 6px;
		cursor: pointer;
		width: 100%;
		background: transparent;
		border: none;
		text-align: left;
		transition: background 120ms ease;
	}

	.tts-persona-row:hover {
		background: var(--fill-hover);
	}

	.tts-persona-row.selected {
		background: var(--fill-selected);
	}

	.tts-persona-radio {
		width: 14px;
		height: 14px;
		border-radius: 50%;
		border: 1.5px solid var(--text-tertiary);
		flex-shrink: 0;
		position: relative;
		transition: border-color 120ms ease;
	}

	.tts-persona-radio.selected {
		border-color: var(--accent);
	}

	.tts-persona-radio.selected::after {
		content: '';
		position: absolute;
		inset: 2px;
		border-radius: 50%;
		background: var(--accent);
	}

	.tts-persona-name {
		font-size: 13px;
		font-weight: 500;
		color: var(--text-primary);
		letter-spacing: -0.01em;
		font-family: var(--font-sans);
		flex: 1;
		text-align: left;
	}

	.tts-persona-provider {
		font-size: 11px;
		color: var(--text-tertiary);
		font-family: var(--font-sans);
		letter-spacing: -0.005em;
	}

	.spin {
		animation: spin 800ms linear infinite;
	}

	@keyframes spin {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}

	/* Mobile keeps the transport, progress, voice, and stop; the decorative wave
	   bars and the speed chip yield — 390px can't fit them all, and speed is the
	   least-reached control. The chosen speed still applies; changing it is
	   desktop/tablet-only until a mobile TTS sheet exists. */
	@media (max-width: 599px) {
		.tts-strip {
			height: 54px;
			min-height: 54px;
			gap: 8px;
			padding: 0 12px;
		}

		.tts-strip-left {
			gap: 2px;
		}

		.tts-wave-group,
		.speed-wrapper {
			display: none;
		}

		.tts-time {
			font-size: 11px;
		}
	}
</style>
