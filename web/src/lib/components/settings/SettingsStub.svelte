<script lang="ts">
	import { t } from '$lib/i18n';

	interface Props {
		title: string;
	}

	let { title }: Props = $props();
</script>

<div class="settings-stub">
	<h2 class="settings-stub-title">{title}</h2>
	<p class="settings-stub-message">{$t('settings_coming_soon')}</p>
</div>

<style>
	.settings-stub {
		padding: 40px 32px;
	}

	.settings-stub-title {
		font-family: var(--font-sans);
		font-size: 22px;
		font-weight: 700;
		letter-spacing: -0.03em;
		line-height: 1.2;
		color: var(--text-primary);
		margin: 0 0 12px;
	}

	.settings-stub-message {
		font-family: var(--font-sans);
		font-size: 15px;
		font-weight: 400;
		letter-spacing: -0.01em;
		line-height: 1.5;
		color: var(--text-secondary);
		margin: 0;
	}
</style>
