<script lang="ts">
	let { children } = $props();
</script>

<div class="auth-backdrop">
	<div class="auth-card">
		<div class="auth-hero">
			<span class="auth-hero-halo">
				<img
					class="auth-hero-mark"
					src="/brand/indelible-mark-primary.svg"
					alt="Indelible"
					width="80"
					height="80"
				/>
			</span>
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
		position: relative;
		background: var(--bg-elevated);
		border-radius: var(--radius-xl);
		padding: 132px 40px 48px;
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

	.auth-hero {
		position: absolute;
		top: -8px;
		left: 0;
		right: 0;
		display: flex;
		justify-content: center;
		pointer-events: none;
		z-index: 1;
	}

	.auth-hero-halo {
		position: relative;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 148px;
		height: 148px;
	}

	.auth-hero-halo::before {
		content: '';
		position: absolute;
		inset: 0;
		border-radius: 50%;
		background: radial-gradient(
			closest-side,
			color-mix(in srgb, var(--accent) 36%, transparent),
			transparent 70%
		);
	}

	:global([data-theme='dark']) .auth-hero-halo::before {
		background: radial-gradient(
			closest-side,
			color-mix(in srgb, var(--accent) 44%, transparent),
			transparent 70%
		);
	}

	.auth-hero-mark {
		position: relative;
		display: block;
		width: 80px;
		height: 80px;
	}
</style>
