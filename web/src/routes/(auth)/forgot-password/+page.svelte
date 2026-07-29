<script lang="ts">
	import { resolve } from '$app/paths';
	import { getAuth } from '$lib/stores/auth.svelte';
	import FormInput from '$lib/components/auth/FormInput.svelte';
	import FormButton from '$lib/components/auth/FormButton.svelte';

	const auth = getAuth();

	let email = $state('');
	let submitting = $state(false);
	let submitted = $state(false);
	let rateLimited = $state(false);

	async function handleSubmit(e: SubmitEvent) {
		e.preventDefault();
		submitting = true;
		rateLimited = false;

		const { success } = await auth.forgotPassword(email);
		submitting = false;

		if (success) {
			submitted = true;
		} else if (auth.error?.toLowerCase().includes('too many')) {
			rateLimited = true;
		}
	}
</script>

<svelte:head>
	<title>Reset password — Indelible</title>
</svelte:head>

{#if submitted}
	<div class="state-icon">
		<svg width="64" height="64" viewBox="0 0 64 64" fill="none" aria-hidden="true">
			<rect x="6" y="14" width="52" height="36" rx="4" stroke="var(--accent)" stroke-width="2.5" />
			<path
				d="M6 18l26 18 26-18"
				stroke="var(--accent)"
				stroke-width="2.5"
				stroke-linecap="round"
				stroke-linejoin="round"
			/>
			<path
				d="M6 50l18-16"
				stroke="var(--accent)"
				stroke-width="2.5"
				stroke-linecap="round"
				stroke-linejoin="round"
			/>
			<path
				d="M58 50l-18-16"
				stroke="var(--accent)"
				stroke-width="2.5"
				stroke-linecap="round"
				stroke-linejoin="round"
			/>
		</svg>
	</div>
	<h1 class="auth-title">Check your email</h1>
	<p class="auth-body">If an account exists for that address, we've sent a password reset link.</p>
	<div class="back-link-row">
		<svg
			width="14"
			height="14"
			viewBox="0 0 24 24"
			fill="none"
			stroke="var(--accent)"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
			aria-hidden="true"
		>
			<path d="M19 12H5" />
			<path d="M12 19l-7-7 7-7" />
		</svg>
		<a href={resolve('/login')} class="auth-link">Back to sign in</a>
	</div>
{:else}
	<h1 class="auth-title">Reset your password</h1>
	<p class="auth-subtitle">Enter your email and we'll send you a reset link</p>

	{#if rateLimited}
		<div class="auth-error-banner" role="alert">Too many requests. Please try again later.</div>
	{/if}

	{#if auth.error && !rateLimited}
		<div class="auth-error-banner" role="alert">{auth.error}</div>
	{/if}

	<form onsubmit={handleSubmit} novalidate>
		<FormInput
			label="Email"
			type="email"
			autocomplete="email"
			placeholder="you@example.com"
			required
			bind:value={email}
		/>

		<div class="form-actions">
			<FormButton loading={submitting}>Send reset link</FormButton>
		</div>
	</form>

	<div class="back-link-row">
		<svg
			width="14"
			height="14"
			viewBox="0 0 24 24"
			fill="none"
			stroke="var(--accent)"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
			aria-hidden="true"
		>
			<path d="M19 12H5" />
			<path d="M12 19l-7-7 7-7" />
		</svg>
		<a href={resolve('/login')} class="auth-link">Back to sign in</a>
	</div>
{/if}

<style>
	.state-icon {
		display: flex;
		align-items: center;
		justify-content: center;
		margin-bottom: 20px;
	}

	.auth-title {
		font-family: var(--font-sans);
		font-size: 22px;
		font-weight: 700;
		letter-spacing: -0.03em;
		line-height: 1.2;
		color: var(--text-primary);
		margin: 0 0 6px;
		text-align: center;
	}

	.auth-subtitle {
		font-family: var(--font-sans);
		font-size: 15px;
		font-weight: 400;
		letter-spacing: -0.01em;
		line-height: 1.5;
		color: var(--text-secondary);
		text-align: center;
		margin: 0 0 24px;
	}

	.auth-body {
		font-family: var(--font-sans);
		font-size: 15px;
		font-weight: 400;
		letter-spacing: -0.01em;
		line-height: 1.5;
		color: var(--text-secondary);
		text-align: center;
		margin: 0 0 8px;
	}

	.auth-error-banner {
		padding: 10px 14px;
		margin-bottom: 16px;
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 400;
		letter-spacing: -0.01em;
		line-height: 1.45;
		color: var(--destructive);
		background: rgba(255, 59, 48, 0.08);
		border: 1px solid rgba(255, 59, 48, 0.2);
		border-radius: var(--radius-sm);
	}

	.form-actions {
		margin-top: 24px;
	}

	.back-link-row {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		margin-top: 20px;
	}

	.auth-link {
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 400;
		letter-spacing: -0.01em;
		color: var(--accent);
		text-decoration: none;
	}

	.auth-link:hover {
		text-decoration: underline;
	}
</style>
