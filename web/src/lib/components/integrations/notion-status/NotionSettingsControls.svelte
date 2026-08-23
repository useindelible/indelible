<script lang="ts">
	import type { NotionSettingsDto, UpdateNotionSettingsRequest } from '$lib/api';
	import { t, type MessageKey } from '$lib/i18n';

	interface Props {
		settings?: NotionSettingsDto | null;
		savingSetting?: keyof UpdateNotionSettingsRequest | null;
		onSettingChange: (key: keyof UpdateNotionSettingsRequest, value: boolean) => void;
	}

	let { settings = null, savingSetting = null, onSettingChange }: Props = $props();

	const rows: Array<{
		labelKey: MessageKey;
		hintKey: MessageKey;
		key: keyof UpdateNotionSettingsRequest;
	}> = [
		{
			labelKey: 'integrations_notion_export_automatically',
			hintKey: 'integrations_notion_export_automatically_hint',
			key: 'export_automatically'
		},
		{
			labelKey: 'integrations_notion_include_locations',
			hintKey: 'integrations_notion_include_locations_hint',
			key: 'include_highlight_locations'
		},
		{
			labelKey: 'integrations_notion_compact_layout',
			hintKey: 'integrations_notion_compact_layout_hint',
			key: 'compact_layout'
		},
		{
			labelKey: 'integrations_notion_select_items',
			hintKey: 'integrations_notion_select_items_hint',
			key: 'selection_enabled'
		}
	];

	function settingValue(key: keyof UpdateNotionSettingsRequest): boolean {
		return settings?.[key] ?? false;
	}
</script>

<section class="group">
	<div class="group-label">{$t('integrations_notion_export_settings')}</div>
	<div class="group-desc">
		{$t('integrations_notion_export_settings_description')}
	</div>

	<div class="group-card">
		{#each rows as row (row.key)}
			<div class="row">
				<div class="label-block">
					<div class="label">{$t(row.labelKey)}</div>
					<div class="hint">{$t(row.hintKey)}</div>
				</div>
				<button
					type="button"
					class="toggle"
					class:on={settingValue(row.key)}
					class:locked={savingSetting === row.key}
					role="switch"
					aria-checked={settingValue(row.key)}
					aria-label={$t(row.labelKey)}
					disabled={savingSetting === row.key}
					onclick={() => onSettingChange(row.key, !settingValue(row.key))}
				></button>
			</div>
		{/each}
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
	}

	.row {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 14px 18px;
		min-height: 52px;
		border-top: 0.5px solid var(--border-primary);
	}

	.row:first-child {
		border-top: none;
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

	.toggle {
		width: 38px;
		height: 22px;
		border-radius: 980px;
		background: var(--fill-selected);
		border: none;
		padding: 2px;
		display: flex;
		align-items: center;
		cursor: pointer;
		transition: background 160ms ease;
	}

	.toggle::after {
		content: '';
		width: 18px;
		height: 18px;
		border-radius: 50%;
		background: var(--text-on-color);
		box-shadow: var(--shadow-1);
		transition: transform 160ms ease;
	}

	.toggle.on {
		background: var(--accent);
	}

	.toggle.on::after {
		transform: translateX(16px);
	}

	.toggle.locked {
		opacity: 0.6;
		cursor: progress;
	}
</style>
