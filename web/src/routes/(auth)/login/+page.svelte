<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { resolve } from '$app/paths';
	import { getAuth } from '$lib/stores/auth.svelte';
	import FormInput from '$lib/components/auth/FormInput.svelte';
	import FormButton from '$lib/components/auth/FormButton.svelte';
	import OAuthButtons from '$lib/components/auth/OAuthButtons.svelte';
	import { getInstanceStatus } from '$lib/api/instance';

	const auth = getAuth();

	let signupsEnabled = $state(false);
	getInstanceStatus().then((status) => {
		signupsEnabled = status.signupsEnabled;
	});

	let email = $state('');
	let password = $state('');
	let submitting = $state(false);

	let fieldErrors = $state<{ email?: string; password?: string }>({});

	const MAX_ATTEMPTS = 5;
	const COOLDOWN_SECONDS = 30;

	let failedAttempts = $state(0);
	let cooldownRemaining = $state(0);
	let cooldownTimer = $state<ReturnType<typeof setInterval> | null>(null);

	let isCoolingDown = $derived(cooldownRemaining > 0);

	function startCooldown(seconds: number = COOLDOWN_SECONDS) {
		cooldownRemaining = seconds;
		if (cooldownTimer) clearInterval(cooldownTimer);
		cooldownTimer = setInterval(() => {
			cooldownRemaining -= 1;
			if (cooldownRemaining <= 0) {
				cooldownRemaining = 0;
				if (cooldownTimer) {
					clearInterval(cooldownTimer);
					cooldownTimer = null;
				}
			}
		}, 1000);
	}

	function validate(): boolean {
		const errors: { email?: string; password?: string } = {};

		if (!email.trim()) {
			errors.email = 'Email is required';
		}

		if (!password) {
			errors.password = 'Password is required';
		}

		fieldErrors = errors;
		return Object.keys(errors).length === 0;
	}

	async function handleSubmit(e: SubmitEvent) {
		e.preventDefault();

		if (isCoolingDown || submitting) return;

		if (!validate()) return;

		submitting = true;
		fieldErrors = {};

		const result = await auth.login(email, password);
		submitting = false;

		if (result.success) {
			// eslint-disable-next-line svelte/no-navigation-without-resolve -- safeRedirectTarget accepts only same-origin absolute paths or resolve('/').
			goto(safeRedirectTarget($page.url.searchParams.get('redirect')));
		} else if (result.rateLimited) {
			startCooldown(result.retryAfter ?? COOLDOWN_SECONDS);
		} else {
			failedAttempts += 1;
			if (failedAttempts >= MAX_ATTEMPTS) {
				startCooldown(COOLDOWN_SECONDS);
				failedAttempts = 0;
			}
		}
	}

	function safeRedirectTarget(redirectUrl: string | null): string {
		if (redirectUrl?.startsWith('/') && !redirectUrl.startsWith('//')) {
			return redirectUrl;
		}
		return resolve('/');
	}

	function getRedirectParam(): string {
		const redirectUrl = $page.url.searchParams.get('redirect');
		if (!redirectUrl?.startsWith('/') || redirectUrl.startsWith('//')) return '';
		return `?redirect=${encodeURIComponent(redirectUrl)}`;
	}

	const forgotPasswordHref = '/forgot-password';
</script>

<svelte:head>
	<title>Sign in - Indelible</title>
</svelte:head>

<h1 class="auth-title">Welcome back</h1>
<p class="auth-subtitle">Sign in to your account</p>

{#if auth.error && !isCoolingDown}
	<div class="auth-error-banner" role="alert">{auth.error}</div>
{/if}

{#if isCoolingDown}
	<div class="auth-error-banner" role="alert">
		Too many attempts. Please try again in {cooldownRemaining} seconds.
	</div>
{/if}

<form onsubmit={handleSubmit} novalidate>
	<FormInput
		label="Email"
		type="email"
		autocomplete="email"
		placeholder="you@example.com"
		required
		bind:value={email}
		error={fieldErrors.email}
	/>

	<FormInput
		label="Password"
		type="password"
		autocomplete="current-password"
		placeholder="Enter your password"
		required
		revealable
		bind:value={password}
		error={fieldErrors.password}
	/>

	<div class="form-actions">
		<FormButton loading={submitting} disabled={isCoolingDown}>Sign in</FormButton>
	</div>
</form>

<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- route created in TASK-042 -->
<a href={forgotPasswordHref} class="forgot-password-link"> Forgot password? </a>

<OAuthButtons dividerText="or" />

{#if signupsEnabled}
	<p class="auth-footer">
		Don't have an account?
		<a href="{resolve('/register')}{getRedirectParam()}" class="auth-link">Sign up</a>
	</p>
{/if}

<style>
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

	.forgot-password-link {
		display: block;
		margin-top: 12px;
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 400;
		letter-spacing: -0.01em;
		color: var(--accent);
		text-decoration: none;
		text-align: center;
	}

	.forgot-password-link:hover {
		text-decoration: underline;
	}

	.auth-footer {
		margin-top: 24px;
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 400;
		letter-spacing: -0.01em;
		color: var(--text-secondary);
		text-align: center;
	}

	.auth-link {
		color: var(--accent);
		text-decoration: none;
	}

	.auth-link:hover {
		text-decoration: underline;
	}
</style>
