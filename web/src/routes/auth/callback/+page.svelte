<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { resolve } from '$app/paths';
	import { getAuth } from '$lib/stores/auth.svelte';
	import { t } from '$lib/i18n';

	const auth = getAuth();

	let status = $state<'loading' | 'error'>('loading');
	let errorMessage = $state('');

	const provider = $derived($page.url.searchParams.get('provider'));

	async function handleCallback() {
		try {
			await auth.initialize();

			if (!auth.isAuthenticated) {
				status = 'error';
				errorMessage = $t('auth_callback_failed');
				return;
			}

			if (auth.needsOnboarding) {
				goto(resolve('/onboarding/welcome'), { replaceState: true });
			} else if (auth.needsVerification) {
				goto(resolve('/verify-email'), { replaceState: true });
			} else {
				goto(resolve('/'), { replaceState: true });
			}
		} catch {
			status = 'error';
			errorMessage = $t('auth_callback_error');
		}
	}

	handleCallback();
</script>

{#if status === 'loading'}
	<div class="callback-container">
		<p>
			{$t(provider ? 'auth_callback_completing_provider' : 'auth_callback_completing', {
				values: { provider: provider ?? '' }
			})}
		</p>
	</div>
{:else if status === 'error'}
	<div class="callback-container">
		<p class="error">{errorMessage}</p>
		<a href={resolve('/login')}>{$t('auth_back_to_login')}</a>
	</div>
{/if}

<style>
	.callback-container {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		min-height: 60vh;
		gap: 16px;
	}

	.error {
		color: var(--text-danger, #dc3545);
	}
</style>
