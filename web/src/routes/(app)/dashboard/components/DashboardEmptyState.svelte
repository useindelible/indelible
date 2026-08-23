<script lang="ts">
	import { t } from '$lib/i18n';
</script>

<div class="empty-state">
	<div class="empty-icon" aria-hidden="true">
		<svg
			width="40"
			height="40"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="1.3"
			stroke-linecap="round"
			stroke-linejoin="round"
		>
			<path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20" />
			<path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" />
		</svg>
	</div>
	<p class="empty-title">{$t('dashboard_empty_title')}</p>
	<p class="empty-subtitle">{$t('dashboard_empty_body')}</p>
</div>

<style>
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		padding: 60px 0;
		text-align: center;
	}

	.empty-icon {
		color: var(--text-quaternary);
		margin-bottom: 4px;
	}

	.empty-title {
		font-family: var(--font-sans);
		font-size: 17px;
		font-weight: 600;
		color: var(--text-primary);
		margin: 0;
	}

	.empty-subtitle {
		font-family: var(--font-sans);
		font-size: 14px;
		color: var(--text-secondary);
		margin: 0;
	}
</style>
