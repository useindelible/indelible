<script lang="ts">
	import SettingsHero from '$lib/components/settings/SettingsHero.svelte';
	import type { FeedStats } from '../feed-model';

	interface Props {
		stats: FeedStats;
		onAddFeed: () => void;
		onImportOpml: () => void;
	}

	let { stats, onAddFeed, onImportOpml }: Props = $props();
</script>

<SettingsHero variant="feed">
	<div class="hero-text">
		<div class="hero-eyebrow">
			<span class="live-dot"></span>
			<span>
				{stats.active > 0
					? `Polling now · ${stats.active} source${stats.active === 1 ? '' : 's'}`
					: 'No active sources'}
			</span>
		</div>
		<h1 class="hero-headline">Your feeds, fresh every few minutes.</h1>
		<p class="hero-sub">
			RSS, Atom, and JSON feeds, polled on a schedule and routed straight to your library or your
			reading queue.
		</p>
		<div class="hero-cta">
			<button type="button" class="hero-btn primary" onclick={onAddFeed}>
				<svg viewBox="0 0 24 24"><path d="M12 5v14M5 12h14" /></svg>
				Add feed
			</button>
			<button type="button" class="hero-btn ghost" onclick={onImportOpml}>
				<svg viewBox="0 0 24 24">
					<path d="M12 3v12" />
					<path d="M7 8l5-5 5 5" />
					<path d="M5 21h14" />
				</svg>
				Import OPML
			</button>
		</div>
	</div>

	<div class="stats-ribbon" role="group" aria-label="Feed statistics">
		<div class="stat-cell">
			<div class="stat-num">{stats.active}</div>
			<div class="stat-label">Active</div>
			<div class="stat-sub">Polled on schedule</div>
		</div>
		<div class="stat-cell paused">
			<div class="stat-num">{stats.paused}</div>
			<div class="stat-label">Paused</div>
			<div class="stat-sub">Won't auto-poll</div>
		</div>
		<div class="stat-cell error">
			<div class="stat-num">{stats.error}</div>
			<div class="stat-label">Error</div>
			<div class="stat-sub">Needs attention</div>
		</div>
		<div class="stat-cell">
			<div class="stat-num">{stats.total}</div>
			<div class="stat-label">Total</div>
			{#if stats.active > 0}
				<div class="live-bar"><span class="live-dot"></span>Polling now</div>
			{:else}
				<div class="stat-sub">All sources</div>
			{/if}
		</div>
	</div>
</SettingsHero>

<style>
	:global(.hero[data-variant='feed']) {
		padding: 40px 56px 36px;
	}

	/* The trailing combinator must live inside :global() — hero-inner carries
	   SettingsHero's scope hash, so a page-scoped `> div` never matches.
	   Reset align-items: SettingsHero's base centers it, but this hero stacks
	   text and the stats ribbon left-aligned. */
	:global(.hero[data-variant='feed'] > div) {
		display: flex;
		flex-direction: column;
		align-items: stretch;
		gap: 22px;
		max-width: 1080px;
		width: 100%;
	}

	.hero-text {
		display: flex;
		flex-direction: column;
	}

	.hero-eyebrow {
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.14em;
		text-transform: uppercase;
		color: var(--feed-eyebrow-color);
		margin-bottom: 10px;
		display: inline-flex;
		align-items: center;
		gap: 8px;
	}

	.live-dot {
		width: 7px;
		height: 7px;
		border-radius: 50%;
		background: var(--feed-amber);
		box-shadow: 0 0 0 3px var(--feed-amber-soft);
		animation: live-pulse 2.2s ease-out infinite;
	}

	@keyframes live-pulse {
		0% {
			box-shadow: 0 0 0 0 var(--feed-pulse-color);
		}
		70% {
			box-shadow: 0 0 0 8px var(--feed-pulse-fade);
		}
		100% {
			box-shadow: 0 0 0 0 var(--feed-pulse-fade);
		}
	}

	.hero-headline {
		font-size: 30px;
		font-weight: 700;
		letter-spacing: -0.034em;
		color: var(--text-primary);
		line-height: 1.08;
		margin: 0 0 8px;
		font-family: var(--font-sans);
	}

	.hero-sub {
		font-size: 14.5px;
		color: var(--feed-eyebrow-color);
		line-height: 1.45;
		max-width: 540px;
		letter-spacing: -0.005em;
		margin: 0 0 18px;
	}

	.hero-cta {
		display: inline-flex;
		align-items: center;
		gap: 10px;
	}

	.hero-btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 9px 16px;
		border-radius: 980px;
		font-family: var(--font-sans);
		font-size: 13.5px;
		font-weight: 600;
		letter-spacing: -0.01em;
		border: none;
		cursor: pointer;
		white-space: nowrap;
		transition:
			transform 140ms,
			box-shadow 140ms;
	}

	.hero-btn svg {
		width: 13px;
		height: 13px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.9;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.hero-btn.primary {
		background: var(--feed-amber);
		color: var(--text-on-color);
		box-shadow: var(--feed-amber-shadow);
	}

	.hero-btn.primary:hover,
	.hero-btn.ghost:hover {
		transform: translateY(-1px);
	}

	.hero-btn.primary:hover {
		box-shadow: var(--feed-amber-shadow-hover);
	}

	.hero-btn.ghost {
		background: var(--feed-metric-card-bg);
		color: var(--feed-metric-num-color);
		box-shadow: 0 0 0 0.5px var(--feed-metric-card-border);
		backdrop-filter: blur(20px);
		-webkit-backdrop-filter: blur(20px);
	}

	.stats-ribbon {
		display: flex;
		align-items: stretch;
		background: var(--feed-metric-card-bg);
		border-radius: 18px;
		box-shadow: var(--feed-metric-card-shadow);
		backdrop-filter: blur(20px);
		-webkit-backdrop-filter: blur(20px);
		overflow: hidden;
		width: 100%;
		max-width: 720px;
	}

	.stat-cell {
		flex: 1;
		padding: 16px 20px;
		display: flex;
		flex-direction: column;
		gap: 4px;
		position: relative;
	}

	.stat-cell + .stat-cell::before {
		content: '';
		position: absolute;
		left: 0;
		top: 14px;
		bottom: 14px;
		width: 1px;
		background: var(--feed-metric-card-border);
	}

	.stat-num {
		font-size: 24px;
		font-weight: 700;
		letter-spacing: -0.04em;
		color: var(--feed-metric-num-color);
		line-height: 1;
		font-variant-numeric: tabular-nums;
	}

	.stat-cell.error .stat-num {
		color: var(--feed-status-error-text);
	}

	.stat-cell.paused .stat-num {
		color: var(--text-secondary);
	}

	.stat-label {
		font-size: 10.5px;
		font-weight: 600;
		letter-spacing: 0.12em;
		text-transform: uppercase;
		color: var(--feed-metric-label-color);
	}

	.stat-sub,
	.live-bar {
		font-size: 11.5px;
		color: var(--feed-metric-label-color);
		letter-spacing: -0.005em;
	}

	.live-bar {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		font-weight: 500;
	}

	.live-bar .live-dot {
		width: 6px;
		height: 6px;
	}

	/* Four stat cells need ~440px; tighten first, then fold to 2×2 with the
	   side dividers dropped where cells no longer sit beside each other. */
	@container hero (max-width: 639px) {
		.stat-cell {
			padding: 12px 14px;
		}

		.stat-num {
			font-size: 20px;
		}
	}

	@container hero (max-width: 459px) {
		.hero-cta {
			flex-wrap: wrap;
		}

		.stats-ribbon {
			flex-wrap: wrap;
		}

		.stat-cell {
			flex: 1 1 50%;
		}

		.stat-cell:nth-child(odd)::before {
			display: none;
		}
	}

	@media (max-width: 760px) {
		:global(.hero[data-variant='feed']) {
			padding: 32px 20px 24px;
		}
	}
</style>
