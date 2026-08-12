<script lang="ts">
	import { resolve } from '$app/paths';
	import type { ReaderAiFailure } from '../reader-realtime';

	type RetryStatus = 'idle' | 'pending' | 'queued' | 'error';

	interface Props {
		failure: ReaderAiFailure;
		status: RetryStatus;
		onRetry: () => void;
		onDismiss: () => void;
	}

	let { failure, status, onRetry, onDismiss }: Props = $props();

	const title = $derived(
		failure.action === 'summary'
			? "Mila couldn't create a summary."
			: failure.action === 'tags'
				? "Mila couldn't suggest tags."
				: failure.action === 'entities'
					? "Mila couldn't extract entities."
					: "Mila couldn't complete this request."
	);
	const retryable = $derived(['summary', 'tags', 'entities'].includes(failure.action));
</script>

<div class="notice" role="alert">
	<div class="notice-copy">
		<strong>{title}</strong>
		{#if status === 'queued'}
			<p class="status success">Retry queued.</p>
		{:else if status === 'error'}
			<p class="status error">Could not queue retry. Try again.</p>
		{/if}
		{#if failure.aiRunId}
			<span class="run-id">Run {failure.aiRunId}</span>
		{/if}
		<details>
			<summary>Technical details</summary>
			<p>{failure.message}</p>
		</details>
	</div>

	<div class="actions">
		{#if retryable}
			<button type="button" class="primary" onclick={onRetry} disabled={status === 'pending'}>
				{status === 'pending' ? 'Queuing retry…' : 'Retry'}
			</button>
		{/if}
		<a href={resolve('/preferences/ai')}>Open Mila settings</a>
		<button type="button" class="quiet" onclick={onDismiss}>Dismiss</button>
	</div>
</div>

<style>
	.notice {
		position: fixed;
		top: 16px;
		left: 50%;
		z-index: 1000;
		display: flex;
		width: min(620px, calc(100vw - 32px));
		max-height: calc(100dvh - 32px);
		overflow-y: auto;
		transform: translateX(-50%);
		gap: 20px;
		justify-content: space-between;
		padding: 14px 16px;
		border: 1px solid var(--destructive-border);
		border-radius: 12px;
		background: var(--bg-elevated);
		color: var(--text-primary);
		box-shadow: var(--shadow-3);
		font-size: 13px;
	}

	.notice-copy {
		min-width: 0;
	}

	strong {
		display: block;
		font-size: 14px;
	}

	.status {
		margin: 4px 0 0;
	}

	.success {
		color: var(--success);
	}

	.error {
		color: var(--destructive);
	}

	.run-id {
		display: block;
		margin-top: 6px;
		color: var(--text-secondary);
		font-family: var(--font-mono);
		font-size: 11px;
	}

	details {
		margin-top: 6px;
		color: var(--text-secondary);
	}

	summary {
		cursor: pointer;
	}

	details p {
		margin: 6px 0 0;
		overflow-wrap: anywhere;
	}

	.actions {
		display: flex;
		flex: 0 0 auto;
		align-items: center;
		gap: 10px;
	}

	.actions a,
	.actions button {
		border: 0;
		background: transparent;
		color: var(--accent);
		font: inherit;
		text-decoration: none;
		cursor: pointer;
		white-space: nowrap;
	}

	.actions .primary {
		padding: 7px 11px;
		border-radius: 7px;
		background: var(--accent);
		color: white;
		font-weight: 600;
	}

	.actions .primary:disabled {
		opacity: 0.55;
		cursor: default;
	}

	.actions .quiet {
		color: var(--text-secondary);
	}

	@media (max-width: 640px) {
		.notice {
			align-items: stretch;
			flex-direction: column;
			gap: 12px;
		}

		.actions {
			flex-wrap: wrap;
		}
	}
</style>
