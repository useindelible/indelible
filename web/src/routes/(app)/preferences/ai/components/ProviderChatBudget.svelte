<script lang="ts">
	import { number, t } from '$lib/i18n';

	interface Props {
		modelContextWindow: number;
		chatContextPct: number;
		onWindowChange: (value: number) => void;
		onPctChange: (value: number) => void;
	}

	let { modelContextWindow, chatContextPct, onWindowChange, onPctChange }: Props = $props();

	const uid = $props.id();
	const windowId = `${uid}-window`;
	const pctId = `${uid}-pct`;

	const inlineTokens = $derived(
		Math.round((Math.max(modelContextWindow, 0) * Math.min(Math.max(chatContextPct, 0), 100)) / 100)
	);
</script>

<div class="section budget">
	<div class="section-title">{$t('prefs_ai_chat_budget')}</div>
	<div class="budget-fields">
		<div class="form-group">
			<label class="form-label" for={windowId}>{$t('prefs_ai_context_window')}</label>
			<input
				id={windowId}
				class="form-input"
				type="number"
				min="1"
				required
				value={modelContextWindow}
				placeholder="16000"
				oninput={(event) => onWindowChange(Number(event.currentTarget.value))}
			/>
			<span class="field-hint">
				{$t('prefs_ai_context_window_hint')}
			</span>
		</div>
		<div class="form-group">
			<label class="form-label" for={pctId}>{$t('prefs_ai_inline_context')}</label>
			<input
				id={pctId}
				class="form-input"
				type="number"
				min="1"
				max="100"
				value={chatContextPct}
				oninput={(event) => onPctChange(Number(event.currentTarget.value))}
			/>
			<span class="field-hint">
				{$t('prefs_ai_inline_context_hint')}
			</span>
		</div>
	</div>
	<div class="readout">
		<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M3 12h4l3 8 4-16 3 8h4" /></svg>
		<span>
			{#if inlineTokens > 0}
				{$t('prefs_ai_budget_readout', { values: { tokens: $number(inlineTokens) } })}
			{:else}
				{$t('prefs_ai_budget_zero')}
			{/if}
		</span>
	</div>
</div>

<style>
	.section {
		position: relative;
		grid-column: 1 / -1;
		display: flex;
		flex-direction: column;
		gap: 14px;
		min-width: 0;
	}
	.section-title {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 11px;
		font-weight: 700;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		color: var(--text-secondary);
	}
	.section-title::before {
		content: '';
		width: 2px;
		height: 12px;
		border-radius: 2px;
		flex-shrink: 0;
		background: var(--mila-action-entities);
	}
	.budget-fields {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 14px 18px;
	}
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
	}
	.form-input {
		width: 100%;
		box-sizing: border-box;
		padding: 10px 12px;
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
	.field-hint {
		font-size: 11.5px;
		line-height: 1.4;
		letter-spacing: -0.005em;
		color: var(--text-tertiary);
	}
	.readout {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 12px;
		border-radius: 10px;
		background: var(--mila-violet-soft);
		color: var(--mila-violet);
		font-size: 12.5px;
		letter-spacing: -0.005em;
	}
	.readout svg {
		width: 13px;
		height: 13px;
		flex-shrink: 0;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.8;
		stroke-linecap: round;
		stroke-linejoin: round;
	}
	@media (max-width: 640px) {
		.budget-fields {
			grid-template-columns: 1fr;
		}
	}
</style>
