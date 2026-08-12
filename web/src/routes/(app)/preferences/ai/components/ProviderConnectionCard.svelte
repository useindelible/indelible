<script lang="ts">
	import type { TestState } from '../mila-settings-model';

	interface Props {
		testState: TestState;
		testMessage: string;
		disabled: boolean;
		onTest: () => void;
	}

	let { testState, testMessage, disabled, onTest }: Props = $props();
</script>

<div class="test-card">
	<div class="test-card-label">Connection</div>
	<div class="test-state-row" data-state={testState}>
		{#if testState === 'testing'}
			<span class="spinner" aria-hidden="true"></span>
		{:else if testState === 'success'}
			<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 12l4 4 10-10" /></svg>
		{:else if testState === 'error'}
			<svg viewBox="0 0 24 24" aria-hidden="true">
				<circle cx="12" cy="12" r="9" />
				<line x1="12" y1="8" x2="12" y2="12" />
				<circle cx="12" cy="16" r="0.5" />
			</svg>
		{:else}
			<svg viewBox="0 0 24 24" aria-hidden="true">
				<circle cx="12" cy="12" r="9" />
				<path d="M12 8v4l3 2" />
			</svg>
		{/if}
		<span class="test-state-msg">{testMessage}</span>
	</div>
	<div class="test-foot">
		<div class="test-meta">
			We send one short completion to the chat provider and one small embedding probe to the
			embedding provider. Nothing is logged.
		</div>
		<button
			type="button"
			class="test-btn"
			onclick={onTest}
			disabled={disabled || testState === 'testing'}
		>
			<svg viewBox="0 0 24 24" aria-hidden="true">
				<path d="M21 12a9 9 0 1 1-3-6.7" />
				<polyline points="21 4 21 10 15 10" />
			</svg>
			{testState === 'testing' ? 'Testing…' : 'Test connection'}
		</button>
	</div>
</div>

<style>
	.test-card {
		grid-column: 1 / -1;
		background: var(--bg-secondary);
		border-radius: 12px;
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		padding: 16px;
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
	.test-card-label {
		font-size: 10.5px;
		font-weight: 600;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		color: var(--text-tertiary);
	}
	.test-state-row {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 9px 11px;
		border-radius: 10px;
		font-size: 12.5px;
		line-height: 1.35;
		letter-spacing: -0.005em;
		background: var(--mila-status-idle-bg);
		color: var(--mila-status-idle-text);
		transition:
			background 200ms,
			color 200ms;
	}
	.test-state-row[data-state='testing'] {
		background: var(--mila-status-test-bg);
		color: var(--mila-status-test-text);
	}
	.test-state-row[data-state='success'] {
		background: var(--mila-status-ok-bg);
		color: var(--mila-status-ok-text);
	}
	.test-state-row[data-state='error'] {
		background: var(--mila-status-err-bg);
		color: var(--mila-status-err-text);
	}
	.test-state-row svg {
		width: 13px;
		height: 13px;
		flex-shrink: 0;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.9;
		stroke-linecap: round;
		stroke-linejoin: round;
	}
	.test-state-msg {
		flex: 1;
		min-width: 0;
		word-break: break-word;
	}
	.spinner {
		width: 13px;
		height: 13px;
		flex-shrink: 0;
		border-radius: 50%;
		border: 1.6px solid currentColor;
		border-right-color: transparent;
		animation: spin 800ms linear infinite;
	}
	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
	.test-foot {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 18px;
	}
	.test-meta {
		flex: 1;
		min-width: 0;
		font-size: 11px;
		line-height: 1.4;
		letter-spacing: -0.005em;
		color: var(--text-tertiary);
	}
	.test-btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		flex-shrink: 0;
		border: 0;
		border-radius: 9px;
		padding: 7px 13px;
		font: inherit;
		font-size: 12.5px;
		font-weight: 500;
		letter-spacing: -0.005em;
		white-space: nowrap;
		cursor: pointer;
		background: var(--mila-violet-soft);
		color: var(--mila-violet);
		transition: background 140ms;
	}
	.test-btn:hover:not(:disabled) {
		background: var(--fill-selected);
	}
	.test-btn:disabled {
		opacity: 0.45;
		cursor: default;
	}
	.test-btn svg {
		width: 12px;
		height: 12px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.7;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	@media (max-width: 640px) {
		.test-foot {
			flex-direction: column;
			align-items: stretch;
			gap: 12px;
		}
		.test-btn {
			justify-content: center;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.spinner {
			animation: none;
		}
	}
</style>
