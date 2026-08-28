<script lang="ts">
	import { browser } from '$app/environment';
	import { page } from '$app/stores';
	import { authorizeExtension, getProfile } from '$lib/api';
	import FormButton from '$lib/components/auth/FormButton.svelte';
	import { t } from '$lib/i18n';

	let status = $state<'loading' | 'authenticated' | 'authorizing' | 'done' | 'error'>('loading');
	let errorMessage = $state('');
	let userName = $state('');

	const codeChallenge = $derived($page.url.searchParams.get('code_challenge'));
	const stateParam = $derived($page.url.searchParams.get('state'));
	const redirectUri = $derived($page.url.searchParams.get('redirect_uri'));

	function redirectToLogin() {
		const returnUrl = encodeURIComponent(window.location.pathname + window.location.search);
		window.location.href = `/login?redirect=${returnUrl}`;
	}

	async function checkAuth() {
		try {
			const { data, response } = await getProfile();

			if (!data) {
				if (response?.status === 401) {
					redirectToLogin();
					return;
				}
				status = 'error';
				errorMessage = $t('extension_auth_check_failed');
				return;
			}

			userName = data.display_name;
			status = 'authenticated';
		} catch {
			redirectToLogin();
		}
	}

	async function authorize() {
		if (!codeChallenge || !stateParam || !redirectUri) {
			status = 'error';
			errorMessage = $t('extension_auth_missing_parameters');
			return;
		}

		status = 'authorizing';

		try {
			const { data, response } = await authorizeExtension({
				body: {
					code_challenge: codeChallenge,
					code_challenge_method: 'S256',
					state: stateParam,
					redirect_uri: redirectUri
				}
			});

			if (!data) {
				if (response?.status === 401) {
					redirectToLogin();
					return;
				}
				status = 'error';
				errorMessage = $t('extension_auth_failed');
				return;
			}

			const result = data as { code: string; state: string };

			const separator = redirectUri.includes('?') ? '&' : '?';
			const callbackUrl = `${redirectUri}${separator}code=${encodeURIComponent(result.code)}&state=${encodeURIComponent(result.state)}`;
			window.location.href = callbackUrl;

			status = 'done';
		} catch {
			status = 'error';
			errorMessage = $t('extension_auth_failed');
		}
	}

	$effect(() => {
		if (browser) {
			checkAuth();
		}
	});
</script>

{#if status === 'loading'}
	<p class="status-text">{$t('extension_auth_checking')}</p>
{:else if status === 'authenticated'}
	{#if !codeChallenge || !stateParam || !redirectUri}
		<div class="error-banner" role="alert">
			{$t('extension_auth_invalid_request')}
		</div>
	{:else}
		<h1 class="auth-title">{$t('extension_auth_title')}</h1>
		<p class="auth-subtitle">
			{$t('extension_auth_allow_as', { values: { name: userName } })}
		</p>
		<form
			onsubmit={(e) => {
				e.preventDefault();
				authorize();
			}}
		>
			<div class="form-actions">
				<FormButton>{$t('extension_auth_authorize')}</FormButton>
			</div>
		</form>
	{/if}
{:else if status === 'authorizing'}
	<h1 class="auth-title">{$t('extension_auth_title')}</h1>
	<p class="status-text">{$t('extension_auth_authorizing')}</p>
{:else if status === 'done'}
	<h1 class="auth-title">{$t('extension_auth_title')}</h1>
	<p class="status-text success">{$t('extension_auth_redirecting')}</p>
{:else if status === 'error'}
	<div class="error-banner" role="alert">{errorMessage}</div>
	<form
		onsubmit={(e) => {
			e.preventDefault();
			checkAuth();
		}}
	>
		<div class="form-actions">
			<FormButton>{$t('common_try_again')}</FormButton>
		</div>
	</form>
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

	.status-text {
		font-family: var(--font-sans);
		font-size: 14px;
		color: var(--text-secondary);
		text-align: center;
		line-height: 1.5;
	}

	.status-text.success {
		color: var(--success, #2e7d32);
	}

	.error-banner {
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
</style>
