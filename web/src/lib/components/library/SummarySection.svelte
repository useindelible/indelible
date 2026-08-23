<script lang="ts">
	import { t } from '$lib/i18n';
	interface Props {
		excerpt?: string | null;
		summary?: string | null;
	}

	let { excerpt = null, summary = null }: Props = $props();

	const normalizedExcerpt = $derived(excerpt?.trim() || null);
	const normalizedSummary = $derived(summary?.trim() || null);
	const isMilaSummary = $derived(
		Boolean(normalizedSummary && normalizedSummary !== normalizedExcerpt)
	);
	const heading = $derived(
		$t(isMilaSummary || !normalizedExcerpt ? 'library_edit_summary' : 'library_excerpt')
	);
	const text = $derived(isMilaSummary ? normalizedSummary : normalizedExcerpt);
</script>

<div class="summary-section">
	<div class="section-heading">{heading}</div>
	{#if text}
		<p class="summary-text">{text}</p>
	{:else}
		<p class="summary-text summary-stub">{$t('library_summary_unavailable')}</p>
	{/if}
	{#if isMilaSummary}
		<p class="summary-attribution">{$t('library_summary_attribution')}</p>
	{/if}
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
