<script lang="ts">
	import SettingsHero from '$lib/components/settings/SettingsHero.svelte';
	import { TERMINAL_LINES } from '../developer-model';

	interface Props {
		activeTokens: number;
		endpointCount: number;
		eventsLast24: number;
		deliveryRate: string;
	}

	let { activeTokens, endpointCount, eventsLast24, deliveryRate }: Props = $props();
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
				<div class="num">{activeTokens}</div>
				<div class="lbl">Active tokens</div>
			</div>
			<div class="hero-stat">
				<div class="num">{endpointCount}</div>
				<div class="lbl">Endpoints</div>
			</div>
			<div class="hero-stat">
				<div class="num">{eventsLast24}</div>
				<div class="lbl">Events · 24h</div>
			</div>
			<div class="hero-stat">
				<div class="num">{deliveryRate}<span class="small">%</span></div>
				<div class="lbl">Delivery</div>
			</div>
		</div>
	</div>

	<div class="term" aria-hidden="true">
		<div class="term-bar">
			<div class="term-dots"><span></span><span></span><span></span></div>
			<div class="term-title">activity.log</div>
			<div class="term-pulse">Live</div>
		</div>
		<div class="term-body">
			{#each TERMINAL_LINES as line, index (line.ts)}
				<div class="term-line" style:animation-delay={`${index * 80}ms`}>
					<span class="ts">{line.ts}</span>
					<span class="method {line.methodClass}">{line.method}</span>
					<span class="path">
						{line.path}{#if line.target}
							<span class="arrow">→</span><span class="target">{line.target}</span>
						{/if}
					</span>
					<span class="status {line.statusClass}">{line.status}</span>
				</div>
			{/each}
			<div class="term-prompt">
				<span class="glyph">›</span>
				<span>tail -f /var/log/indelible/api</span>
				<span class="blink"></span>
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
		display: grid;
		grid-template-columns: minmax(0, 1fr) 480px;
		align-items: stretch;
		gap: 40px;
		max-width: 1080px;
		width: 100%;
	}

	.hero-text {
		display: flex;
		flex-direction: column;
		justify-content: space-between;
		min-height: 280px;
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
		grid-template-columns: repeat(4, auto);
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

	.small {
		font-size: 14px;
		font-weight: 500;
		color: var(--hero-developer-sub);
		font-style: italic;
	}

	.lbl {
		font-size: 10.5px;
		letter-spacing: 0.12em;
		text-transform: uppercase;
		color: var(--hero-developer-sub);
		margin-top: 6px;
		font-weight: 600;
	}

	.term {
		background: var(--dev-term-bg);
		backdrop-filter: blur(20px) saturate(180%);
		-webkit-backdrop-filter: blur(20px) saturate(180%);
		border-radius: 12px;
		box-shadow: var(--dev-term-shadow);
		overflow: hidden;
		position: relative;
		color: var(--dev-term-text);
		display: flex;
		flex-direction: column;
		min-height: 280px;
	}

	.term-bar,
	.term-line,
	.term-prompt {
		font-family: 'SF Mono', 'Fira Code', Menlo, ui-monospace, monospace;
	}

	.term-bar {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 9px 14px;
		background: var(--dev-term-bar);
		border-bottom: 0.5px solid var(--dev-term-divider);
		font-size: 11px;
		color: var(--dev-term-dim);
	}

	.term-dots {
		display: inline-flex;
		gap: 5px;
		margin-right: 4px;
	}

	.term-dots span {
		width: 9px;
		height: 9px;
		border-radius: 50%;
		background: var(--dev-term-divider);
	}

	.term-title {
		letter-spacing: 0.02em;
	}

	.term-pulse {
		margin-left: auto;
		color: var(--dev-success-fg);
		font-size: 10.5px;
		font-weight: 600;
		letter-spacing: 0.05em;
		text-transform: uppercase;
	}

	.term-body {
		padding: 12px 14px 14px;
		flex: 1;
		font-family: 'SF Mono', 'Fira Code', Menlo, ui-monospace, monospace;
		font-size: 11.5px;
		line-height: 1.65;
		overflow: hidden;
	}

	.term-line {
		display: grid;
		grid-template-columns: 56px 52px minmax(0, 1fr) auto;
		gap: 10px;
		align-items: baseline;
		padding: 2px 0;
		color: var(--dev-term-text);
		opacity: 0;
		animation: termLineIn 360ms cubic-bezier(0.2, 0.7, 0.3, 1) forwards;
	}

	@keyframes termLineIn {
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	.ts {
		color: var(--dev-term-dim);
		font-size: 11px;
	}

	.method,
	.status {
		font-weight: 700;
		letter-spacing: 0.04em;
		font-size: 10.5px;
	}

	.method.get {
		color: var(--dev-term-method-get);
	}
	.method.post {
		color: var(--dev-term-method-post);
	}
	.method.delete {
		color: var(--dev-term-method-delete);
	}
	.method.hook,
	.target {
		color: var(--dev-scope-ext-fg);
	}

	.path {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.arrow {
		color: var(--dev-term-dim);
		margin: 0 6px;
	}

	.status {
		text-align: right;
	}
	.status.s2xx {
		color: var(--dev-term-status-2xx);
	}
	.status.s4xx {
		color: var(--dev-term-status-4xx);
	}
	.status.s5xx {
		color: var(--dev-term-status-5xx);
	}

	.term-prompt {
		margin-top: 8px;
		color: var(--dev-term-dim);
		display: flex;
		gap: 6px;
		align-items: center;
		font-size: 11.5px;
	}

	.glyph,
	.blink {
		color: var(--dev-term-accent);
	}

	.blink {
		width: 6px;
		height: 12px;
		background: var(--dev-term-accent);
		margin-left: 2px;
		animation: caretBlink 1.1s steps(2, start) infinite;
	}

	@keyframes caretBlink {
		50% {
			opacity: 0;
		}
	}

	/* The terminal column is fixed at 480px; below that plus a readable
	   text column the grid must stack. */
	@container hero (max-width: 839px) {
		:global(.hero[data-variant='developer'] > div) {
			grid-template-columns: 1fr;
			max-width: none;
		}
	}

	@media (max-width: 599px) {
		:global(.hero[data-variant='developer']) {
			padding: 24px 16px 22px;
		}
	}
</style>
