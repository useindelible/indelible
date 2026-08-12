<script lang="ts">
	import SettingsHero from '$lib/components/settings/SettingsHero.svelte';

	interface Props {
		tokenCount: number;
		endpointCount: number;
	}

	let { tokenCount, endpointCount }: Props = $props();
</script>

<SettingsHero variant="developer">
	<div class="hero-text">
		<div>
			<div class="hero-eyebrow">
				<span class="dot"></span>
				<span>Developer · Tokens &amp; Webhooks</span>
			</div>
			<h1 class="hero-title">Wire Indelible<br />into your stack.</h1>
			<p class="hero-sub">
				PATs limit inbound API access through explicit permissions. Webhook deliveries are outbound
				and HMAC-signed with endpoint secrets.
			</p>
		</div>

		<div class="hero-stats">
			<div class="hero-stat">
				<div class="num">{tokenCount}</div>
				<div class="lbl">Tokens</div>
			</div>
			<div class="hero-stat">
				<div class="num">{endpointCount}</div>
				<div class="lbl">Endpoints</div>
			</div>
		</div>
	</div>
</SettingsHero>

<style>
	:global(.hero[data-variant='developer']) {
		padding: 48px 56px;
	}

	:global(.hero[data-variant='developer'])::after {
		background-image:
			linear-gradient(to right, var(--hero-developer-grid) 0.5px, transparent 0.5px),
			linear-gradient(to bottom, var(--hero-developer-grid) 0.5px, transparent 0.5px);
		background-size: 40px 40px;
		background-position: -1px -1px;
		opacity: 0.6;
		height: auto;
		inset: 0;
		pointer-events: none;
	}

	/* The trailing combinator must live inside :global() — hero-inner carries
	   SettingsHero's scope hash, so a page-scoped `> div` never matches. */
	:global(.hero[data-variant='developer'] > div) {
		display: block;
		max-width: 720px;
		width: 100%;
	}

	.hero-text {
		display: flex;
		flex-direction: column;
		justify-content: space-between;
		min-height: 220px;
	}

	.hero-eyebrow {
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.14em;
		text-transform: uppercase;
		color: var(--hero-developer-eyebrow);
		margin-bottom: 14px;
		display: inline-flex;
		align-items: center;
		gap: 8px;
	}

	.dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--hero-developer-eyebrow);
		opacity: 0.7;
	}

	.hero-title {
		font-family: 'New York', 'Iowan Old Style', Georgia, 'Times New Roman', serif;
		font-style: italic;
		font-weight: 600;
		font-size: 40px;
		line-height: 1.05;
		letter-spacing: -0.034em;
		color: var(--hero-developer-name);
		margin-bottom: 14px;
	}

	.hero-sub {
		font-size: 14px;
		line-height: 1.55;
		color: var(--hero-developer-sub);
		max-width: 440px;
	}

	.hero-stats {
		display: grid;
		grid-template-columns: repeat(2, max-content);
		gap: 28px;
		margin-top: 28px;
	}

	.num {
		font-family: 'New York', Georgia, serif;
		font-size: 26px;
		font-weight: 600;
		color: var(--hero-developer-name);
		letter-spacing: -0.04em;
		line-height: 1;
		display: inline-flex;
		align-items: baseline;
		gap: 4px;
	}

	.lbl {
		font-size: 10.5px;
		letter-spacing: 0.12em;
		text-transform: uppercase;
		color: var(--hero-developer-sub);
		margin-top: 6px;
		font-weight: 600;
	}

	@media (max-width: 599px) {
		:global(.hero[data-variant='developer']) {
			padding: 24px 16px 22px;
		}
	}
</style>
