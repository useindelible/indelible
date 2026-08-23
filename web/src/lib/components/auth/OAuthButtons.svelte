<script lang="ts">
	import { api, listProviders, type OAuthProviderInfo } from '$lib/api';
	import AuthDivider from './AuthDivider.svelte';
	import { t } from '$lib/i18n';

	interface Props {
		dividerText?: string;
	}

	let { dividerText }: Props = $props();

	type OAuthProvider = OAuthProviderInfo;

	let providers = $state<OAuthProvider[]>([]);
	let loadingProvider = $state<string | null>(null);
	let loaded = $state(false);

	async function fetchProviders() {
		try {
			const { data } = await listProviders();
			if (data) {
				providers = data.providers.filter((provider) => provider.enabled);
			}
		} catch {
			providers = [];
		} finally {
			loaded = true;
		}
	}

	fetchProviders();

	function startOAuth(providerId: string) {
		loadingProvider = providerId;
		const baseUrl = api.getConfig().baseUrl?.replace(/\/$/, '') ?? '';
		window.location.href = `${baseUrl}/api/v1/auth/oauth/${encodeURIComponent(providerId)}/start`;
	}

	function getProviderLabel(provider: OAuthProvider): string {
		return $t('auth_continue_with', { values: { provider: provider.name } });
	}
</script>

{#if loaded && providers.length > 0}
	<AuthDivider text={dividerText ?? $t('auth_or')} />
	<div class="oauth-buttons">
		{#each providers as provider (provider.id)}
			<button
				type="button"
				class="oauth-button"
				disabled={loadingProvider !== null}
				onclick={() => startOAuth(provider.id)}
			>
				<span class="oauth-icon" aria-hidden="true">
					{#if provider.id === 'google'}
						<svg width="18" height="18" viewBox="0 0 24 24">
							<path
								d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92a5.06 5.06 0 0 1-2.2 3.32v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.1z"
								fill="#4285F4"
							/>
							<path
								d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"
								fill="#34A853"
							/>
							<path
								d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18A10.96 10.96 0 0 0 1 12c0 1.77.42 3.45 1.18 4.93l3.66-2.84z"
								fill="#FBBC05"
							/>
							<path
								d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"
								fill="#EA4335"
							/>
						</svg>
					{:else if provider.id === 'apple'}
						<svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
							<path
								d="M17.05 20.28c-.98.95-2.05.88-3.08.4-1.09-.5-2.08-.48-3.24 0-1.44.62-2.2.44-3.06-.4C2.79 15.25 3.51 7.59 9.05 7.31c1.35.07 2.29.74 3.08.8 1.18-.24 2.31-.93 3.57-.84 1.51.12 2.65.72 3.4 1.8-3.12 1.87-2.38 5.98.48 7.13-.57 1.5-1.31 2.99-2.54 4.09zM12.03 7.25c-.15-2.23 1.66-4.07 3.74-4.25.32 2.32-2.12 4.53-3.74 4.25z"
							/>
						</svg>
					{/if}
				</span>
				<span>{getProviderLabel(provider)}</span>
			</button>
		{/each}
	</div>
{/if}

<style>
	.oauth-buttons {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.oauth-button {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 10px;
		width: 100%;
		padding: 10px 20px;
		font-family: var(--font-sans);
		font-size: 15px;
		font-weight: 500;
		letter-spacing: -0.01em;
		line-height: 1.5;
		color: var(--text-primary);
		background: var(--bg-primary);
		border: 1px solid var(--border-secondary);
		border-radius: var(--radius-sm);
		cursor: pointer;
		transition:
			background-color 0.15s ease,
			border-color 0.15s ease;
	}

	.oauth-button:hover:not(:disabled) {
		background: var(--fill-hover);
		border-color: var(--border-primary);
	}

	.oauth-button:disabled {
		opacity: 0.55;
		cursor: not-allowed;
	}

	.oauth-icon {
		font-size: 18px;
		line-height: 1;
	}
</style>
