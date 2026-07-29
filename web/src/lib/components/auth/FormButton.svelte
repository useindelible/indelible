<script lang="ts">
	interface Props {
		type?: 'submit' | 'button';
		loading?: boolean;
		disabled?: boolean;
		children: import('svelte').Snippet;
	}

	let { type = 'submit', loading = false, disabled = false, children }: Props = $props();
</script>

<button {type} class="form-button" disabled={loading || disabled} aria-busy={loading}>
	{#if loading}
		<span class="spinner" aria-hidden="true"></span>
	{/if}
	<span class:visually-hidden={loading}>
		{@render children()}
	</span>
</button>

<style>
	.form-button {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		width: 100%;
		padding: 12px 20px;
		font-family: var(--font-sans);
		font-size: 15px;
		font-weight: 600;
		letter-spacing: -0.01em;
		line-height: 1.5;
		color: var(--text-on-color);
		background: var(--accent);
		border: none;
		border-radius: var(--radius-sm);
		cursor: pointer;
		transition:
			background-color 0.15s ease,
			opacity 0.15s ease;
	}

	.form-button:hover:not(:disabled) {
		background: var(--accent-hover);
	}

	.form-button:disabled {
		opacity: 0.55;
		cursor: not-allowed;
	}

	.spinner {
		display: inline-block;
		width: 16px;
		height: 16px;
		border: 2px solid rgba(255, 255, 255, 0.3);
		border-top-color: var(--text-on-color);
		border-radius: 50%;
		animation: spin 0.6s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.visually-hidden {
		position: absolute;
		width: 1px;
		height: 1px;
		padding: 0;
		margin: -1px;
		overflow: hidden;
		clip: rect(0, 0, 0, 0);
		white-space: nowrap;
		border: 0;
	}
</style>
