<script lang="ts">
	import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';
	import { t } from '$lib/i18n';
	import type { TriageModeDto } from '$lib/api';

	interface Props {
		triageMode: TriageModeDto;
		autoAdvance: boolean;
		onTriageModeChange: (value: TriageModeDto) => void;
		onAutoAdvanceChange: (value: boolean) => void;
	}

	let { triageMode, autoAdvance, onTriageModeChange, onAutoAdvanceChange }: Props = $props();
</script>

<SettingsGroup title={$t('prefs_reading_triage_workflow')}>
	<div class="group-card">
		<div class="row with-stack">
			<div class="label-block">
				<div class="label">{$t('prefs_reading_triage_mode')}</div>
				<div class="hint">{$t('prefs_reading_triage_mode_hint')}</div>
			</div>
			<div class="radio-stack" role="radiogroup" aria-label={$t('prefs_reading_triage_mode')}>
				<button
					type="button"
					class="radio-card"
					class:selected={triageMode === 'focus'}
					role="radio"
					aria-checked={triageMode === 'focus'}
					onclick={() => onTriageModeChange('focus')}
				>
					<span class="radio-check" aria-hidden="true">
						<svg viewBox="0 0 24 24">
							<polyline points="20 6 9 17 4 12" />
						</svg>
					</span>
					<div class="radio-flow" aria-hidden="true">
						<span class="node"></span><span class="arrow"></span><span class="node"></span><span
							class="arrow"
						></span><span class="node"></span>
					</div>
					<div class="radio-meta">
						<div class="radio-label">{$t('prefs_reading_triage')}</div>
						<div class="radio-sub">{$t('prefs_reading_triage_flow')}</div>
					</div>
				</button>
				<button
					type="button"
					class="radio-card"
					class:selected={triageMode === 'manual'}
					role="radio"
					aria-checked={triageMode === 'manual'}
					onclick={() => onTriageModeChange('manual')}
				>
					<span class="radio-check" aria-hidden="true">
						<svg viewBox="0 0 24 24">
							<polyline points="20 6 9 17 4 12" />
						</svg>
					</span>
					<div class="radio-flow" aria-hidden="true">
						<span class="node"></span><span class="arrow"></span><span class="node"></span>
					</div>
					<div class="radio-meta">
						<div class="radio-label">{$t('prefs_reading_simple')}</div>
						<div class="radio-sub">{$t('prefs_reading_simple_flow')}</div>
					</div>
				</button>
			</div>
		</div>
		<div class="row">
			<div class="label-block">
				<div class="label" id="auto-advance-label">{$t('prefs_reading_auto_advance')}</div>
				<div class="hint">{$t('prefs_reading_auto_advance_hint')}</div>
			</div>
			<label class="toggle">
				<input
					type="checkbox"
					aria-labelledby="auto-advance-label"
					checked={autoAdvance}
					onchange={(event) => onAutoAdvanceChange(event.currentTarget.checked)}
				/>
				<span class="toggle-track"></span>
			</label>
		</div>
	</div>
</SettingsGroup>

<style>
	.group-card {
		background: var(--card-bg);
		border-radius: 14px;
		overflow: hidden;
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		container-type: inline-size;
		container-name: settings-card;
	}

	.row {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 14px 18px;
		min-height: 52px;
		border-top: 0.5px solid var(--border-hairline);
	}

	.row:first-child {
		border-top: none;
	}

	.row.with-stack {
		align-items: flex-start;
		flex-direction: column;
		gap: 14px;
		padding: 18px;
	}

	.label-block {
		flex: 1;
		min-width: 0;
	}

	.with-stack .label-block {
		width: 100%;
	}

	.label {
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 500;
		color: var(--text-primary);
		margin-bottom: 2px;
	}

	.hint {
		font-family: var(--font-sans);
		font-size: 12px;
		color: var(--text-secondary);
		line-height: 1.4;
	}

	.radio-stack {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 12px;
		width: 100%;
		max-width: 540px;
	}

	.radio-card {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 10px;
		padding: 14px 16px;
		border-radius: 12px;
		background: var(--bg-elevated);
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		cursor: pointer;
		transition:
			box-shadow 140ms,
			background 140ms,
			transform 140ms;
		position: relative;
		text-align: left;
		border: none;
		font-family: var(--font-sans);
	}

	.radio-card:hover {
		background: var(--fill-hover);
		transform: translateY(-1px);
	}

	.radio-card.selected {
		box-shadow:
			inset 0 0 0 1.5px var(--accent),
			0 0 0 4px var(--accent-soft);
		background: var(--bg-elevated);
	}

	.radio-flow {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		height: 14px;
	}

	.node {
		width: 7px;
		height: 7px;
		border-radius: 50%;
		background: var(--text-quaternary);
		transition: background 160ms;
	}

	.arrow {
		width: 16px;
		height: 1.5px;
		background: var(--text-quaternary);
		border-radius: 1px;
	}

	.radio-card.selected .node,
	.radio-card.selected .arrow {
		background: var(--accent);
	}

	.radio-check {
		position: absolute;
		top: 12px;
		right: 12px;
		width: 18px;
		height: 18px;
		border-radius: 50%;
		background: var(--accent);
		color: var(--text-on-color);
		display: none;
		align-items: center;
		justify-content: center;
		box-shadow: var(--shadow-1);
	}

	.radio-check svg {
		width: 10px;
		height: 10px;
		stroke: currentColor;
		fill: none;
		stroke-width: 2.5;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.radio-card.selected .radio-check {
		display: inline-flex;
	}

	.radio-label {
		font-size: 14px;
		font-weight: 600;
		color: var(--text-primary);
		margin-bottom: 3px;
	}

	.radio-sub {
		font-size: 12px;
		color: var(--text-secondary);
		line-height: 1.42;
	}

	.toggle {
		display: inline-flex;
		align-items: center;
		cursor: pointer;
		flex-shrink: 0;
		position: relative;
	}

	.toggle input {
		position: absolute;
		opacity: 0;
		pointer-events: none;
	}

	.toggle-track {
		width: 36px;
		height: 21px;
		border-radius: 980px;
		background: var(--fill-selected);
		position: relative;
		transition: background 160ms;
	}

	.toggle-track::after {
		content: '';
		position: absolute;
		left: 2px;
		top: 2px;
		width: 17px;
		height: 17px;
		border-radius: 50%;
		background: var(--text-on-color);
		box-shadow: var(--shadow-1);
		transition: left 180ms;
	}

	.toggle input:checked + .toggle-track {
		background: var(--accent);
	}

	.toggle input:checked + .toggle-track::after {
		left: 17px;
	}

	/* Two-up radio cards get too cramped for their descriptions. */
	@container settings-card (max-width: 499px) {
		.radio-stack {
			grid-template-columns: 1fr;
		}
	}
</style>
