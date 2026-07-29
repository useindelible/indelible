<script lang="ts">
	import SettingsHero from '$lib/components/settings/SettingsHero.svelte';
	import type { IntegrationConnectionDto } from '$lib/api';
	import type { ObsidianHeroState } from '../obsidian-model';

	interface Props {
		connection: IntegrationConnectionDto | undefined;
		heroState: ObsidianHeroState;
		statusLabel: string;
		lastSyncLabel: string;
		setupRunning: boolean;
		setupError: string | null;
		onSetup: () => void;
	}

	let {
		connection,
		heroState,
		statusLabel,
		lastSyncLabel,
		setupRunning,
		setupError,
		onSetup
	}: Props = $props();
</script>

<SettingsHero variant="obsidian">
	<div class="hero-watermark" aria-hidden="true">
		<svg viewBox="0 0 240 240">
			<polygon points="120,12 216,78 120,228 24,78" />
			<polyline points="24,78 120,108 216,78" />
			<line x1="120" y1="12" x2="120" y2="228" />
			<polyline points="63,45 120,108 177,45" />
		</svg>
	</div>

	<div class="hero-text">
		<div class="hero-eyebrow">
			<svg viewBox="0 0 24 24" aria-hidden="true">
				<path d="M9 7h-3a3 3 0 1 0 0 6h3" />
				<path d="M15 17h3a3 3 0 1 0 0-6h-3" />
				<path d="M9 12h6" />
			</svg>
			Integrations
			<span class="eyebrow-sep">/</span>
			<svg viewBox="0 0 24 24" aria-hidden="true">
				<polygon points="12,2 20,8.5 12,22 4,8.5" />
				<polyline points="4,8.5 12,11.5 20,8.5" />
			</svg>
			Obsidian
		</div>
		<h1 class="hero-headline">Obsidian Export</h1>
		<p class="hero-sub">
			Server-rendered Markdown, append-only highlights, and granular formatting controls straight
			into your vault.
		</p>
		<div class="hero-ops">
			<span class="hero-status-pill" data-status={heroState}>
				<span class="pulse"></span>{statusLabel}
			</span>
			{#if heroState !== 'disconnected'}
				<span class="ops-sep" aria-hidden="true"></span>
				<span>Synced <strong>{lastSyncLabel}</strong></span>
				{#if connection?.pending_jobs && connection.pending_jobs > 0}
					<span class="ops-sep" aria-hidden="true"></span>
					<span><strong>{connection.pending_jobs}</strong> queued</span>
				{/if}
			{:else}
				<span class="ops-sep" aria-hidden="true"></span>
				<button type="button" class="hero-cta" onclick={onSetup} disabled={setupRunning}>
					{setupRunning ? 'Setting up…' : 'Set up Obsidian export'}
				</button>
				{#if setupError}
					<span class="hero-cta-error">{setupError}</span>
				{/if}
			{/if}
		</div>
	</div>

	<div class="hero-gem" aria-hidden="true">
		<svg viewBox="0 0 240 280">
			<circle class="gem-glow" cx="120" cy="140" r="118" />
			<polygon points="120,30 210,108 120,250 30,108" />
			<polyline points="30,108 120,110 210,108" />
			<line x1="120" y1="30" x2="120" y2="250" />
		</svg>
	</div>
</SettingsHero>

<style>
	.hero-watermark {
		position: absolute;
		right: 200px;
		top: 50%;
		transform: translateY(-50%) rotate(15deg);
		width: 320px;
		height: 320px;
		pointer-events: none;
		opacity: 0.07;
		color: var(--obs-mark-from);
		z-index: 0;
	}
	.hero-watermark svg,
	.hero-gem svg {
		width: 100%;
		height: 100%;
		stroke: currentColor;
		fill: none;
		stroke-linecap: round;
		stroke-linejoin: round;
	}
	.hero-text {
		flex: 1 1 0;
		min-width: 0;
		position: relative;
		z-index: 1;
	}
	.hero-eyebrow {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		font-size: 11px;
		font-weight: 700;
		letter-spacing: 0;
		text-transform: uppercase;
		color: var(--hero-obsidian-eyebrow);
		margin-bottom: 10px;
	}
	.hero-eyebrow svg {
		width: 12px;
		height: 12px;
		stroke: currentColor;
		fill: none;
		stroke-width: 2.2;
	}
	.eyebrow-sep {
		color: var(--hero-obsidian-edge);
		font-weight: 400;
	}
	.hero-headline {
		font-size: clamp(26px, 3.2vw, 38px);
		font-weight: 600;
		letter-spacing: 0;
		line-height: 1.06;
		color: var(--hero-obsidian-headline);
		margin: 0 0 8px;
	}
	.hero-sub {
		font-size: 13.5px;
		color: var(--hero-obsidian-eyebrow);
		line-height: 1.52;
		max-width: 460px;
		margin: 0 0 16px;
	}
	.hero-ops {
		display: flex;
		align-items: center;
		flex-wrap: wrap;
		gap: 12px 16px;
		font-size: 12px;
		color: var(--hero-obsidian-eyebrow);
	}
	.hero-ops > * {
		display: inline-flex;
		align-items: center;
		gap: 6px;
	}
	.hero-ops strong {
		color: var(--hero-obsidian-headline);
		font-weight: 600;
	}
	.ops-sep {
		width: 1px;
		height: 11px;
		background: var(--hero-obsidian-edge);
	}
	.hero-status-pill {
		padding: 3px 11px;
		border-radius: 980px;
		font-size: 11.5px;
		font-weight: 600;
	}
	.pulse {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: currentColor;
	}
	.hero-status-pill[data-status='connected'] {
		background: var(--obs-status-active-bg);
		color: var(--obs-status-active-text);
	}
	.hero-status-pill[data-status='syncing'] {
		background: var(--obs-state-running-bg);
		color: var(--obs-state-running-text);
	}
	.hero-status-pill[data-status='error'] {
		background: var(--obs-state-error-bg);
		color: var(--obs-state-error-text);
	}
	.hero-status-pill[data-status='disconnected'] {
		background: var(--obs-state-idle-bg);
		color: var(--obs-state-idle-text);
	}
	.hero-cta {
		font: 600 12.5px/1 var(--font-sans);
		padding: 7px 14px;
		border-radius: 980px;
		border: 1px solid transparent;
		background: var(--obs-accent);
		color: var(--text-on-color);
		cursor: pointer;
	}
	.hero-cta:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}
	.hero-cta-error {
		color: var(--destructive);
		font-size: 12px;
	}
	.hero-gem {
		flex-shrink: 0;
		width: 220px;
		height: 240px;
		color: var(--obs-mark-from);
		position: relative;
		z-index: 1;
	}
	.hero-gem polygon {
		fill: color-mix(in oklab, var(--obs-mark-from) 36%, transparent);
	}
	.gem-glow {
		fill: color-mix(in oklab, var(--obs-mark-from) 14%, transparent);
		stroke: none;
	}
	/* The gem is decorative; below its 220px plus a readable text column,
	   drop it and let the watermark slide off the edge. */
	@container hero (max-width: 679px) {
		.hero-gem {
			display: none;
		}
		.hero-watermark {
			right: -40px;
		}
	}
</style>
