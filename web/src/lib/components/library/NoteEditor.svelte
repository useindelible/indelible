<script lang="ts">
	import type { Snippet } from 'svelte';
	import { t } from '$lib/i18n';

	interface Props {
		value: string;
		placeholder: string;
		rows: number;
		saving: boolean;
		error?: string;
		focus?: boolean;
		onSave: () => void;
		onCancel: () => void;
		onDelete?: () => void;
		children?: Snippet;
	}

	let {
		value = $bindable(),
		placeholder,
		rows,
		saving,
		error = '',
		focus = false,
		onSave,
		onCancel,
		onDelete,
		children
	}: Props = $props();

	let input = $state<HTMLTextAreaElement | undefined>(undefined);

	$effect(() => {
		if (focus && input) input.focus();
	});
</script>

<div class="add-note-form">
	<textarea
		bind:this={input}
		bind:value
		class="add-note-input"
		{placeholder}
		{rows}
		onkeydown={(e) => {
			if (e.key === 'Escape') onCancel();
		}}
	></textarea>
	{#if error}
		<p class="note-error">{error}</p>
	{/if}
	{@render children?.()}
	<div class="add-note-actions">
		{#if onDelete}
			<button type="button" class="note-delete" onclick={onDelete} disabled={saving}>
				{$t('common_delete')}
			</button>
		{/if}
		<button type="button" class="note-cancel" onclick={onCancel} disabled={saving}>
			{$t('common_cancel')}
		</button>
		<button type="button" class="note-save" disabled={saving || !value.trim()} onclick={onSave}>
			{saving ? $t('common_saving') : $t('common_save')}
		</button>
	</div>
</div>

<style>
	.add-note-form {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.add-note-input {
		width: 100%;
		padding: 10px;
		border-radius: 8px;
		border: 1.5px solid var(--accent);
		background: var(--fill-hover);
		font-size: 13px;
		font-family: var(--font-sans);
		color: var(--text-primary);
		line-height: 1.55;
		resize: none;
		outline: none;
		box-sizing: border-box;
	}

	.note-error {
		font-size: 12px;
		color: var(--destructive);
		font-family: var(--font-sans);
		margin: 0 0 4px;
	}

	.add-note-actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
	}

	.note-delete {
		font-size: 13px;
		font-family: var(--font-sans);
		color: var(--destructive);
		background: none;
		border: none;
		cursor: pointer;
		padding: 4px 10px;
		margin-right: auto;
	}

	.note-delete:hover:not(:disabled) {
		opacity: 0.8;
	}

	.note-delete:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.note-cancel {
		font-size: 13px;
		font-family: var(--font-sans);
		color: var(--text-secondary);
		background: none;
		border: none;
		cursor: pointer;
		padding: 4px 10px;
	}

	.note-cancel:hover {
		color: var(--text-primary);
	}

	.note-save {
		font-size: 13px;
		font-family: var(--font-sans);
		font-weight: 500;
		color: var(--text-on-color);
		background: var(--accent);
		border: none;
		border-radius: 6px;
		cursor: pointer;
		padding: 4px 14px;
		transition: opacity 120ms ease;
	}

	.note-save:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}
</style>
