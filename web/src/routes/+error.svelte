<script lang="ts">
	import { page } from '$app/state';
	import { resolve } from '$app/paths';
	import { t } from '$lib/i18n';

	const is404 = $derived(page.status === 404);

	const title = $derived(is404 ? $t('error_page_not_found_title') : $t('error_generic_title'));
	const body = $derived(is404 ? $t('error_page_not_found_body') : $t('error_generic_body'));
</script>

<div class="error-page">
	<div class="backdrop-code" aria-hidden="true">{page.status}</div>

	<div class="error-content">
		<div class="error-icon" aria-hidden="true">
			{#if is404}
				<svg width="32" height="32" viewBox="0 0 32 32" fill="none">
					<path
						d="M8 6h10l6 6v14a2 2 0 01-2 2H8a2 2 0 01-2-2V8a2 2 0 012-2z"
						stroke="currentColor"
						stroke-width="1.5"
						stroke-linecap="round"
						stroke-linejoin="round"
					/>
					<path
						d="M18 6v6h6"
						stroke="currentColor"
						stroke-width="1.5"
						stroke-linecap="round"
						stroke-linejoin="round"
					/>
					<path
						d="M13 17l6 6M19 17l-6 6"
						stroke="currentColor"
						stroke-width="1.5"
						stroke-linecap="round"
					/>
				</svg>
			{:else}
				<svg width="32" height="32" viewBox="0 0 32 32" fill="none">
					<circle cx="16" cy="16" r="12" stroke="currentColor" stroke-width="1.5" />
					<path d="M16 10v7" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
					<circle cx="16" cy="21" r="1" fill="currentColor" />
				</svg>
			{/if}
		</div>

		<h1 class="error-title">{title}</h1>
		<p class="error-body">{body}</p>

		<a href={resolve('/')} class="error-link">
			<svg
				width="16"
				height="16"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
				stroke-linecap="round"
				stroke-linejoin="round"
				aria-hidden="true"
			>
				<path d="M19 12H5M12 19l-7-7 7-7" />
			</svg>
			{$t('error_go_to_library')}
		</a>
	</div>
</div>

<style>
	.error-page {
		min-height: 100vh;
		display: flex;
		align-items: center;
		justify-content: center;
		background: var(--bg-primary);
		position: relative;
		overflow: hidden;
	}

	.backdrop-code {
		position: absolute;
		font-family: var(--font-sans);
		font-size: clamp(140px, 28vw, 280px);
		font-weight: 700;
		letter-spacing: -0.06em;
		line-height: 1;
		color: rgba(0, 0, 0, 0.06);
		user-select: none;
		pointer-events: none;
	}

	:global([data-theme='dark']) .backdrop-code {
		color: rgba(255, 255, 255, 0.09);
	}

	.error-content {
		position: relative;
		display: flex;
		flex-direction: column;
		align-items: center;
		text-align: center;
		gap: 12px;
		padding: 40px 24px;
		max-width: 400px;
		animation: fade-up 0.4s ease both;
	}

	.error-icon {
		color: var(--text-secondary);
		margin-bottom: 4px;
	}

	.error-title {
		font-family: var(--font-sans);
		font-size: 22px;
		font-weight: 700;
		letter-spacing: -0.03em;
		line-height: 1.2;
		color: var(--text-primary);
		margin: 0;
	}

	.error-body {
		font-family: var(--font-sans);
		font-size: 15px;
		font-weight: 400;
		letter-spacing: -0.01em;
		line-height: 1.5;
		color: var(--text-secondary);
		margin: 0;
	}

	.error-link {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		margin-top: 8px;
		font-family: var(--font-sans);
		font-size: 15px;
		font-weight: 500;
		letter-spacing: -0.01em;
		color: var(--accent);
		text-decoration: none;
		transition: opacity 0.15s ease;
	}

	.error-link:hover {
		opacity: 0.75;
	}

	@keyframes fade-up {
		from {
			opacity: 0;
			transform: translateY(12px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}
</style>
