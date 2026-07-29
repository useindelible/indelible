<script lang="ts">
	import SettingsHero from '$lib/components/settings/SettingsHero.svelte';
	import type { ConnectionState } from '$lib/integrations/status';
	import NotionHeroTile from './NotionHeroTile.svelte';

	interface Props {
		heroDocs: number;
		formattedHeroLastSync: string;
		workspaceIcon: string | null;
		workspaceName: string | null;
		databaseLabel: string;
		connectionState: ConnectionState;
		heroStatus: string;
		pendingJobs: number;
	}

	let {
		heroDocs,
		formattedHeroLastSync,
		workspaceIcon,
		workspaceName,
		databaseLabel,
		connectionState,
		heroStatus,
		pendingJobs
	}: Props = $props();
</script>

<SettingsHero variant="notion">
	<div class="hero-text">
		<div class="hero-eyebrow">
			<span class="conn-glyph" aria-hidden="true">
				<span class="i">i</span>
				<span class="x">×</span>
				<span class="n">N</span>
			</span>
			<span>Indelible × Notion</span>
		</div>
		<h1 class="hero-title">Your library,<br />kept in Notion.</h1>
		<p class="hero-sub">
			Indelible exports every saved document, highlight, and note into a managed database in your
			Notion workspace. Append-only by default — your edits stay safe.
		</p>
		<div class="hero-stats" aria-label="Notion export summary">
			<div class="hero-stat">
				<span class="v">{heroDocs}</span>
				<span class="l">Documents</span>
			</div>
			<div class="hero-stat-divider"></div>
			<div class="hero-stat">
				<span class="v">{formattedHeroLastSync}</span>
				<span class="l">Last sync</span>
			</div>
		</div>
	</div>

	<NotionHeroTile
		{workspaceIcon}
		{workspaceName}
		{databaseLabel}
		{connectionState}
		{heroStatus}
		{formattedHeroLastSync}
		{pendingJobs}
	/>
</SettingsHero>

<style>
	:global(.hero[data-variant='notion']) {
		padding: 30px 56px 36px;
	}

	:global(.hero[data-variant='notion'])::after {
		inset: 0;
		height: auto;
		background: repeating-linear-gradient(
			0deg,
			transparent 0,
			transparent 31px,
			var(--hero-notion-rule) 31px,
			var(--hero-notion-rule) 32px
		);
		opacity: 1;
		pointer-events: none;
	}

	:global(.hero[data-variant='notion'] > div) {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto;
		align-items: center;
		gap: clamp(48px, 8vw, 120px);
		max-width: none;
		width: 100%;
	}

	:global(.hero[data-variant='notion'] > div > :nth-child(2)) {
		justify-self: end;
		margin-right: clamp(24px, 4vw, 64px);
	}

	.hero-text {
		max-width: 460px;
	}

	.hero-eyebrow {
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.16em;
		text-transform: uppercase;
		color: var(--hero-notion-eyebrow);
		margin-bottom: 14px;
		display: inline-flex;
		align-items: center;
		gap: 8px;
	}

	.conn-glyph {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		color: var(--notion-accent);
	}

	.conn-glyph .i,
	.conn-glyph .n {
		width: 14px;
		height: 14px;
		display: flex;
		align-items: center;
		justify-content: center;
		line-height: 1;
	}

	.conn-glyph .i {
		border-radius: 4px;
		background: linear-gradient(
			135deg,
			var(--hero-notion-indelible-from),
			var(--hero-notion-indelible-to)
		);
		color: var(--text-on-color);
		font-weight: 800;
		font-size: 9px;
		letter-spacing: -0.04em;
	}

	.conn-glyph .x {
		font-size: 12px;
		opacity: 0.6;
	}

	.conn-glyph .n {
		border-radius: 3px;
		background: var(--hero-notion-icon-bg);
		color: var(--hero-notion-icon-fg);
		font-family: 'New York', 'Iowan Old Style', Georgia, serif;
		font-style: italic;
		font-weight: 700;
		font-size: 10px;
	}

	.hero-title {
		font-family: 'New York', 'Iowan Old Style', Georgia, 'Times New Roman', serif;
		font-style: italic;
		font-weight: 600;
		font-size: 38px;
		line-height: 1.06;
		letter-spacing: -0.025em;
		color: var(--hero-notion-name);
		margin: 0 0 14px;
	}

	.hero-sub {
		font-size: 14px;
		line-height: 1.55;
		color: var(--hero-notion-sub);
		max-width: 440px;
		margin: 0 0 20px;
	}

	.hero-stats {
		display: flex;
		align-items: center;
		gap: 18px;
		flex-wrap: wrap;
	}

	.hero-stat {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.hero-stat .v {
		font-size: 18px;
		font-weight: 700;
		letter-spacing: -0.025em;
		color: var(--hero-notion-name);
		font-feature-settings: 'tnum' on;
	}

	.hero-stat .l {
		font-size: 10.5px;
		font-weight: 600;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		color: var(--hero-notion-eyebrow);
	}

	.hero-stat-divider {
		width: 0.5px;
		height: 28px;
		background: var(--hero-notion-edge);
	}

	/* The tile column needs ~320px plus gap and end margin; below that plus a
	   readable text column the grid must stack. Width is hero-driven (two
	   collapsible sidebars), so query the hero container, not the viewport. */
	@container hero (max-width: 699px) {
		:global(.hero[data-variant='notion'] > div) {
			grid-template-columns: 1fr;
			gap: 24px;
		}

		:global(.hero[data-variant='notion'] > div > :nth-child(2)) {
			justify-self: stretch;
			margin-right: 0;
		}
	}

	@media (max-width: 720px) {
		:global(.hero[data-variant='notion']) {
			padding: 24px 20px 28px;
		}
	}
</style>
