<script lang="ts">
	let { children } = $props();
</script>

<div class="auth-backdrop">
	<div class="auth-card">
		<div class="auth-logo">
			<span class="auth-logo-text">Indelible</span>
		</div>
		{@render children()}
	</div>
</div>

<style>
	.auth-backdrop {
		display: flex;
		align-items: center;
		justify-content: center;
		min-height: 100dvh;
		background: var(--bg-secondary);
		padding: 24px 16px;
	}

	.auth-card {
		background: var(--bg-elevated);
		border-radius: var(--radius-xl);
		padding: 48px 40px;
		width: 100%;
		max-width: 420px;
		box-shadow:
			0 4px 24px rgba(0, 0, 0, 0.08),
			0 1px 4px rgba(0, 0, 0, 0.04);
	}

	:global([data-theme='dark']) .auth-card {
		box-shadow:
			0 4px 24px rgba(0, 0, 0, 0.4),
			0 1px 4px rgba(0, 0, 0, 0.2);
	}

	.auth-logo {
		text-align: center;
		margin-bottom: 32px;
	}

	.auth-logo-text {
		font-family: var(--font-sans);
		font-size: 28px;
		font-weight: 700;
		letter-spacing: -0.03em;
		color: var(--text-primary);
	}
</style>
