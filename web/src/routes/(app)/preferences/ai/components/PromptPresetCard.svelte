<script lang="ts">
	import type { MilaPromptPresetResponse } from '$lib/api';
	import { t } from '$lib/i18n';

	interface Props {
		expanded: boolean;
		preset: MilaPromptPresetResponse;
		onDelete: (id: string) => void;
		onEdit: (preset: MilaPromptPresetResponse) => void;
		onToggle: (id: string | null | undefined) => void;
	}

	let { expanded, preset, onDelete, onEdit, onToggle }: Props = $props();
</script>

<div class="preset-item" class:expanded>
	<div
		class="preset-summary"
		role="button"
		tabindex="0"
		aria-expanded={expanded}
		onclick={() => onToggle(preset.id)}
		onkeydown={(event) => {
			if (event.key === 'Enter' || event.key === ' ') {
				event.preventDefault();
				onToggle(preset.id);
			}
		}}
	>
		<span class="preset-chevron">›</span>
		<span class="preset-name">{preset.name}</span>
		<span class="preset-prompt-snip">{preset.system_prompt}</span>
		<span class="preset-badges">
			{#if preset.is_default}<span class="badge default">{$t('prefs_ai_default')}</span>{/if}
			{#if preset.is_built_in}<span class="badge built-in">{$t('prefs_ai_built_in')}</span>{/if}
		</span>
		{#if !preset.is_built_in}
			<div class="preset-actions-row">
				<button
					type="button"
					class="icon-btn"
					aria-label={$t('prefs_ai_edit_preset')}
					onclick={(event) => {
						event.stopPropagation();
						onEdit(preset);
					}}
				>
					{$t('prefs_ai_edit')}
				</button>
				<button
					type="button"
					class="icon-btn danger"
					aria-label={$t('prefs_ai_delete_preset')}
					onclick={(event) => {
						event.stopPropagation();
						if (preset.id) onDelete(preset.id);
					}}
				>
					{$t('common_delete')}
				</button>
			</div>
		{/if}
	</div>
	<div class="preset-detail">
		<div class="preset-detail-inner">
			<div class="preset-detail-label">{$t('prefs_ai_system_prompt')}</div>
			<pre class="preset-detail-prompt">{preset.system_prompt}</pre>
		</div>
	</div>
</div>

<style>
	.preset-item {
		border-top: 1px solid var(--border-hairline);
	}
	.preset-summary {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 11px 18px;
		cursor: pointer;
		user-select: none;
	}
	.preset-summary:hover,
	.preset-item.expanded .preset-summary {
		background: var(--fill-hover);
	}
	.preset-chevron {
		color: var(--text-tertiary);
		font-size: 18px;
		transition: transform 180ms;
	}
	.preset-item.expanded .preset-chevron {
		transform: rotate(90deg);
		color: var(--mila-violet);
	}
	.preset-name {
		font-size: 13.5px;
		font-weight: 500;
		color: var(--text-primary);
		white-space: nowrap;
	}
	.preset-prompt-snip {
		flex: 1;
		min-width: 0;
		font-family: var(--font-mono, ui-monospace, monospace);
		font-size: 12px;
		color: var(--text-secondary);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.preset-badges,
	.preset-actions-row {
		display: inline-flex;
		gap: 6px;
		flex-shrink: 0;
	}
	.badge {
		padding: 2px 8px;
		border-radius: 980px;
		font-size: 10.5px;
		font-weight: 600;
	}
	.badge.default {
		background: var(--mila-violet-soft);
		color: var(--mila-violet);
	}
	.badge.built-in {
		background: var(--bg-tertiary);
		color: var(--text-secondary);
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
	}
	.icon-btn {
		border: 0;
		border-radius: 7px;
		background: transparent;
		color: var(--text-tertiary);
		cursor: pointer;
		font-size: 11.5px;
		padding: 5px 7px;
	}
	.icon-btn:hover {
		background: var(--fill-hover);
		color: var(--text-primary);
	}
	.icon-btn.danger:hover {
		color: var(--mila-status-err-text);
		background: var(--mila-status-err-bg);
	}
	.preset-detail {
		display: none;
	}
	.preset-item.expanded .preset-detail {
		display: block;
	}
	.preset-detail-inner {
		padding: 4px 18px 16px 47px;
	}
	.preset-detail-label {
		font-size: 10.5px;
		font-weight: 600;
		text-transform: uppercase;
		color: var(--text-tertiary);
		margin-bottom: 8px;
	}
	.preset-detail-prompt {
		font-family: var(--font-mono, ui-monospace, monospace);
		font-size: 12.5px;
		line-height: 1.55;
		color: var(--text-primary);
		background: var(--bg-secondary);
		border-radius: 10px;
		padding: 12px 14px;
		white-space: pre-wrap;
		word-break: break-word;
		margin: 0;
		box-shadow: inset 0 0 0 0.5px var(--border-hairline);
	}
</style>
