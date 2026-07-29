<script lang="ts">
	import SettingsHero from '$lib/components/settings/SettingsHero.svelte';
	import type { ReaderPreviewStyles } from '../reading-appearance-model';

	interface Props {
		preview: ReaderPreviewStyles;
	}

	let { preview }: Props = $props();
</script>

<SettingsHero variant="reading">
	<div class="hero-text">
		<div class="hero-eyebrow">Reading &amp; Appearance</div>
		<h1 class="hero-title">Read like you <em>mean it.</em></h1>
		<p class="hero-sub">
			A reader, a theme, a feel. Tune the page below — the preview to the right responds in kind.
		</p>
	</div>
	<div class="reader-tile-wrap">
		<article
			class="reader-tile"
			style:--r-font-family={preview.fontFamily}
			style:--r-font-size={preview.fontSize}
			style:--r-line-height={preview.lineHeight}
		>
			<div class="reader-tile-bar">
				<span class="dot"></span><span class="dot"></span><span class="dot"></span>
			</div>
			<div class="reader-tile-byline">
				<span class="ind">i</span>
				<span>Indelible Journal</span>
				<span>·</span>
				<span>3 min read</span>
			</div>
			<h2 class="reader-tile-h">What we save says who we are.</h2>
			<div class="reader-tile-rule"></div>
			<p class="reader-tile-p">
				An archive worth keeping is one you'd want to revisit. Indelible holds onto the things you
				care about — articles, highlights, notes, papers — so you can come back to them at
				<span class="lk">your own pace, in your own light</span>.
			</p>
		</article>
	</div>
</SettingsHero>

<style>
	.hero-text {
		display: flex;
		flex-direction: column;
		gap: 12px;
		max-width: 380px;
	}

	.hero-eyebrow {
		font-family: var(--font-sans);
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.16em;
		text-transform: uppercase;
		color: var(--hero-reading-eyebrow);
		display: inline-flex;
		align-items: center;
		gap: 10px;
	}

	.hero-eyebrow::before {
		content: '';
		width: 22px;
		height: 1px;
		background: currentColor;
		opacity: 0.5;
	}

	.hero-title {
		font-family: 'New York', 'Iowan Old Style', Georgia, 'Times New Roman', serif;
		font-size: 38px;
		font-weight: 600;
		line-height: 1.04;
		color: var(--hero-reading-name);
		margin: 0;
	}

	.hero-title em {
		font-style: italic;
		color: var(--hero-reading-name);
		opacity: 0.86;
	}

	.hero-sub {
		font-family: var(--font-sans);
		font-size: 14px;
		color: var(--hero-reading-sub);
		line-height: 1.55;
		max-width: 340px;
		margin: 0;
	}

	.reader-tile-wrap {
		position: relative;
		display: flex;
		justify-content: flex-start;
		align-items: center;
		flex: 1;
		min-width: 0;
	}

	.reader-tile-wrap::before,
	.reader-tile-wrap::after {
		content: '';
		position: absolute;
		inset: 0;
		border-radius: 16px;
		background: var(--hero-reading-tile-bg);
		opacity: 0.55;
		box-shadow: var(--hero-reading-tile-stack-shadow);
		z-index: 0;
	}

	.reader-tile-wrap::before {
		transform: rotate(-2.4deg) translate(-12px, 6px);
	}

	.reader-tile-wrap::after {
		transform: rotate(1.6deg) translate(8px, -4px);
		opacity: 0.4;
	}

	.reader-tile {
		position: relative;
		z-index: 1;
		background: var(--hero-reading-tile-bg);
		border-radius: 16px;
		box-shadow: var(--hero-reading-tile-shadow);
		padding: 28px 32px 30px;
		width: 100%;
		max-width: 520px;
		color: var(--hero-reading-tile-text);
		transition: background 200ms ease;
	}

	.reader-tile-bar {
		display: flex;
		gap: 5px;
		margin-bottom: 18px;
	}

	.reader-tile-bar .dot {
		width: 9px;
		height: 9px;
		border-radius: 50%;
		background: var(--hero-reading-tile-rule);
	}

	.reader-tile-byline {
		display: flex;
		gap: 8px;
		align-items: center;
		font-family: var(--font-sans);
		font-size: 11px;
		font-weight: 500;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		color: var(--hero-reading-tile-byline);
		margin-bottom: 12px;
	}

	.reader-tile-byline .ind {
		width: 14px;
		height: 14px;
		border-radius: 4px;
		background: linear-gradient(135deg, var(--accent), var(--accent-swatch-indigo));
		color: var(--text-on-color);
		display: inline-flex;
		align-items: center;
		justify-content: center;
		font-size: 9px;
		font-weight: 800;
	}

	.reader-tile-h {
		font-family: var(--r-font-family);
		font-size: calc(var(--r-font-size) * 1.85);
		font-weight: 600;
		line-height: 1.12;
		color: var(--hero-reading-tile-text);
		margin: 0 0 14px;
		transition:
			font-family 180ms,
			font-size 180ms;
	}

	.reader-tile-rule {
		height: 1px;
		width: 36px;
		background: var(--accent);
		border-radius: 1px;
		margin-bottom: 14px;
	}

	.reader-tile-p {
		font-family: var(--r-font-family);
		font-size: var(--r-font-size);
		line-height: var(--r-line-height);
		color: var(--hero-reading-tile-text);
		margin: 0;
		transition:
			font-family 180ms,
			font-size 180ms,
			line-height 180ms;
	}

	.reader-tile-p .lk {
		color: var(--accent);
		border-bottom: 1px solid currentColor;
		padding-bottom: 1px;
	}

	:global(.hero[data-variant='reading'] .hero-inner) {
		display: grid;
		grid-template-columns: minmax(280px, 380px) 1fr;
		gap: 56px;
		align-items: center;
		max-width: 1100px;
	}

	/* Below the text column minimum (280) + gap (56) + a usable tile (~380),
	   two columns crush the tile to a sliver — stack instead. */
	@container hero (max-width: 719px) {
		:global(.hero[data-variant='reading'] .hero-inner) {
			grid-template-columns: 1fr;
			gap: 28px;
		}
	}
</style>
