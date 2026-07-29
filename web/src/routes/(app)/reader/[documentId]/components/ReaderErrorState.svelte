<script lang="ts">
	interface Props {
		message: string;
		onBack: () => void;
	}

	let { message, onBack }: Props = $props();
</script>

<div class="reader-error-page">
	<p>{message}</p>
	<button type="button" class="error-back-btn" onclick={onBack}>Return to library</button>
</div>

<style>
	.reader-error-page {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 16px;
		height: 100vh;
	}

	.reader-error-page p {
		font-size: 14px;
		color: var(--text-secondary);
		font-family: var(--font-sans);
		margin: 0;
	}

	.error-back-btn {
		font-size: 13px;
		font-weight: 500;
		color: var(--accent);
		background: none;
		border: 1px solid var(--border-primary);
		border-radius: 980px;
		padding: 8px 20px;
		cursor: pointer;
		font-family: var(--font-sans);
		transition: background 120ms ease;
	}

	.error-back-btn:hover {
		background: var(--fill-hover);
	}
</style>
