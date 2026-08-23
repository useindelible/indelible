<script lang="ts">
	import type { NotionConnectionDetails, NotionStatusSummary } from './notion-status-model';
	import { t } from '$lib/i18n';

	interface Props {
		details: NotionConnectionDetails;
		summary: NotionStatusSummary;
		onReauthorize: () => void;
		onChangeAccount: () => void;
	}

	let { details, summary, onReauthorize, onChangeAccount }: Props = $props();
	let copiedKey = $state<string | null>(null);

	async function copy(text: string | null, key: string) {
		if (!text) return;
		try {
			await navigator.clipboard.writeText(text);
			copiedKey = key;
			setTimeout(() => {
				if (copiedKey === key) copiedKey = null;
			}, 1400);
		} catch {
			copiedKey = null;
		}
	}
</script>

<section class="group">
	<div class="group-label">{$t('integrations_notion_connection')}</div>
	<div class="group-desc">
		{$t('integrations_notion_connection_description')}
	</div>

	<div class="group-card">
		<div class="conn-card">
			<div class="conn-card-icon">{details.workspaceIcon ?? '📚'}</div>
			<div class="conn-card-id">
				<div class="conn-card-name">
					{details.workspaceName ?? $t('integrations_notion_workspace')}
					{#if summary.connectionState === 'connected' || summary.connectionState === 'syncing'}
						<span class="verified-badge" title={$t('integrations_notion_authorized')}>
							<svg viewBox="0 0 24 24"><path d="M5 13l4 4L19 7" /></svg>
						</span>
					{/if}
				</div>
				<div class="conn-card-meta">
					{#if summary.formattedConnectedOn}
						<span class="since"
							>{$t('integrations_notion_connected_on', {
								values: { date: summary.formattedConnectedOn }
							})}</span
						>
					{/if}
				</div>
			</div>
			<div class="conn-card-actions">
				<button class="btn ghost" type="button" onclick={onReauthorize}>
					<svg viewBox="0 0 24 24"><path d="M21 12a9 9 0 1 1-3.5-7.1" /><path d="M21 4v5h-5" /></svg
					>
					{$t('integrations_notion_reauthorize')}
				</button>
				<button class="btn" type="button" onclick={onChangeAccount}
					>{$t('integrations_notion_change_account')}</button
				>
			</div>
		</div>

		<div class="conn-detail-row">
			<span class="key">{$t('integrations_notion_managed_database')}</span>
			<span class="val" class:empty={!details.databaseId} data-testid="notion-database-id">
				{details.databaseId ?? $t('integrations_notion_provisioned_first_export')}
			</span>
			{#if details.databaseId}
				<button
					class="copy-btn"
					class:copied={copiedKey === 'database'}
					type="button"
					onclick={() => copy(details.databaseId, 'database')}
					aria-label={$t('integrations_notion_copy_database_id')}
				>
					<svg viewBox="0 0 24 24"
						><rect x="9" y="9" width="13" height="13" rx="2" /><path
							d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"
						/></svg
					>
				</button>
			{:else}
				<span></span>
			{/if}
		</div>

		<div class="conn-detail-row">
			<span class="key">{$t('integrations_notion_data_source')}</span>
			<span class="val" class:empty={!details.dataSourceId}>
				{details.dataSourceId ?? $t('integrations_notion_provisioned_first_export')}
			</span>
			{#if details.dataSourceId}
				<button
					class="copy-btn"
					class:copied={copiedKey === 'data_source'}
					type="button"
					onclick={() => copy(details.dataSourceId, 'data_source')}
					aria-label={$t('integrations_notion_copy_data_source_id')}
				>
					<svg viewBox="0 0 24 24"
						><rect x="9" y="9" width="13" height="13" rx="2" /><path
							d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"
						/></svg
					>
				</button>
			{:else}
				<span></span>
			{/if}
		</div>
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

	.conn-card {
		padding: 20px 22px;
		display: grid;
		grid-template-columns: auto 1fr auto;
		gap: 18px;
		align-items: center;
	}

	.conn-card-icon {
		width: 56px;
		height: 56px;
		border-radius: 14px;
		background: var(--bg-elevated);
		box-shadow: var(--shadow-1);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 28px;
		line-height: 1;
		flex-shrink: 0;
	}

	.conn-card-id {
		min-width: 0;
	}

	.conn-card-name {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 17px;
		font-weight: 600;
		color: var(--text-primary);
		margin-bottom: 4px;
	}

	.verified-badge {
		color: var(--success);
		width: 14px;
		height: 14px;
	}

	.verified-badge svg {
		width: 100%;
		height: 100%;
		stroke: currentColor;
		fill: none;
		stroke-width: 2.4;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.conn-card-meta {
		display: flex;
		align-items: center;
		gap: 12px;
		font-size: 12px;
		color: var(--text-secondary);
		flex-wrap: wrap;
	}

	.conn-card-actions {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.conn-detail-row {
		display: grid;
		grid-template-columns: 160px 1fr auto;
		gap: 14px;
		padding: 12px 22px;
		align-items: center;
		border-top: 0.5px solid var(--border-primary);
	}

	.key {
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--text-tertiary);
	}

	.val {
		font-family: 'SF Mono', 'Fira Code', 'Menlo', ui-monospace, monospace;
		font-size: 12px;
		color: var(--text-secondary);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.val.empty {
		font-family: var(--font-sans);
		font-style: italic;
		font-size: 12.5px;
		color: var(--text-quaternary);
	}

	.copy-btn,
	.btn {
		border: none;
		cursor: pointer;
	}

	.copy-btn {
		width: 26px;
		height: 26px;
		border-radius: 7px;
		background: var(--bg-secondary);
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		color: var(--text-tertiary);
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 0;
	}

	.copy-btn svg,
	.btn svg {
		width: 13px;
		height: 13px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.7;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.copy-btn.copied {
		color: var(--success);
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
		color: var(--text-primary);
		background: var(--bg-secondary);
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		white-space: nowrap;
	}

	.btn.ghost {
		background: transparent;
		box-shadow: inset 0 0 0 0.5px var(--border-secondary);
	}

	/* Two fixed action buttons can't share the row with the workspace name on
	   a narrow card: the card stacks, long names clip instead of pushing
	   wide, and the detail keys move above their values. */
	@container settings-card (max-width: 539px) {
		.conn-card {
			grid-template-columns: 1fr;
			gap: 12px;
			padding: 16px;
		}

		.conn-card-id {
			overflow: hidden;
		}

		.conn-card-actions {
			flex-wrap: wrap;
		}

		.conn-detail-row {
			grid-template-columns: minmax(0, 1fr) auto;
			row-gap: 4px;
			padding: 12px 16px;
		}

		.conn-detail-row .key {
			grid-column: 1 / -1;
		}
	}
</style>
