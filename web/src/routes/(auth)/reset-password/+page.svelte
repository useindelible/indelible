<script lang="ts">
	import { resolve } from '$app/paths';
	import { getAuth } from '$lib/stores/auth.svelte';
	import FormInput from '$lib/components/auth/FormInput.svelte';
	import FormButton from '$lib/components/auth/FormButton.svelte';

	const { data } = $props();
	const auth = getAuth();

	let password = $state('');
	let confirmPassword = $state('');
	let submitting = $state(false);
	let success = $state(false);
	let tokenError = $state(false);
	let validationError = $state<string | null>(null);

	const hasToken = $derived(data.token !== null);

	function validate(): boolean {
		if (password.length < 8) {
			validationError = 'Password must be at least 8 characters.';
			return false;
		}
		if (password.length > 2048) {
			validationError = 'Password must be no more than 2048 characters.';
			return false;
		}
		if (password !== confirmPassword) {
			validationError = 'Passwords do not match.';
			return false;
		}
		validationError = null;
		return true;
	}

	async function handleSubmit(e: SubmitEvent) {
		e.preventDefault();
		if (!validate() || !data.token) return;

		submitting = true;
		tokenError = false;

		const result = await auth.resetPassword(data.token, password);
		submitting = false;

		if (result.success) {
			success = true;
		} else if (result.expired) {
			tokenError = true;
		}
	}
</script>

<svelte:head>
	<title>Set new password — Indelible</title>
</svelte:head>

{#if !hasToken}
	<div class="state-icon">
		<svg width="64" height="64" viewBox="0 0 64 64" fill="none" aria-hidden="true">
			<circle cx="32" cy="32" r="28" stroke="var(--destructive)" stroke-width="2.5" />
			<path
				d="M22 22l20 20M42 22L22 42"
				stroke="var(--destructive)"
				stroke-width="3"
				stroke-linecap="round"
			/>
		</svg>
	</div>
	<h1 class="auth-title">Invalid reset link</h1>
	<p class="auth-body">This password reset link is missing a token. Please request a new one.</p>
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
		<a href={resolve('/forgot-password')} class="auth-link">Request a new link</a>
	</div>
{:else if success}
	<div class="state-icon">
		<svg width="64" height="64" viewBox="0 0 64 64" fill="none" aria-hidden="true">
			<circle cx="32" cy="32" r="28" stroke="var(--success)" stroke-width="2.5" />
			<path
				d="M20 32l8 8 16-16"
				stroke="var(--success)"
				stroke-width="3"
				stroke-linecap="round"
				stroke-linejoin="round"
			/>
		</svg>
	</div>
	<h1 class="auth-title">Password updated</h1>
	<p class="auth-body">Your password has been reset successfully.</p>
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
		<a href={resolve('/login')} class="auth-link">Sign in</a>
	</div>
{:else if tokenError}
	<div class="state-icon">
		<svg width="64" height="64" viewBox="0 0 64 64" fill="none" aria-hidden="true">
			<circle cx="32" cy="32" r="28" stroke="var(--destructive)" stroke-width="2.5" />
			<path
				d="M22 22l20 20M42 22L22 42"
				stroke="var(--destructive)"
				stroke-width="3"
				stroke-linecap="round"
			/>
		</svg>
	</div>
	<h1 class="auth-title">Link expired</h1>
	<p class="auth-body">This password reset link has expired or is invalid.</p>
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
		<a href={resolve('/forgot-password')} class="auth-link">Request a new link</a>
	</div>
{:else}
	<h1 class="auth-title">Set new password</h1>
	<p class="auth-subtitle">Choose a strong password for your account</p>

	{#if validationError}
		<div class="auth-error-banner" role="alert">{validationError}</div>
	{/if}

	{#if auth.error && !tokenError}
		<div class="auth-error-banner" role="alert">{auth.error}</div>
	{/if}

	<form onsubmit={handleSubmit} novalidate>
		<FormInput
			label="New password"
			type="password"
			autocomplete="new-password"
			placeholder="Enter new password"
			required
			bind:value={password}
		/>

		<FormInput
			label="Confirm password"
			type="password"
			autocomplete="new-password"
			placeholder="Confirm new password"
			required
			bind:value={confirmPassword}
		/>

		<div class="form-actions">
			<FormButton loading={submitting}>Reset password</FormButton>
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
