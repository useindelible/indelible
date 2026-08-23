<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/stores';
	import { getAuth } from '$lib/stores/auth.svelte';
	import { t } from '$lib/i18n';

	const { data } = $props();
	const auth = getAuth();

	let verifying = $state(false);
	let verified = $state(false);
	let verifyError = $state(false);
	let resending = $state(false);
	let resent = $state(false);
	let countdown = $state(0);
	let countdownInterval = $state<ReturnType<typeof setInterval> | null>(null);

	const hasToken = $derived(data.token !== null);

	$effect(() => {
		if (hasToken && data.token && !verifying && !verified && !verifyError) {
			verifyToken(data.token);
		}
	});

	async function verifyToken(token: string) {
		verifying = true;
		const { success } = await auth.verifyEmail(token);
		verifying = false;

		if (success) {
			verified = true;
		} else {
			verifyError = true;
		}
	}

	function startCountdown() {
		countdown = 60;
		if (countdownInterval) clearInterval(countdownInterval);
		countdownInterval = setInterval(() => {
			countdown -= 1;
			if (countdown <= 0) {
				if (countdownInterval) clearInterval(countdownInterval);
				countdownInterval = null;
			}
		}, 1000);
	}

	async function handleResend() {
		resending = true;
		resent = false;

		const { success } = await auth.resendVerification();
		resending = false;

		if (success) {
			resent = true;
			startCountdown();
		}
	}

	function handleContinue() {
		const redirectTarget = $page.url.searchParams.get('redirect');
		if (auth.needsOnboarding) {
			goto(resolve('/onboarding/welcome'));
		} else if (redirectTarget) {
			// eslint-disable-next-line svelte/no-navigation-without-resolve -- redirect is a caller-supplied URL from query string
			goto(redirectTarget);
		} else {
			goto(resolve('/'));
		}
	}
</script>

<svelte:head>
	<title>{$t('auth_verify_email_title')}</title>
</svelte:head>

{#if hasToken}
	{#if verifying}
		<div class="state-icon">
			<div class="spinner" role="status" aria-label={$t('auth_verifying_email')}></div>
		</div>
		<h1 class="auth-title">{$t('auth_verifying_your_email')}</h1>
	{:else if verified}
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
		<h1 class="auth-title">{$t('auth_email_verified')}</h1>
		<p class="auth-body">{$t('auth_email_verified_body')}</p>
		<div class="button-row">
			<button class="primary-button" onclick={handleContinue}
				>{$t('auth_continue_to_indelible')}</button
			>
		</div>
	{:else if verifyError}
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
		<h1 class="auth-title">{$t('auth_verification_failed')}</h1>
		<p class="auth-body">{$t('auth_verification_failed_body')}</p>
		{#if auth.error}
			<p class="auth-error">{auth.error}</p>
		{/if}
		<div class="button-row">
			<button
				class="primary-button"
				onclick={handleResend}
				disabled={resending || countdown > 0}
				aria-busy={resending}
			>
				{#if resending}
					{$t('auth_sending')}
				{:else if countdown > 0}
					{$t('auth_resend_email_countdown', { values: { seconds: countdown } })}
				{:else}
					{$t('auth_resend_verification_email')}
				{/if}
			</button>
		</div>
		<p class="auth-footer">
			<a href={resolve('/login')} class="auth-link">{$t('auth_back_to_sign_in')}</a>
		</p>
	{/if}
{:else}
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

	<h1 class="auth-title">{$t('auth_check_email')}</h1>
	<p class="auth-body">
		{$t('auth_verification_sent_to')}<br />
		{#if auth.user?.email}
			<strong>{auth.user.email}</strong>
		{:else}
			{$t('auth_your_email_address')}
		{/if}
	</p>

	{#if resent}
		<p class="resent-notice">{$t('auth_verification_email_sent')}</p>
	{/if}

	{#if auth.error}
		<p class="auth-error">{auth.error}</p>
	{/if}

	<div class="resend-row">
		<button class="link-button" onclick={handleResend} disabled={resending || countdown > 0}>
			{#if resending}
				{$t('auth_sending')}
			{:else if countdown > 0}
				{$t('auth_resend_email_countdown', { values: { seconds: countdown } })}
			{:else}
				{$t('auth_resend_email')}
			{/if}
		</button>
	</div>

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
		<a href={resolve('/login')} class="auth-link">{$t('auth_back_to_sign_in')}</a>
	</div>

	<p class="auth-footnote">{$t('auth_check_spam')}</p>
{/if}

<style>
	.state-icon {
		display: flex;
		align-items: center;
		justify-content: center;
		margin-bottom: 20px;
	}

	.spinner {
		width: 40px;
		height: 40px;
		border: 3px solid var(--border-secondary);
		border-top-color: var(--accent);
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.auth-title {
		font-family: var(--font-sans);
		font-size: 22px;
		font-weight: 700;
		letter-spacing: -0.03em;
		line-height: 1.2;
		color: var(--text-primary);
		margin: 0 0 8px;
		text-align: center;
	}

	.auth-body {
		font-family: var(--font-sans);
		font-size: 15px;
		font-weight: 400;
		letter-spacing: -0.01em;
		line-height: 1.5;
		color: var(--text-secondary);
		text-align: center;
		margin: 0;
	}

	.auth-body strong {
		font-weight: 500;
		color: var(--text-primary);
	}

	.primary-button {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 100%;
		padding: 12px 20px;
		font-family: var(--font-sans);
		font-size: 15px;
		font-weight: 600;
		letter-spacing: -0.01em;
		line-height: 1.5;
		color: #ffffff;
		background: var(--accent);
		border: none;
		border-radius: var(--radius-sm);
		cursor: pointer;
		transition:
			background-color 0.15s ease,
			opacity 0.15s ease;
	}

	.primary-button:hover:not(:disabled) {
		background: var(--accent-hover);
	}

	.primary-button:disabled {
		opacity: 0.55;
		cursor: not-allowed;
	}

	.button-row {
		margin-top: 24px;
	}

	.resend-row {
		margin-top: 20px;
		text-align: center;
	}

	.link-button {
		background: none;
		border: none;
		padding: 0;
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 400;
		letter-spacing: -0.01em;
		color: var(--accent);
		cursor: pointer;
	}

	.link-button:hover:not(:disabled) {
		text-decoration: underline;
	}

	.link-button:disabled {
		opacity: 0.6;
		cursor: default;
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

	.auth-footer {
		margin-top: 20px;
		font-family: var(--font-sans);
		font-size: 13px;
		letter-spacing: -0.01em;
		color: var(--text-secondary);
		text-align: center;
	}

	.auth-footnote {
		margin-top: 20px;
		font-family: var(--font-sans);
		font-size: 12px;
		font-weight: 400;
		letter-spacing: -0.005em;
		line-height: 1.4;
		color: var(--text-secondary);
		text-align: center;
	}

	.resent-notice {
		margin-top: 12px;
		font-family: var(--font-sans);
		font-size: 13px;
		letter-spacing: -0.01em;
		color: var(--success);
		text-align: center;
	}

	.auth-error {
		margin-top: 12px;
		font-family: var(--font-sans);
		font-size: 13px;
		letter-spacing: -0.01em;
		color: var(--destructive);
		text-align: center;
	}
</style>
