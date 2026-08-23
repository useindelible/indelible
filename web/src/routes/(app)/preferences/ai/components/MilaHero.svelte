<script lang="ts">
	import SettingsHero from '$lib/components/settings/SettingsHero.svelte';
	import { t } from '$lib/i18n';

	interface Props {
		enabled: boolean;
		onToggleEnabled: () => void;
	}

	let { enabled, onToggleEnabled }: Props = $props();
</script>

<SettingsHero variant="mila">
	<div class="hero-text">
		<div class="hero-eyebrow"><span>{$t('prefs_ai_hero_eyebrow')}</span></div>
		<h1 class="hero-headline">{$t('prefs_ai_hero_title')}</h1>
		<p class="hero-sub">{$t('prefs_ai_hero_description')}</p>
		<div class="enable-bar">
			<button
				type="button"
				class="toggle"
				class:on={enabled}
				role="switch"
				aria-checked={enabled}
				aria-label={$t(enabled ? 'prefs_ai_disable_mila' : 'prefs_ai_enable_mila')}
				onclick={onToggleEnabled}
			>
				<span class="toggle-track"></span>
			</button>
			<div>
				<div class="enable-label">{$t('prefs_ai_enable_mila')}</div>
				<div class="enable-state">
					{$t('prefs_ai_enable_state', {
						values: { state: $t(enabled ? 'prefs_ai_on' : 'prefs_ai_off') }
					})}
				</div>
			</div>
		</div>
	</div>
	<div class="mila-orb-wrap" aria-hidden="true">
		<div class="mila-glow"></div>
		<div class="mila-orb"></div>
	</div>
</SettingsHero>

<style>
	:global(.hero[data-variant='mila']) {
		padding: 40px 56px 36px;
	}
	/* The trailing combinator must live inside :global() — hero-inner carries
	   SettingsHero's scope hash, so a page-scoped `> div` never matches. */
	:global(.hero[data-variant='mila'] > div) {
		display: flex;
		flex-direction: row;
		align-items: center;
		gap: 28px;
		max-width: 1080px;
		width: 100%;
	}
	.hero-text {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-width: 0;
	}
	.hero-eyebrow {
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0;
		text-transform: uppercase;
		color: var(--text-secondary);
		margin-bottom: 10px;
	}
	.hero-headline {
		font-size: 30px;
		font-weight: 700;
		letter-spacing: 0;
		color: var(--text-primary);
		line-height: 1.1;
		margin: 0 0 10px;
	}
	.hero-sub {
		font-size: 14.5px;
		color: var(--text-secondary);
		line-height: 1.5;
		max-width: 540px;
		letter-spacing: 0;
		margin: 0;
	}
	.enable-bar {
		margin-top: 16px;
		display: inline-flex;
		align-items: center;
		gap: 12px;
		padding: 8px 14px 8px 10px;
		border-radius: 980px;
		background: var(--mila-metric-card-bg);
		box-shadow:
			0 0 0 0.5px var(--mila-metric-card-border),
			0 4px 14px var(--mila-violet-soft);
		align-self: flex-start;
		width: max-content;
	}
	.enable-label {
		font-size: 13px;
		font-weight: 600;
		color: var(--mila-metric-num-color, var(--text-primary));
	}
	.enable-state {
		font-size: 11.5px;
		color: var(--mila-metric-label-color, var(--text-secondary));
	}
	.toggle {
		display: inline-flex;
		align-items: center;
		cursor: pointer;
		flex-shrink: 0;
		background: transparent;
		border: 0;
		padding: 0;
	}
	.toggle-track {
		width: 36px;
		height: 21px;
		border-radius: 980px;
		background: var(--mila-status-idle-bg);
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		position: relative;
	}
	.toggle-track::after {
		content: '';
		position: absolute;
		left: 2px;
		top: 2px;
		width: 17px;
		height: 17px;
		border-radius: 50%;
		background: var(--bg-primary);
		box-shadow:
			0 1px 2px rgba(0, 0, 0, 0.2),
			0 0 0 0.5px rgba(0, 0, 0, 0.05);
	}
	.toggle.on .toggle-track {
		background: var(--mila-violet);
	}
	.toggle.on .toggle-track::after {
		left: 17px;
	}
	.mila-orb-wrap {
		flex-shrink: 0;
		width: 168px;
		height: 168px;
		position: relative;
		display: flex;
		align-items: center;
		justify-content: center;
	}
	.mila-orb {
		width: 132px;
		height: 132px;
		border-radius: 50%;
		background: linear-gradient(135deg, var(--mila-violet) 0%, var(--mila-violet-strong) 100%);
		box-shadow: var(--mila-orb-shadow);
		position: relative;
		z-index: 2;
	}
	.mila-orb::after {
		content: '';
		position: absolute;
		inset: 14% 18% 38% 22%;
		background: var(--mila-orb-highlight);
		border-radius: 50%;
		filter: blur(2px);
	}
	.mila-glow {
		position: absolute;
		inset: 0;
		border-radius: 50%;
		background: radial-gradient(circle at 50% 50%, var(--mila-violet-soft) 0%, transparent 65%);
		filter: blur(14px);
		z-index: 1;
	}
	/* The orb is decorative; below its 168px plus a readable text column,
	   drop it rather than squeeze the text. */
	@container hero (max-width: 509px) {
		.mila-orb-wrap {
			display: none;
		}
	}
	@media (max-width: 720px) {
		:global(.hero[data-variant='mila']) {
			padding: 32px 20px 28px;
		}
	}
</style>
