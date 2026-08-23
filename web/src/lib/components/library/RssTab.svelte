<script lang="ts">
	import { t } from '$lib/i18n';
	// RSS feed subscription is not yet implemented. This tab is a placeholder
	// displayed as disabled in the tab bar until the feed management API exists.
</script>

<div class="rss-stub">
	<svg
		class="rss-icon"
		viewBox="0 0 24 24"
		fill="none"
		stroke="currentColor"
		stroke-width="1.5"
		stroke-linecap="round"
		stroke-linejoin="round"
	>
		<path d="M4 11a9 9 0 0 1 9 9" />
		<path d="M4 4a16 16 0 0 1 16 16" />
		<circle cx="5" cy="19" r="1" fill="currentColor" stroke="none" />
	</svg>
	<p class="rss-title">{$t('library_rss_coming_soon')}</p>
	<p class="rss-description">
		{$t('library_rss_description')}
	</p>
</div>

<style>
	.rss-stub {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 10px;
		padding: 32px 0;
		text-align: center;
	}

	.rss-icon {
		width: 40px;
		height: 40px;
		color: var(--text-quaternary, var(--text-tertiary));
	}

	.rss-title {
		font-size: 15px;
		font-weight: 600;
		color: var(--text-secondary);
		font-family: var(--font-sans);
		letter-spacing: -0.01em;
		margin: 0;
	}

	.rss-description {
		font-size: 13px;
		color: var(--text-tertiary);
		font-family: var(--font-sans);
		letter-spacing: -0.01em;
		line-height: 1.5;
		margin: 0;
		max-width: 280px;
	}
</style>
