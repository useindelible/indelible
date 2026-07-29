<script lang="ts">
	import type { ImportJobStatusResponse } from '$lib/api';
	import { isTerminalImportStatus, normalizeImportStatus } from '$lib/integrations/status';
	import { relativeTime, sourceFileLabel, statusForJob } from '../integrations-hub-model';

	interface Props {
		history: ImportJobStatusResponse[];
		onRollback: (jobId: string) => void;
	}

	let { history, onRollback }: Props = $props();
</script>

{#if history.length === 0}
	<p class="zone-meta">No imports yet. Drop a file above to start.</p>
{:else}
	<div class="history-table">
		<table>
			<thead>
				<tr>
					<th>Source</th>
					<th>Started</th>
					<th>Status</th>
					<th class="num-col">Items</th>
					<th class="num-col">Action</th>
				</tr>
			</thead>
			<tbody>
				{#each history as job (job.id)}
					{@const status = statusForJob(job)}
					<tr>
						<td>
							<div class="source-cell">
								<div class="source-mark readwise">R</div>
								<span class="source-name">{sourceFileLabel(job)}</span>
							</div>
						</td>
						<td class="when">{relativeTime(job.created_at)}</td>
						<td>
							<span class="status-pill {status.variant}">
								{#if status.variant === 'active'}
									<svg viewBox="0 0 24 24" aria-hidden="true">
										<polyline points="20 6 9 17 4 12" />
									</svg>
								{:else if status.variant === 'syncing' || status.variant === 'attention'}
									<span class="pulse-dot"></span>
								{/if}
								{status.label}
							</span>
						</td>
						<td class="num num-col">
							{(job.counts.imported + job.counts.updated).toLocaleString()}
						</td>
						<td class="num-col">
							{#if isTerminalImportStatus(job.status) && normalizeImportStatus(job.status) !== 'rolled_back'}
								<button class="table-action" onclick={() => onRollback(job.id)}>Roll back</button>
							{:else}
								<span class="table-empty">—</span>
							{/if}
						</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
{/if}

<style>
	.zone-meta {
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--text-tertiary);
		margin: 0;
	}

	.history-table {
		background: var(--bg-elevated);
		border-radius: 14px;
		overflow: hidden;
		box-shadow: var(--int-shadow-card);
		width: 100%;
	}

	table {
		width: 100%;
		border-collapse: collapse;
		font-size: 12.5px;
	}

	th {
		text-align: left;
		padding: 11px 16px;
		font-size: 10.5px;
		font-weight: 600;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		color: var(--text-tertiary);
		background: var(--int-table-head-bg);
		border-bottom: 0.5px solid var(--border-primary);
	}

	th.num-col,
	td.num-col {
		text-align: right;
	}

	td {
		padding: 13px 16px;
		border-bottom: 0.5px solid var(--border-hairline);
		color: var(--text-primary);
		letter-spacing: -0.005em;
		vertical-align: middle;
	}

	tr:last-child td {
		border-bottom: none;
	}

	tr:hover {
		background: var(--int-table-row-hover);
	}

	.source-cell {
		display: flex;
		align-items: center;
		gap: 9px;
	}

	.source-mark {
		width: 22px;
		height: 22px;
		border-radius: 6px;
		flex-shrink: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		background: var(--bg-secondary);
		color: var(--text-secondary);
		font-weight: 700;
		font-size: 11px;
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
	}

	.source-name {
		font-size: 13px;
		font-weight: 500;
		letter-spacing: -0.01em;
	}

	.num,
	.when {
		font-variant-numeric: tabular-nums;
		color: var(--text-secondary);
	}

	.when {
		font-size: 12px;
	}

	.status-pill {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 2px 8px;
		border-radius: 980px;
		font-size: 10.5px;
		font-weight: 600;
		letter-spacing: 0.005em;
		flex-shrink: 0;
	}

	.status-pill svg {
		width: 9px;
		height: 9px;
		stroke: currentColor;
		fill: none;
		stroke-width: 2;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.status-pill.active {
		background: var(--int-status-active-bg);
		color: var(--int-status-active-text);
	}

	.status-pill.syncing {
		background: var(--int-status-syncing-bg);
		color: var(--int-status-syncing-text);
	}

	.status-pill.attention {
		background: var(--int-status-attention-bg);
		color: var(--int-status-attention-text);
	}

	.status-pill.coming {
		background: var(--int-status-coming-bg);
		color: var(--int-status-coming-text);
	}

	.pulse-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: currentColor;
	}

	.table-action {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 4px 10px;
		border-radius: 7px;
		font-size: 11.5px;
		font-weight: 500;
		color: var(--text-secondary);
		background: transparent;
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		border: none;
		cursor: pointer;
	}

	.table-action:hover {
		background: var(--fill-hover);
		color: var(--text-primary);
	}

	.table-empty {
		color: var(--text-tertiary);
	}
</style>
