<script lang="ts">
	import type { IntegrationConnectionDto } from '$lib/api';
	import type { NotionStatusSummary } from './notion-status-model';
	import { t } from '$lib/i18n';

	interface Props {
		connection: IntegrationConnectionDto;
		summary: NotionStatusSummary;
		syncing?: boolean;
		syncError?: string | null;
		onSync: () => void;
		onReauthorize: () => void;
	}

	let {
		connection,
		summary,
		syncing = false,
		syncError = null,
		onSync,
		onReauthorize
	}: Props = $props();
</script>

<section class="group">
	<div class="group-label">{$t('integrations_notion_sync')}</div>
	<div class="group-desc">
		{$t('integrations_notion_sync_description')}
	</div>

	<div class="group-card">
		<div class="sync-strip">
			<div class="sync-strip-meta">
				<div class="sync-stat">
					<span class="l">{$t('integrations_notion_status')}</span>
					<span class="v">
						<span class="pulse-dot" data-tone={summary.statusTone}></span>
						{summary.statusLabel}
					</span>
				</div>
				<div class="sync-stat">
					<span class="l">{$t('integrations_notion_last_sync')}</span>
					<span class="v muted">{summary.formattedLastSync}</span>
				</div>
				<div class="sync-stat">
					<span class="l">{$t('integrations_notion_pending_jobs')}</span>
					<span class="v muted" data-testid="notion-pending-pill">{summary.pendingJobs}</span>
				</div>
			</div>
			<div class="sync-strip-cta">
				<button
					class="btn primary"
					type="button"
					onclick={onSync}
					disabled={syncing}
					data-testid="notion-start-export"
				>
					{#if syncing}
						<span class="spinner"></span>
						{$t('integrations_notion_exporting')}
					{:else}
						<svg viewBox="0 0 24 24"><path d="M5 12h14" /><path d="M13 5l7 7-7 7" /></svg>
						{$t('integrations_notion_start_export')}
					{/if}
				</button>
			</div>
		</div>

		{#if syncError}
			<div class="callout error" role="alert">
				<div class="callout-body">
					<div class="callout-title">{$t('integrations_notion_export_failed')}</div>
					<div class="callout-detail">{syncError}</div>
				</div>
			</div>
		{/if}

		{#if summary.isAuthFailure}
			<div class="callout error" role="alert" data-testid="notion-auth-failure">
				<div class="callout-body">
					<div class="callout-title">{$t('integrations_notion_authorization_attention')}</div>
					<div class="callout-detail">
						{$t('integrations_notion_authorization_attention_hint')}
					</div>
					<div class="callout-actions">
						<button class="btn primary" type="button" onclick={onReauthorize}
							>{$t('integrations_notion_reauthorize')}</button
						>
					</div>
				</div>
			</div>
		{:else if summary.isSchemaError}
			<div class="callout error" role="alert" data-testid="notion-schema-error">
				<div class="callout-body">
					<div class="callout-title">{$t('integrations_notion_schema_changed')}</div>
					<div class="callout-detail">
						{$t('integrations_notion_schema_changed_hint')}
					</div>
				</div>
			</div>
		{:else if summary.isRateLimited}
			<div class="callout warning" role="status" data-testid="notion-rate-limit">
				<div class="callout-body">
					<div class="callout-title">{$t('integrations_notion_rate_limit_reached')}</div>
					<div class="callout-detail">{$t('integrations_notion_rate_limit_hint')}</div>
				</div>
			</div>
		{:else if connection.last_error}
			<div class="callout error" role="alert">
				<div class="callout-body">
					<div class="callout-title">{$t('integrations_notion_last_sync_error')}</div>
					<div class="callout-detail">{connection.last_error}</div>
				</div>
			</div>
		{/if}
	</div>
</section>

<style>
	.group {
		margin-bottom: 28px;
	}

	.group-label {
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		color: var(--text-tertiary);
		padding: 0 4px 4px;
	}

	.group-desc {
		font-size: 12.5px;
		color: var(--text-secondary);
		padding: 0 4px 10px;
		line-height: 1.45;
	}

	.group-card {
		background: var(--bg-elevated);
		border-radius: 14px;
		overflow: hidden;
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		container-type: inline-size;
		container-name: settings-card;
	}

	.sync-strip {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto;
		gap: 18px;
		padding: 18px 22px;
		align-items: center;
	}

	.sync-strip-meta {
		display: flex;
		align-items: center;
		gap: 24px;
		flex-wrap: wrap;
	}

	.sync-stat {
		display: flex;
		flex-direction: column;
		gap: 3px;
	}

	.l {
		font-size: 10.5px;
		font-weight: 600;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		color: var(--text-tertiary);
	}

	.v {
		font-size: 14px;
		font-weight: 600;
		color: var(--text-primary);
		display: inline-flex;
		align-items: center;
		gap: 6px;
	}

	.v.muted {
		color: var(--text-secondary);
		font-weight: 500;
	}

	.pulse-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--success);
	}

	.pulse-dot[data-tone='warning'] {
		background: var(--warning);
	}
	.pulse-dot[data-tone='error'] {
		background: var(--destructive);
	}
	.pulse-dot[data-tone='info'] {
		background: var(--accent);
	}

	.btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 8px 14px;
		border-radius: 8px;
		font-family: inherit;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		border: none;
		color: var(--text-primary);
		background: var(--bg-secondary);
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		white-space: nowrap;
	}

	.btn.primary {
		background: var(--accent);
		color: var(--text-on-color);
	}

	.btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.btn svg {
		width: 14px;
		height: 14px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.8;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.spinner {
		width: 12px;
		height: 12px;
		border-radius: 50%;
		border: 2px solid currentColor;
		border-top-color: transparent;
		animation: spin 700ms linear infinite;
	}

	.callout {
		margin: 0 18px 18px;
		padding: 12px 14px;
		border-radius: 10px;
		border: 0.5px solid var(--border-primary);
		background: var(--bg-secondary);
	}

	.callout.error {
		border-color: var(--destructive);
		background: var(--fill-danger);
	}

	.callout.warning {
		border-color: var(--warning);
		background: var(--fill-warning);
	}

	.callout-title {
		font-size: 13px;
		font-weight: 600;
		color: var(--text-primary);
	}

	.callout-detail {
		font-size: 12px;
		color: var(--text-secondary);
		line-height: 1.4;
		margin-top: 3px;
	}

	.callout-actions {
		margin-top: 10px;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	/* The Start Export CTA drops below the stats on a narrow card. */
	@container settings-card (max-width: 539px) {
		.sync-strip {
			grid-template-columns: 1fr;
			gap: 14px;
			padding: 16px;
		}

		.sync-strip-meta {
			gap: 16px;
		}
	}
</style>
