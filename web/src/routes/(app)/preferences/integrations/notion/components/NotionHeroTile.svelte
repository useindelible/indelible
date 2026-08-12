<script lang="ts">
	import type { ConnectionState } from '$lib/integrations/status';

	interface Props {
		workspaceIcon: string | null;
		workspaceName: string | null;
		databaseLabel: string;
		connectionState: ConnectionState;
		heroStatus: string;
		formattedHeroLastSync: string;
		pendingJobs: number;
	}

	let {
		workspaceIcon,
		workspaceName,
		databaseLabel,
		connectionState,
		heroStatus,
		formattedHeroLastSync,
		pendingJobs
	}: Props = $props();

	const sampleRows = [
		['On the difficulty of writing', 'Substack', 'May 3'],
		['A short history of patience', 'Web', 'May 1'],
		['What it means to read again', 'Email', 'Apr 29']
	];
</script>

<div class="notion-tile" aria-hidden="true">
	<div class="notion-tile-header">
		<span class="notion-tile-emoji">{workspaceIcon ?? '📚'}</span>
		<span class="notion-tile-workspace">{workspaceName ?? 'Notion workspace'}</span>
		<span class="notion-tile-active-pill" class:attention={connectionState === 'failed'}>
			<span class="dot"></span>
			{heroStatus}
		</span>
	</div>
	<div class="notion-tile-title">
		<span class="db-icon">
			<svg viewBox="0 0 24 24"
				><ellipse cx="12" cy="6" rx="8" ry="2.5" /><path
					d="M4 6v6c0 1.4 3.6 2.5 8 2.5s8-1.1 8-2.5V6"
				/><path d="M4 12v6c0 1.4 3.6 2.5 8 2.5s8-1.1 8-2.5v-6" /></svg
			>
		</span>
		{databaseLabel}
	</div>
	<div class="notion-tile-example">Example preview</div>
	<div class="notion-tile-cols">
		<span class="notion-tile-col">
			<svg viewBox="0 0 24 24"><path d="M5 7h14M5 12h10M5 17h12" /></svg>
			Title
		</span>
		<span class="notion-tile-col">
			<svg viewBox="0 0 24 24"><path d="M4 6h12M4 10h16M4 14h12M4 18h16" /></svg>
			Source
		</span>
		<span class="notion-tile-col">
			<svg viewBox="0 0 24 24"
				><rect x="3" y="5" width="18" height="16" rx="2" /><path d="M3 9h18M8 3v4M16 3v4" /></svg
			>
			Saved
		</span>
	</div>
	{#each sampleRows as row (row[0])}
		<div class="notion-tile-row">
			<div class="title-cell">
				<span class="doc-icon">
					<svg viewBox="0 0 24 24"
						><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" /><path
							d="M14 2v6h6"
						/></svg
					>
				</span>
				{row[0]}
			</div>
			<div><span class="source-pill">{row[1]}</span></div>
			<div class="date-cell">{row[2]}</div>
		</div>
	{/each}
	<div class="notion-tile-footer">
		<span class="tile-meta">
			<span class="sync-mark">
				<svg viewBox="0 0 24 24"><path d="M21 12a9 9 0 1 1-3.5-7.1" /><path d="M21 4v5h-5" /></svg>
			</span>
			Last edited {formattedHeroLastSync}
		</span>
		<span>{pendingJobs} pending</span>
	</div>
</div>

<style>
	.notion-tile {
		background: var(--notion-tile-bg);
		border-radius: 12px;
		padding: 20px 22px;
		box-shadow: var(--notion-tile-shadow);
		transform: rotate(-1deg);
		color: var(--notion-tile-text);
		min-width: 320px;
		max-width: 420px;
		transition: transform 200ms ease;
	}

	.notion-tile:hover {
		transform: rotate(-0.4deg) translateY(-2px);
	}

	.notion-tile-header {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-bottom: 14px;
	}

	.notion-tile-emoji {
		font-size: 16px;
	}

	.notion-tile-workspace {
		font-size: 12px;
		color: var(--notion-tile-sub);
		flex: 1;
	}

	.notion-tile-active-pill {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.04em;
		color: var(--success);
		padding: 3px 8px;
		border-radius: 999px;
		background: var(--fill-success);
		text-transform: none;
	}

	.notion-tile-active-pill.attention {
		color: var(--destructive);
		background: var(--fill-danger);
	}

	.notion-tile-active-pill .dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: currentColor;
	}

	.notion-tile-title {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 16px;
		font-weight: 600;
		color: var(--notion-tile-text);
		margin-bottom: 14px;
		letter-spacing: -0.01em;
	}

	.notion-tile-title .db-icon {
		width: 18px;
		height: 18px;
		display: inline-flex;
		opacity: 0.7;
	}

	.notion-tile-title .db-icon svg,
	.notion-tile-col svg,
	.doc-icon svg,
	.sync-mark svg {
		width: 100%;
		height: 100%;
		stroke: currentColor;
		fill: none;
	}

	.notion-tile-title .db-icon svg,
	.sync-mark svg {
		stroke-width: 1.6;
	}

	.notion-tile-example {
		margin-bottom: 6px;
		font-size: 9.5px;
		font-weight: 700;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		color: var(--notion-tile-sub);
	}

	.notion-tile-cols,
	.notion-tile-row {
		display: grid;
		grid-template-columns: 1.5fr 1fr 0.7fr;
		gap: 12px;
	}

	.notion-tile-cols {
		padding-bottom: 8px;
		border-bottom: 0.5px solid var(--notion-tile-divider);
		margin-bottom: 6px;
	}

	.notion-tile-col {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		font-size: 10px;
		font-weight: 500;
		letter-spacing: 0.05em;
		text-transform: uppercase;
		color: var(--notion-tile-sub);
	}

	.notion-tile-col svg {
		width: 11px;
		height: 11px;
		stroke-width: 1.5;
	}

	.notion-tile-row {
		padding: 8px 0;
		border-bottom: 0.5px solid var(--notion-tile-divider);
		font-size: 12px;
		align-items: center;
	}

	.notion-tile-row:last-of-type {
		border-bottom: none;
	}

	.title-cell {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		color: var(--notion-tile-text);
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.doc-icon {
		width: 12px;
		height: 12px;
		display: inline-flex;
		opacity: 0.55;
		flex-shrink: 0;
	}

	.doc-icon svg {
		stroke-width: 1.6;
	}

	.source-pill {
		display: inline-block;
		font-size: 10.5px;
		padding: 2px 7px;
		border-radius: 4px;
		background: var(--notion-tile-row, rgba(40, 30, 10, 0.05));
		color: var(--notion-tile-sub);
	}

	.date-cell {
		font-size: 11px;
		color: var(--notion-tile-sub);
		text-align: right;
		font-feature-settings: 'tnum' on;
	}

	.notion-tile-footer {
		margin-top: 10px;
		padding-top: 10px;
		border-top: 0.5px solid var(--notion-tile-divider);
		display: flex;
		justify-content: space-between;
		align-items: center;
		font-size: 10.5px;
		color: var(--notion-tile-sub);
	}

	.tile-meta {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		min-width: 0;
	}

	.sync-mark {
		width: 11px;
		height: 11px;
		display: inline-flex;
	}

	/* Matches the hero grid stack in NotionHero: when the hero is too narrow
	   for two columns, the tile stretches full width. */
	@container hero (max-width: 699px) {
		.notion-tile {
			max-width: none;
			min-width: 0;
		}
	}
</style>
