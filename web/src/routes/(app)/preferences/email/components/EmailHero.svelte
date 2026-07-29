<script lang="ts">
	import { formatRelative } from '../email-model';

	interface Props {
		senderCount: number;
		totalDeliveries: number;
		totalBlocked: number;
		lastDelivery: string | null | undefined;
	}

	let { senderCount, totalDeliveries, totalBlocked, lastDelivery }: Props = $props();
</script>

<section class="hero" aria-label="Email hero">
	<div class="airmail-strip" aria-hidden="true"></div>
	<div class="hero-postmark" aria-hidden="true">
		<div class="pm-arc-top">Indelible · Postroom</div>
		<div class="pm-rule"></div>
		<div class="pm-date">No. {senderCount || '—'}</div>
		<div class="pm-rule"></div>
		<div class="pm-arc-bottom">
			{new Date().toLocaleDateString(undefined, {
				day: '2-digit',
				month: 'short',
				year: 'numeric'
			})}
		</div>
	</div>

	<div class="hero-inner">
		<div class="hero-eyebrow">Postroom · {senderCount} senders on file</div>
		<h1 class="hero-headline">Newsletters arrive here. <em>You decide where they go.</em></h1>
		<p class="hero-sub">
			Two inboxes — one for the Feed, one for the Library. Every sender is logged, sorted, and one
			click from a clean unsubscribe.
		</p>

		<div class="stats-card" role="group" aria-label="Email statistics">
			<div class="stat-cell">
				<div class="stat-num">{senderCount}</div>
				<div class="stat-label">Senders</div>
				<div class="stat-sub">On file, all time</div>
			</div>
			<div class="stat-cell">
				<div class="stat-num">{totalDeliveries.toLocaleString()}</div>
				<div class="stat-label">Deliveries</div>
				<div class="stat-sub">Across both inboxes</div>
			</div>
			<div class="stat-cell" class:blocked={totalBlocked > 0}>
				<div class="stat-num">{totalBlocked}</div>
				<div class="stat-label">Blocked</div>
				<div class="stat-sub">Held at the door</div>
			</div>
			<div class="stat-cell">
				<div class="stat-num">{formatRelative(lastDelivery)}</div>
				<div class="stat-label">Last delivery</div>
				<div class="stat-sub">Most recent arrival</div>
			</div>
		</div>
	</div>
</section>

<style>
	.hero {
		position: relative;
		padding: 44px 56px 40px;
		overflow: hidden;
		border-bottom: 0.5px solid var(--border-hairline);
		/* The hero's available width depends on two collapsible sidebars, so
		   internal layout must query the hero itself, not the viewport. */
		container-type: inline-size;
		container-name: hero;
		background:
			radial-gradient(540px 380px at 92% -10%, var(--hero-blob-a), transparent 60%),
			radial-gradient(620px 460px at 0% 110%, var(--hero-blob-b), transparent 62%),
			radial-gradient(420px 280px at 55% 100%, var(--hero-blob-c), transparent 65%),
			linear-gradient(155deg, var(--hero-from) 0%, var(--hero-to) 100%);
	}

	.hero::after {
		content: '';
		position: absolute;
		left: 0;
		right: 0;
		bottom: 0;
		height: 1px;
		background-image: radial-gradient(circle, var(--text-quaternary) 0.6px, transparent 1px);
		background-size: 6px 1px;
		background-repeat: repeat-x;
		opacity: 0.6;
		z-index: 2;
		pointer-events: none;
	}

	.airmail-strip {
		position: absolute;
		left: 0;
		top: 0;
		bottom: 0;
		width: 22px;
		background-image: repeating-linear-gradient(
			135deg,
			var(--airmail-red) 0 10px,
			transparent 10px 16px,
			var(--airmail-navy) 16px 26px,
			transparent 26px 32px
		);
		opacity: 0.85;
		z-index: 1;
		pointer-events: none;
	}

	.hero-postmark {
		position: absolute;
		top: 32px;
		right: 56px;
		z-index: 3;
		width: 116px;
		height: 116px;
		border-radius: 50%;
		border: 1.5px dashed var(--stamp-line);
		color: var(--stamp-ink);
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 2px;
		transform: rotate(-9deg);
		opacity: 0.92;
		text-align: center;
		background: radial-gradient(circle at 50% 50%, var(--stamp-fill), transparent 70%);
		pointer-events: none;
	}

	.hero-postmark::before {
		content: '';
		position: absolute;
		inset: 6px;
		border-radius: 50%;
		border: 0.5px solid var(--stamp-line);
		opacity: 0.55;
	}

	.pm-arc-top,
	.pm-arc-bottom {
		font-family: var(--font-mono);
		font-size: 8.5px;
		font-weight: 600;
		letter-spacing: 0.16em;
		text-transform: uppercase;
	}

	.pm-date {
		font-family: var(--font-display);
		font-style: italic;
		font-weight: 500;
		font-size: 18px;
		letter-spacing: -0.02em;
		line-height: 1;
		margin: 1px 0;
	}

	.pm-rule {
		width: 56px;
		height: 1px;
		background: var(--stamp-line);
		opacity: 0.6;
		margin: 1px 0;
	}

	.hero-inner {
		position: relative;
		z-index: 3;
		display: flex;
		flex-direction: column;
		gap: 28px;
		max-width: 1080px;
		margin-left: 16px;
	}

	.hero-eyebrow {
		font-family: var(--font-mono);
		font-size: 10.5px;
		font-weight: 600;
		letter-spacing: 0.22em;
		text-transform: uppercase;
		color: var(--accent-strong);
		margin-bottom: 14px;
		display: inline-flex;
		align-items: center;
		gap: 12px;
	}

	.hero-eyebrow::before {
		content: '';
		width: 28px;
		height: 1px;
		background: var(--accent-line);
	}

	.hero-headline {
		font-family: var(--font-display);
		font-optical-sizing: auto;
		font-variation-settings:
			'SOFT' 30,
			'WONK' 0;
		font-size: 44px;
		font-weight: 500;
		letter-spacing: -0.03em;
		color: var(--text-primary);
		line-height: 1.02;
		margin: 0 0 12px;
		max-width: 720px;
	}

	.hero-headline em {
		font-style: italic;
		font-variation-settings:
			'SOFT' 100,
			'WONK' 1;
		font-weight: 500;
		color: var(--accent-strong);
	}

	.hero-sub {
		font-family: var(--font-body);
		font-size: 15px;
		color: var(--text-secondary);
		line-height: 1.5;
		letter-spacing: -0.005em;
		margin: 0;
		max-width: 560px;
	}

	.stats-card {
		display: flex;
		align-items: stretch;
		gap: 0;
		background: var(--paper);
		border-radius: 4px;
		box-shadow:
			var(--envelope-shadow),
			inset 0 0 0 0.5px var(--envelope-edge);
		overflow: hidden;
		max-width: 720px;
		position: relative;
	}

	.stats-card::before {
		content: '';
		position: absolute;
		left: 0;
		top: 0;
		bottom: 0;
		width: 18px;
		background-image: radial-gradient(circle, var(--perf-dot) 1.2px, transparent 1.6px);
		background-size: 18px 14px;
		background-position: 9px 9px;
		background-repeat: repeat-y;
		opacity: 0.55;
		pointer-events: none;
	}

	.stat-cell {
		flex: 1;
		padding: 14px 18px 14px 28px;
		display: flex;
		flex-direction: column;
		gap: 3px;
		position: relative;
	}

	.stat-cell + .stat-cell {
		padding-left: 18px;
	}

	.stat-cell + .stat-cell::before {
		content: '';
		position: absolute;
		left: 0;
		top: 12px;
		bottom: 12px;
		width: 0.5px;
		background: var(--envelope-edge);
	}

	.stat-num {
		font-family: var(--font-display);
		font-variation-settings:
			'SOFT' 50,
			'WONK' 0;
		font-size: 26px;
		font-weight: 500;
		letter-spacing: -0.03em;
		color: var(--text-primary);
		line-height: 1;
		display: inline-flex;
		align-items: baseline;
		gap: 5px;
	}

	.blocked .stat-num {
		color: var(--accent-strong);
	}

	.stat-label {
		font-family: var(--font-mono);
		font-size: 9.5px;
		font-weight: 600;
		letter-spacing: 0.18em;
		text-transform: uppercase;
		color: var(--text-tertiary);
	}

	.stat-sub {
		font-family: var(--font-body);
		font-size: 11.5px;
		color: var(--text-tertiary);
		letter-spacing: -0.005em;
		margin-top: 2px;
	}

	/* Four cells across need ~640px; below that fold the card into a 2×2 grid
	   and drop the decorative postmark. */
	@container hero (max-width: 639px) {
		.hero-postmark {
			display: none;
		}

		.stats-card {
			display: grid;
			grid-template-columns: repeat(2, 1fr);
		}

		.stat-cell {
			border-bottom: 0.5px solid var(--border-primary);
		}
	}

	@media (max-width: 900px) {
		.hero {
			padding: 28px 24px 24px;
		}

		.hero-headline {
			font-size: 24px;
		}
	}
</style>
