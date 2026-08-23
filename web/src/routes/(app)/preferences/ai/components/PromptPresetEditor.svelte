<script lang="ts">
	import type { ActionKey, PresetEditorState } from '../mila-settings-model';
	import { t } from '$lib/i18n';

	interface Props {
		action: ActionKey;
		editor: PresetEditorState;
		editorSaving: boolean;
		onCancel: () => void;
		onChange: (patch: Partial<PresetEditorState>) => void;
		onSave: () => void;
	}

	let { action, editor, editorSaving, onCancel, onChange, onSave }: Props = $props();
</script>

<div class="preset-edit-form">
	<div class="preset-edit-header">
		<div class="preset-edit-title">
			<span class="preset-edit-title-dot"></span>
			{$t(editor.mode === 'add' ? 'prefs_ai_new_preset_title' : 'prefs_ai_edit_preset_title', {
				values: { action }
			})}
		</div>
		<button
			type="button"
			class="preset-edit-close"
			aria-label={$t('common_discard')}
			onclick={onCancel}
		>
			<svg viewBox="0 0 24 24" aria-hidden="true">
				<path d="M6 6l12 12M18 6L6 18" />
			</svg>
		</button>
	</div>
	<div class="preset-edit-body">
		<div class="preset-edit-field">
			<label class="preset-edit-field-label" for="preset-edit-name"
				>{$t('prefs_ai_preset_name')}</label
			>
			<input
				id="preset-edit-name"
				class="preset-edit-name-input"
				value={editor.name}
				placeholder={$t('prefs_ai_preset_name_placeholder')}
				oninput={(event) => onChange({ name: event.currentTarget.value })}
			/>
		</div>
		<div class="preset-edit-field">
			<label class="preset-edit-field-label" for="preset-edit-prompt"
				>{$t('prefs_ai_system_prompt')}</label
			>
			<textarea
				id="preset-edit-prompt"
				class="preset-edit-prompt-textarea"
				value={editor.system_prompt}
				rows={6}
				placeholder={$t('prefs_ai_system_prompt_placeholder')}
				oninput={(event) => onChange({ system_prompt: event.currentTarget.value })}
			></textarea>
		</div>
		<div class="preset-edit-default-row">
			<div class="preset-edit-default-text">
				<div class="preset-edit-default-title">
					{$t('prefs_ai_set_default_for', { values: { action } })}
				</div>
				<div class="preset-edit-default-sub">
					{$t('prefs_ai_set_default_hint', {
						values: { action }
					})}
				</div>
			</div>
			<button
				type="button"
				class="toggle"
				class:on={editor.is_default}
				role="switch"
				aria-checked={editor.is_default}
				aria-label={$t(editor.is_default ? 'prefs_ai_unset_default' : 'prefs_ai_set_default')}
				onclick={() => onChange({ is_default: !editor.is_default })}
			>
				<span class="toggle-track"></span>
			</button>
		</div>
	</div>
	<div class="preset-edit-footer">
		<button type="button" class="btn ghost compact" onclick={onCancel}>{$t('common_cancel')}</button
		>
		<button
			type="button"
			class="btn violet compact"
			onclick={onSave}
			disabled={editorSaving || !editor.name.trim()}
		>
			{editorSaving
				? $t('common_saving')
				: editor.mode === 'add'
					? $t('prefs_ai_add_preset')
					: $t('prefs_ai_save_changes')}
		</button>
	</div>
</div>

<style>
	.preset-edit-form {
		margin: 8px 14px 14px;
		background: var(--bg-elevated);
		border-radius: 14px;
		overflow: hidden;
		display: flex;
		flex-direction: column;
		box-shadow:
			0 0 0 1px var(--mila-violet-soft),
			0 8px 28px -10px rgba(132, 66, 217, 0.22),
			var(--shadow-1);
	}
	.preset-edit-header,
	.preset-edit-footer {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 14px 12px 16px;
		background: var(--bg-secondary);
		border-bottom: 1px solid var(--border-hairline);
	}
	.preset-edit-footer {
		justify-content: flex-end;
		gap: 8px;
		border-top: 1px solid var(--border-hairline);
		border-bottom: 0;
	}
	.preset-edit-title {
		display: inline-flex;
		align-items: center;
		gap: 9px;
		font-size: 12.5px;
		font-weight: 600;
		color: var(--text-primary);
		letter-spacing: 0;
	}
	.preset-edit-title-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--mila-violet);
		box-shadow: 0 0 0 3px var(--mila-violet-soft);
	}
	.preset-edit-close,
	.toggle,
	.btn {
		border: 0;
		cursor: pointer;
	}
	.preset-edit-close {
		width: 26px;
		height: 26px;
		border-radius: 7px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		color: var(--text-tertiary);
		background: transparent;
	}
	.preset-edit-close svg {
		width: 12px;
		height: 12px;
		stroke: currentColor;
		fill: none;
		stroke-width: 2;
	}
	.preset-edit-body {
		padding: 16px;
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	.preset-edit-field {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.preset-edit-field-label {
		font-size: 10.5px;
		font-weight: 600;
		letter-spacing: 0;
		text-transform: uppercase;
		color: var(--text-tertiary);
	}
	.preset-edit-name-input,
	.preset-edit-prompt-textarea {
		border-radius: 9px;
		background: var(--input-bg);
		box-shadow: var(--mila-input-shadow);
		color: var(--text-primary);
		border: 0;
		outline: none;
		letter-spacing: 0;
	}
	.preset-edit-name-input {
		padding: 10px 12px;
		font-size: 13.5px;
	}
	.preset-edit-prompt-textarea {
		padding: 12px 14px;
		font-family: var(--font-mono, ui-monospace, monospace);
		font-size: 12.5px;
		line-height: 1.55;
		resize: vertical;
		min-height: 140px;
	}
	.preset-edit-default-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 14px;
		padding: 12px 14px;
		border-radius: 10px;
		background: var(--bg-secondary);
		box-shadow: inset 0 0 0 0.5px var(--border-hairline);
	}
	.preset-edit-default-title {
		font-size: 13px;
		font-weight: 500;
		color: var(--text-primary);
	}
	.preset-edit-default-sub {
		font-size: 11.5px;
		color: var(--text-secondary);
	}
	.toggle {
		background: transparent;
		padding: 0;
	}
	.toggle-track {
		width: 36px;
		height: 21px;
		border-radius: 980px;
		background: var(--mila-status-idle-bg);
		position: relative;
		display: block;
	}
	.toggle-track::after {
		content: '';
		position: absolute;
		left: 2px;
		top: 2px;
		width: 17px;
		height: 17px;
		border-radius: 50%;
		background: var(--bg-primary);
		box-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
	}
	.toggle.on .toggle-track {
		background: var(--mila-violet);
	}
	.toggle.on .toggle-track::after {
		left: 17px;
	}
	.btn {
		border-radius: 9px;
		font-size: 11.5px;
		font-weight: 500;
		padding: 5px 10px;
	}
	.btn.ghost {
		background: transparent;
		color: var(--text-primary);
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
	}
	.btn.violet {
		background: var(--mila-violet);
		color: var(--text-on-color);
	}
	.btn:disabled {
		opacity: 0.45;
		cursor: default;
	}
</style>
