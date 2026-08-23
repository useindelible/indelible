<script lang="ts">
	import CheckRow from './CheckRow.svelte';
	import { t } from '$lib/i18n';

	interface Props {
		label: string;
		value: string;
		show: boolean;
		hasStoredKey: boolean;
		emptyHint: string;
		clearLabel: string;
		clear: boolean;
		onValueChange: (value: string) => void;
		onToggleShow: () => void;
		onClearChange: (clear: boolean) => void;
	}

	let {
		label,
		value,
		show,
		hasStoredKey,
		emptyHint,
		clearLabel,
		clear,
		onValueChange,
		onToggleShow,
		onClearChange
	}: Props = $props();

	const fieldId = $props.id();
	const typed = $derived(value.trim().length > 0);
	const hint = $derived(
		hasStoredKey
			? typed
				? $t('prefs_ai_key_replaces_saved')
				: $t('prefs_ai_key_already_configured')
			: typed
				? $t('prefs_ai_key_new')
				: emptyHint
	);
	// Removing a saved key only makes sense when no replacement has been typed.
	const canClear = $derived(hasStoredKey && !typed);
</script>

<div class="form-group">
	<label class="form-label" for={fieldId}>
		{label}
		<span class="hint">{hint}</span>
	</label>
	<div class="password-input-wrap">
		<input
			id={fieldId}
			class="form-input"
			type={show ? 'text' : 'password'}
			{value}
			placeholder="sk-..."
			oninput={(event) => onValueChange(event.currentTarget.value)}
		/>
		<button
			type="button"
			class="eye-btn"
			aria-label={$t(show ? 'prefs_ai_hide_field' : 'prefs_ai_show_field', {
				values: { label }
			})}
			onclick={onToggleShow}
		>
			{#if show}
				<svg viewBox="0 0 24 24">
					<path d="M1 12s4-7 11-7 11 7 11 7-4 7-11 7S1 12 1 12z" />
					<circle cx="12" cy="12" r="3" />
					<line x1="3" y1="3" x2="21" y2="21" />
				</svg>
			{:else}
				<svg viewBox="0 0 24 24">
					<path d="M1 12s4-7 11-7 11 7 11 7-4 7-11 7S1 12 1 12z" />
					<circle cx="12" cy="12" r="3" />
				</svg>
			{/if}
		</button>
	</div>
	{#if canClear}
		<CheckRow checked={clear} label={clearLabel} subtle onChange={onClearChange} />
	{/if}
</div>

<style>
	.form-group {
		display: flex;
		flex-direction: column;
		gap: 6px;
		min-width: 0;
	}
	.form-label {
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--text-tertiary);
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 10px;
	}
	.hint {
		font-size: 11.5px;
		font-weight: 500;
		text-transform: none;
		letter-spacing: -0.005em;
		color: var(--text-tertiary);
		text-align: right;
	}
	.form-input {
		width: 100%;
		box-sizing: border-box;
		padding: 10px 40px 10px 12px;
		border-radius: 10px;
		background: var(--input-bg);
		box-shadow: var(--mila-input-shadow);
		border: 0;
		font-size: 13.5px;
		letter-spacing: -0.005em;
		color: var(--text-primary);
		outline: none;
		transition: box-shadow 150ms;
	}
	.form-input:focus {
		box-shadow:
			inset 0 0 0 1.5px var(--mila-violet),
			0 0 0 4px var(--mila-violet-soft);
	}
	.form-input::placeholder {
		color: var(--text-tertiary);
	}
	.password-input-wrap {
		position: relative;
		display: flex;
		align-items: center;
	}
	.eye-btn {
		position: absolute;
		right: 6px;
		width: 30px;
		height: 30px;
		border: 0;
		border-radius: 8px;
		background: transparent;
		color: var(--text-tertiary);
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: pointer;
		transition:
			background 120ms,
			color 120ms;
	}
	.eye-btn:hover {
		background: var(--fill-hover);
		color: var(--text-primary);
	}
	.eye-btn svg {
		width: 14px;
		height: 14px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.7;
		stroke-linecap: round;
		stroke-linejoin: round;
	}
</style>
