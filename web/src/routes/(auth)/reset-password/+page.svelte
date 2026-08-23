<script lang="ts">
	import { resolve } from '$app/paths';
	import { getAuth } from '$lib/stores/auth.svelte';
	import FormInput from '$lib/components/auth/FormInput.svelte';
	import FormButton from '$lib/components/auth/FormButton.svelte';
	import { t } from '$lib/i18n';

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
			validationError = $t('auth_password_too_short_sentence');
			return false;
		}
		if (password.length > 2048) {
			validationError = $t('auth_password_too_long_sentence');
			return false;
		}
		if (password !== confirmPassword) {
			validationError = $t('auth_passwords_do_not_match');
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
	<title>{$t('auth_set_new_password_title')}</title>
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
	<h1 class="auth-title">{$t('auth_invalid_reset_link')}</h1>
	<p class="auth-body">{$t('auth_invalid_reset_link_body')}</p>
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
		<a href={resolve('/forgot-password')} class="auth-link">{$t('auth_request_new_link')}</a>
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
	<h1 class="auth-title">{$t('auth_password_updated')}</h1>
	<p class="auth-body">{$t('auth_password_updated_body')}</p>
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
		<a href={resolve('/login')} class="auth-link">{$t('auth_sign_in')}</a>
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
	<h1 class="auth-title">{$t('auth_link_expired')}</h1>
	<p class="auth-body">{$t('auth_link_expired_body')}</p>
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
		<a href={resolve('/forgot-password')} class="auth-link">{$t('auth_request_new_link')}</a>
	</div>
{:else}
	<h1 class="auth-title">{$t('auth_set_new_password')}</h1>
	<p class="auth-subtitle">{$t('auth_set_new_password_subtitle')}</p>

	{#if validationError}
		<div class="auth-error-banner" role="alert">{validationError}</div>
	{/if}

	{#if auth.error && !tokenError}
		<div class="auth-error-banner" role="alert">{auth.error}</div>
	{/if}

	<form onsubmit={handleSubmit} novalidate>
		<FormInput
			label={$t('auth_new_password')}
			type="password"
			autocomplete="new-password"
			placeholder={$t('auth_new_password_placeholder')}
			required
			bind:value={password}
		/>

		<FormInput
			label={$t('auth_confirm_password')}
			type="password"
			autocomplete="new-password"
			placeholder={$t('auth_confirm_password_placeholder')}
			required
			bind:value={confirmPassword}
		/>

		<div class="form-actions">
			<FormButton loading={submitting}>{$t('auth_reset_password')}</FormButton>
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
		<a href={resolve('/login')} class="auth-link">{$t('auth_back_to_sign_in')}</a>
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
