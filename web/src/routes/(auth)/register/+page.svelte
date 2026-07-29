<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { resolve } from '$app/paths';
	import { getAuth } from '$lib/stores/auth.svelte';
	import FormInput from '$lib/components/auth/FormInput.svelte';
	import FormButton from '$lib/components/auth/FormButton.svelte';
	import OAuthButtons from '$lib/components/auth/OAuthButtons.svelte';
	import PasswordStrength from '$lib/components/auth/PasswordStrength.svelte';
	import { getInstanceStatus } from '$lib/api/instance';

	const auth = getAuth();

	let signupsEnabled = $state(false);
	let setupRequired = $state(false);
	getInstanceStatus().then((status) => {
		signupsEnabled = status.signupsEnabled;
		setupRequired = status.setupRequired;
	});

	let displayName = $state('');
	let email = $state('');
	let password = $state('');
	let submitting = $state(false);

	let fieldErrors = $state<{
		displayName?: string;
		email?: string;
		password?: string;
	}>({});

	function validate(): boolean {
		const errors: typeof fieldErrors = {};

		if (!displayName.trim()) {
			errors.displayName = 'Display name is required';
		} else if (displayName.trim().length > 100) {
			errors.displayName = 'Display name must be 100 characters or fewer';
		}

		if (!email.trim()) {
			errors.email = 'Email is required';
		} else if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email.trim())) {
			errors.email = 'Please enter a valid email address';
		}

		if (!password) {
			errors.password = 'Password is required';
		} else if (password.length < 8) {
			errors.password = 'Password must be at least 8 characters';
		} else if (password.length > 2048) {
			errors.password = 'Password must be 2048 characters or fewer';
		}

		fieldErrors = errors;
		return Object.keys(errors).length === 0;
	}

	async function handleSubmit(e: SubmitEvent) {
		e.preventDefault();

		if (submitting) return;

		if (!validate()) return;

		submitting = true;
		fieldErrors = {};

		const { success } = await auth.register(email, password, displayName.trim());
		submitting = false;

		if (success) {
			if (auth.needsVerification) {
				const redirectParam = $page.url.searchParams.get('redirect');
				if (redirectParam) {
					// eslint-disable-next-line svelte/no-navigation-without-resolve -- URL is built from resolve()
					goto(`${resolve('/verify-email')}?redirect=${encodeURIComponent(redirectParam)}`);
				} else {
					goto(resolve('/verify-email'));
				}
			} else {
				const redirectUrl = $page.url.searchParams.get('redirect');
				// eslint-disable-next-line svelte/no-navigation-without-resolve -- /onboarding/welcome created in TASK-044
				goto(redirectUrl ?? '/onboarding/welcome');
			}
		}
	}

	function getRedirectParam(): string {
		const redirectUrl = $page.url.searchParams.get('redirect');
		return redirectUrl ? `?redirect=${encodeURIComponent(redirectUrl)}` : '';
	}
</script>

<svelte:head>
	<title>Create account - Indelible</title>
</svelte:head>

{#if signupsEnabled}
	<h1 class="auth-title">{setupRequired ? 'Set up Indelible' : 'Create your account'}</h1>
	<p class="auth-subtitle">
		{setupRequired ? 'Create the first account to get started' : 'Start saving what matters'}
	</p>

	{#if auth.error}
		<div class="auth-error-banner" role="alert">{auth.error}</div>
	{/if}

	<form onsubmit={handleSubmit} novalidate>
		<FormInput
			label="Display name"
			type="text"
			autocomplete="name"
			placeholder="Your name"
			required
			bind:value={displayName}
			error={fieldErrors.displayName}
		/>

		<FormInput
			label="Email"
			type="email"
			autocomplete="email"
			placeholder="you@example.com"
			required
			bind:value={email}
			error={fieldErrors.email}
		/>

		<div class="password-field">
			<FormInput
				label="Password"
				type="password"
				autocomplete="new-password"
				placeholder="Min. 8 characters"
				required
				revealable
				bind:value={password}
				error={fieldErrors.password}
			/>
			<PasswordStrength {password} />
		</div>

		<div class="form-actions">
			<FormButton loading={submitting}>Create account</FormButton>
		</div>
	</form>

	<OAuthButtons dividerText="or" />
{/if}

{#if !setupRequired}
	<p class="auth-footer">
		Already have an account?
		<a href="{resolve('/login')}{getRedirectParam()}" class="auth-link">Sign in</a>
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

	.password-field {
		margin-bottom: 16px;
	}

	.password-field :global(.form-field) {
		margin-bottom: 0;
	}

	.form-actions {
		margin-top: 24px;
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
