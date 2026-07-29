<script lang="ts">
	import { resolve } from '$app/paths';
	import type { IntegrationConnectionDto } from '$lib/api';
	import type { ObsidianHeroState } from '../obsidian-model';

	interface Props {
		connection: IntegrationConnectionDto;
		heroState: ObsidianHeroState;
	}

	let { connection, heroState }: Props = $props();
</script>

<section class="section">
	<div class="section-head">
		<h2 class="section-title">Sync</h2>
		<p class="section-sub">Vault writes happen from the Obsidian plugin</p>
	</div>
	<div class="card card-stack">
		<div class="row">
			<div>
				<p class="row-title">Manual vault sync</p>
				<p class="row-sub">
					Run this from the Obsidian plugin settings or command palette. The plugin downloads
					server-rendered artifacts and writes them into your open vault.
				</p>
			</div>
			<span class="tag-pill">In Obsidian</span>
		</div>

		<div class="row">
			<div>
				<p class="row-title">Plugin settings</p>
				<p class="row-sub">
					Schedule, sync-on-open, deleted-file resync, and current-file reimport confirmation live
					in the Obsidian plugin's settings.
				</p>
			</div>
			<span class="tag-pill">In Obsidian</span>
		</div>

		<div class="row">
			<div>
				<p class="row-title">Plugin access token</p>
				<p class="row-sub">
					The Obsidian plugin authenticates with a personal access token scoped to exports. Generate
					one in Developer settings, then paste it into the plugin.
				</p>
			</div>
			<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- href uses resolve() for the route and appends a query string. -->
			<a class="btn" href={`${resolve('/preferences/developer')}?scope=obsidian_plugin`}
				>Generate token</a
			>
		</div>

		{#if heroState === 'error' && connection.last_error}
			<div class="alert-block">
				<div class="alert">
					<strong>Last sync failed</strong>
					<p>Indelible will retry when the plugin next runs from Obsidian.</p>
					<span>{connection.last_error}</span>
				</div>
			</div>
		{/if}
	</div>
</section>

<style>
	.section {
		margin-top: 8px;
	}
	.section-head {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		margin: 0 4px 12px;
		gap: 12px;
	}
	.section-title {
		font-size: 12px;
		font-weight: 550;
		color: var(--text-tertiary);
		text-transform: uppercase;
		letter-spacing: 0;
		margin: 0;
	}
	.section-sub,
	.row-sub {
		font-size: 12.5px;
		color: var(--text-tertiary);
		margin: 0;
	}
	.card {
		background: var(--bg-elevated);
		border: 1px solid var(--border-hairline);
		border-radius: 14px;
		box-shadow: var(--shadow-1);
		overflow: hidden;
	}
	.card-stack > * + * {
		border-top: 1px solid var(--border-hairline);
	}
	.row {
		display: grid;
		grid-template-columns: 1fr auto;
		gap: 24px;
		align-items: center;
		padding: 14px 22px;
	}
	.row-title {
		font-size: 14px;
		font-weight: 500;
		margin: 0;
		color: var(--text-primary);
	}
	.row-sub {
		margin-top: 4px;
		line-height: 1.45;
		max-width: 64ch;
	}
	.btn,
	.tag-pill {
		display: inline-flex;
		align-items: center;
		border-radius: 8px;
		font-size: 13px;
		font-weight: 500;
		white-space: nowrap;
	}
	.btn {
		padding: 7px 12px;
		border: 1px solid var(--border-hairline);
		background: var(--bg-elevated);
		color: var(--text-primary);
	}
	.tag-pill {
		padding: 4px 10px;
		border-radius: 999px;
		font-size: 12px;
		background: var(--obs-accent-soft);
		color: var(--obs-accent-ink);
		border: 1px solid color-mix(in oklab, var(--obs-accent) 16%, transparent);
	}
	.alert-block {
		padding: 16px 22px 18px;
	}
	.alert {
		padding: 14px;
		border-radius: 12px;
		background: var(--obs-alert-bg);
		border: 1px solid var(--obs-alert-border);
		color: var(--text-secondary);
	}
	.alert strong {
		color: var(--text-primary);
	}
	.alert p {
		margin: 4px 0 10px;
	}
	.alert span {
		font-family: var(--font-mono, ui-monospace, monospace);
		font-size: 11.5px;
		color: var(--obs-alert-text);
		word-break: break-word;
	}
</style>
