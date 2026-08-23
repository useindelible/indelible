<script lang="ts">
	import { t } from '$lib/i18n';
</script>

<div class="reader-loading-page">
	<span class="loading-text">{$t('common_loading')}</span>
</div>

<style>
	.reader-loading-page {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 16px;
		height: 100vh;
	}

	.loading-text {
		font-size: 14px;
		color: var(--text-secondary);
		font-family: var(--font-sans);
	}
</style>
