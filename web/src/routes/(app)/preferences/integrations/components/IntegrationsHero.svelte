<script lang="ts">
	import SettingsHero from '$lib/components/settings/SettingsHero.svelte';
	import type { RingCounts, RingDash, SevenDayDelta } from '../integrations-hub-model';
	import QuickstartGrid from './QuickstartGrid.svelte';
	import { number, t } from '$lib/i18n';

	interface Props {
		heroState: 'populated' | 'empty';
		ringCounts: RingCounts;
		ringDash: RingDash;
		sevenDayItems: number;
		sevenDayDelta: SevenDayDelta | null;
		onCopyInbox: () => void;
		onStartNotion: () => void;
	}

	let {
		heroState,
		ringCounts,
		ringDash,
		sevenDayItems,
		sevenDayDelta,
		onCopyInbox,
		onStartNotion
	}: Props = $props();
</script>

<SettingsHero variant="integrations">
	<div class="hero-text">
		<div class="hero-eyebrow">{$t('integrations_hub_eyebrow')}</div>
		{#if heroState === 'populated'}
			<h1 class="hero-headline">{$t('integrations_hub_populated_title')}</h1>
			<p class="hero-sub">{$t('integrations_hub_populated_subtitle')}</p>
		{:else}
			<h1 class="hero-headline">{$t('integrations_hub_empty_title')}</h1>
			<p class="hero-sub">{$t('integrations_hub_empty_subtitle')}</p>
		{/if}
	</div>

	{#if heroState === 'populated'}
		<div class="connection-ring-wrap">
			<div class="connection-ring">
				<svg viewBox="0 0 100 100" aria-hidden="true">
					<circle class="ring-track-circle" cx="50" cy="50" r="42" />
					<circle
						class="ring-arc connected"
						cx="50"
						cy="50"
						r="42"
						stroke-dasharray="{ringDash.connected.dash} 264"
						stroke-dashoffset={ringDash.connected.offset}
					/>
					<circle
						class="ring-arc syncing"
						cx="50"
						cy="50"
						r="42"
						stroke-dasharray="{ringDash.syncing.dash} 264"
						stroke-dashoffset={ringDash.syncing.offset}
					/>
					<circle
						class="ring-arc attention"
						cx="50"
						cy="50"
						r="42"
						stroke-dasharray="{ringDash.attention.dash} 264"
						stroke-dashoffset={ringDash.attention.offset}
					/>
				</svg>
				<div class="connection-ring-center">
					<div class="ring-num">{ringCounts.total}</div>
					<div class="ring-lbl">{$t('integrations_hub_sources')}</div>
				</div>
			</div>
			<div class="ring-legend">
				<div>
					<span class="dot connected"></span>{$t('integrations_hub_connected_count', {
						values: { count: ringCounts.connected }
					})}
				</div>
				<div>
					<span class="dot syncing"></span>{$t('integrations_hub_syncing_count', {
						values: { count: ringCounts.syncing }
					})}
				</div>
				<div>
					<span class="dot attention"></span>{$t('integrations_hub_attention_count', {
						values: { count: ringCounts.attention }
					})}
				</div>
			</div>
		</div>

		<div class="spark-card">
			<div class="spark-label">{$t('integrations_hub_seven_day_activity')}</div>
			<div class="spark-num">
				{$number(sevenDayItems)}
				{#if sevenDayDelta}
					<span class="delta" data-sign={sevenDayDelta.sign}
						>{sevenDayDelta.label === 'new'
							? $t('integrations_hub_new')
							: sevenDayDelta.label}</span
					>
				{/if}
			</div>
		</div>
	{:else}
		<QuickstartGrid {onCopyInbox} {onStartNotion} />
	{/if}
</SettingsHero>

<style>
	:global(.hero[data-variant='integrations']) {
		padding: 40px 56px 36px;
	}

	:global(.hero[data-variant='integrations'] .hero-inner) {
		display: flex;
		align-items: center;
		gap: 36px;
		max-width: 1080px;
	}

	.hero-text {
		flex: 1;
		min-width: 0;
	}

	.hero-eyebrow {
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0;
		text-transform: uppercase;
		color: var(--hero-integrations-eyebrow);
		margin-bottom: 10px;
	}

	.hero-headline {
		font-size: 30px;
		font-weight: 700;
		letter-spacing: 0;
		color: var(--hero-integrations-headline);
		line-height: 1.08;
		margin: 0 0 8px;
	}

	.hero-sub {
		font-size: 14.5px;
		color: var(--hero-integrations-eyebrow);
		line-height: 1.45;
		max-width: 520px;
		letter-spacing: 0;
		margin: 0;
	}

	.connection-ring-wrap,
	.spark-card {
		flex-shrink: 0;
		background: var(--int-metric-card-bg);
		box-shadow:
			0 0 0 0.5px var(--int-metric-card-border),
			var(--shadow-1);
		backdrop-filter: blur(20px);
		-webkit-backdrop-filter: blur(20px);
	}

	.connection-ring-wrap {
		display: flex;
		align-items: center;
		gap: 18px;
		padding: 14px 22px 14px 18px;
		border-radius: 18px;
	}

	.connection-ring {
		position: relative;
		width: 96px;
		height: 96px;
		flex-shrink: 0;
	}

	.connection-ring svg {
		width: 100%;
		height: 100%;
		transform: rotate(-90deg);
	}

	.ring-track-circle,
	.ring-arc {
		fill: none;
		stroke-width: 9;
	}

	.ring-track-circle {
		stroke: var(--int-ring-track);
	}

	.ring-arc {
		stroke-linecap: round;
	}

	.ring-arc.connected {
		stroke: var(--int-ring-connected);
	}

	.ring-arc.syncing {
		stroke: var(--int-ring-syncing);
	}

	.ring-arc.attention {
		stroke: var(--int-ring-attention);
	}

	.connection-ring-center {
		position: absolute;
		inset: 0;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		color: var(--int-metric-num);
	}

	.ring-num {
		font-size: 24px;
		font-weight: 700;
		letter-spacing: 0;
		line-height: 1;
	}

	.ring-lbl,
	.spark-label {
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0;
		text-transform: uppercase;
		color: var(--int-metric-label);
	}

	.ring-legend {
		display: flex;
		flex-direction: column;
		gap: 7px;
		font-size: 12px;
		color: var(--int-metric-num);
		font-weight: 500;
		white-space: nowrap;
	}

	.dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		display: inline-flex;
		margin-right: 8px;
	}

	.dot.connected {
		background: var(--int-ring-connected);
	}

	.dot.syncing {
		background: var(--int-ring-syncing);
	}

	.dot.attention {
		background: var(--int-ring-attention);
	}

	.spark-card {
		padding: 14px 22px;
		border-radius: 18px;
		min-width: 200px;
	}

	.spark-num {
		font-size: 22px;
		font-weight: 700;
		letter-spacing: 0;
		color: var(--int-metric-num);
		line-height: 1.1;
		margin-top: 4px;
	}

	.delta {
		font-size: 12px;
		font-weight: 600;
		margin-left: 6px;
		color: var(--int-ring-connected);
		letter-spacing: 0;
	}

	.delta[data-sign='down'] {
		color: var(--int-ring-attention);
	}

	.delta[data-sign='flat'] {
		color: var(--int-metric-label);
	}

	/* The connected-state stats card runs up to 520px and doesn't shrink;
	   below that plus a readable text column the row must stack. */
	@container hero (max-width: 855px) {
		:global(.hero[data-variant='integrations'] .hero-inner) {
			flex-direction: column;
			align-items: flex-start;
		}
	}

	@media (max-width: 599px) {
		:global(.hero[data-variant='integrations']) {
			padding: 32px 20px 24px;
		}
	}
</style>
