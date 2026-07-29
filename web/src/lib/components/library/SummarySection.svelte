<script lang="ts">
	interface Props {
		summary?: string | null;
	}

	let { summary = null }: Props = $props();
</script>

<div class="summary-section">
	<div class="section-heading">Summary</div>
	{#if summary}
		<p class="summary-text">{summary}</p>
	{:else}
		<p class="summary-text summary-stub">AI summary not yet available.</p>
	{/if}
	<p class="summary-attribution">Summarized by Mila</p>
</div>

<style>
	.summary-section {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.section-heading {
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--text-tertiary);
		line-height: 1.2;
	}

	.summary-text {
		font-size: 13px;
		font-weight: 400;
		letter-spacing: -0.01em;
		line-height: 1.65;
		color: var(--text-primary);
		margin: 0;
	}

	.summary-stub {
		color: var(--text-tertiary);
		font-style: italic;
	}

	.summary-attribution {
		font-size: 11px;
		font-weight: 400;
		color: var(--text-tertiary);
		font-style: italic;
		margin: -4px 0 0;
	}
</style>
