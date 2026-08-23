<script lang="ts">
	import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';
	import type { MilaPromptPresetResponse, MilaPromptPresetsResponse } from '$lib/api';
	import {
		presetsForAction,
		type ActionKey,
		type ActionMeta,
		type PresetEditorState
	} from '../mila-settings-model';
	import PromptPresetCard from './PromptPresetCard.svelte';
	import PromptPresetEditor from './PromptPresetEditor.svelte';
	import { t } from '$lib/i18n';

	interface Props {
		actions: ActionMeta[];
		editorSaving: boolean;
		editorState: PresetEditorState | null;
		expandedPresetId: string | null;
		presets: MilaPromptPresetsResponse | null;
		onAdd: (action: ActionKey) => void;
		onCancelEditor: () => void;
		onDelete: (id: string) => void;
		onEdit: (action: ActionKey, preset: MilaPromptPresetResponse) => void;
		onEditorChange: (patch: Partial<PresetEditorState>) => void;
		onSaveEditor: () => void;
		onTogglePreset: (id: string | null | undefined) => void;
	}

	let {
		actions,
		editorSaving,
		editorState,
		expandedPresetId,
		presets,
		onAdd,
		onCancelEditor,
		onDelete,
		onEdit,
		onEditorChange,
		onSaveEditor,
		onTogglePreset
	}: Props = $props();
</script>

<SettingsGroup title={$t('prefs_ai_prompt_presets')} meta={$t('prefs_ai_prompt_presets_hint')}>
	{#each actions as action (action.key)}
		{@const list = presetsForAction(presets, action.key)}
		<section class="preset-action-section">
			<div class="preset-action-head">
				<div class="preset-action-icon {action.key}"></div>
				<div class="preset-action-meta">
					<div class="preset-action-name">{$t(action.nameKey)}</div>
					<div class="preset-action-desc">{$t(action.descKey)}</div>
				</div>
				<div class="preset-action-count">
					{$t('prefs_ai_preset_count', { values: { count: list.length } })}
				</div>
			</div>

			<div class="preset-list">
				{#if list.length === 0 && editorState?.action !== action.key}
					<div class="preset-empty">{$t('prefs_ai_no_presets')}</div>
				{:else}
					{#each list as preset (preset.id ?? preset.name)}
						<PromptPresetCard
							{preset}
							expanded={expandedPresetId === preset.id}
							{onDelete}
							onEdit={(item) => onEdit(action.key, item)}
							onToggle={onTogglePreset}
						/>
					{/each}
				{/if}

				{#if editorState?.action === action.key}
					<PromptPresetEditor
						action={action.key}
						editor={editorState}
						{editorSaving}
						onCancel={onCancelEditor}
						onChange={onEditorChange}
						onSave={onSaveEditor}
					/>
				{:else}
					<button type="button" class="add-preset-row" onclick={() => onAdd(action.key)}>
						{$t('prefs_ai_add_action_preset', {
							values: { action: action.key }
						})}
					</button>
				{/if}
			</div>
		</section>
	{/each}
</SettingsGroup>

<style>
	.preset-action-section {
		background: var(--bg-elevated);
		border-radius: 16px;
		box-shadow: var(--shadow-1);
		margin-bottom: 12px;
		overflow: hidden;
	}
	.preset-action-head {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 14px 18px;
		border-bottom: 1px solid var(--border-hairline);
		background: linear-gradient(135deg, transparent 0%, var(--mila-violet-soft) 100%);
	}
	.preset-action-icon {
		width: 28px;
		height: 28px;
		border-radius: 8px;
		flex-shrink: 0;
		background: var(--mila-violet);
	}
	.preset-action-icon.summary {
		background: var(--mila-action-summary);
	}
	.preset-action-icon.tags {
		background: var(--mila-action-tags);
	}
	.preset-action-icon.entities {
		background: var(--mila-action-entities);
	}
	.preset-action-icon.chat {
		background: var(--mila-action-chat);
	}
	.preset-action-icon.custom {
		background: var(--mila-action-custom);
	}
	.preset-action-meta {
		flex: 1;
		min-width: 0;
	}
	.preset-action-name {
		font-size: 14px;
		font-weight: 600;
		color: var(--text-primary);
	}
	.preset-action-desc,
	.preset-action-count,
	.preset-empty {
		font-size: 12px;
		color: var(--text-secondary);
	}
	.preset-action-count {
		color: var(--text-tertiary);
		white-space: nowrap;
	}
	.preset-list {
		padding: 4px 0;
	}
	.preset-empty {
		padding: 14px 18px;
		font-style: italic;
	}
	.add-preset-row {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 11px 18px;
		border-top: 1px solid var(--border-hairline);
		color: var(--mila-violet);
		font-size: 12.5px;
		font-weight: 600;
		width: 100%;
		text-align: left;
		background: transparent;
		border-left: 0;
		border-right: 0;
		border-bottom: 0;
		cursor: pointer;
	}
	.add-preset-row:hover {
		background: var(--mila-violet-soft);
	}
</style>
