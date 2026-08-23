<script lang="ts">
	import { t } from '$lib/i18n';

	interface Props {
		onDisconnect: () => void;
	}

	let { onDisconnect }: Props = $props();
</script>

<section class="group">
	<div class="group-label">{$t('integrations_notion_danger_zone')}</div>
	<div class="group-card danger">
		<div class="danger-card">
			<div class="label-block">
				<div class="label">{$t('integrations_notion_disconnect_title')}</div>
				<div class="hint">
					{$t('integrations_notion_disconnect_hint')}
				</div>
			</div>
			<button class="btn danger" type="button" onclick={onDisconnect}
				>{$t('integrations_disconnect')}</button
			>
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

	.group-card {
		background: var(--bg-elevated);
		border-radius: 14px;
		overflow: hidden;
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		container-type: inline-size;
		container-name: settings-card;
	}

	.group-card.danger {
		box-shadow: inset 0 0 0 0.5px var(--destructive);
	}

	.danger-card {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 18px;
		padding: 16px 18px;
	}

	.label-block {
		flex: 1;
		min-width: 0;
	}

	.label {
		font-size: 13px;
		font-weight: 500;
		color: var(--text-primary);
		margin-bottom: 2px;
	}

	.hint {
		font-size: 12px;
		color: var(--text-secondary);
		line-height: 1.4;
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

	.btn.danger {
		color: var(--destructive);
		background: var(--fill-danger);
		box-shadow: inset 0 0 0 0.5px var(--destructive);
	}

	@container settings-card (max-width: 539px) {
		.danger-card {
			flex-direction: column;
			align-items: flex-start;
			gap: 12px;
			padding: 16px;
		}
	}
</style>
