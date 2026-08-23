<script lang="ts">
	import { t } from '$lib/i18n';

	interface Props {
		text?: string;
	}

	let { text }: Props = $props();
</script>

<div class="auth-divider">
	<div class="auth-divider-line"></div>
	<span class="auth-divider-text">{text ?? $t('auth_or')}</span>
	<div class="auth-divider-line"></div>
</div>

<style>
	.auth-divider {
		display: flex;
		align-items: center;
		gap: 16px;
		margin: 24px 0;
	}

	.auth-divider-line {
		flex: 1;
		height: 1px;
		background: var(--border-primary);
	}

	.auth-divider-text {
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 400;
		letter-spacing: -0.01em;
		color: var(--text-tertiary);
		text-transform: lowercase;
	}
</style>
