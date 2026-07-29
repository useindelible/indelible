<script lang="ts">
	import type { IntegrationCallback } from '$lib/integrations/callback';
	import { findProvider } from '$lib/integrations/providers';
	import Button from '$lib/components/ui/Button.svelte';

	interface Props {
		callback: IntegrationCallback | null;
		onDismiss: () => void;
		onAction?: ((callback: IntegrationCallback) => void) | null;
	}

	let { callback, onDismiss, onAction = null }: Props = $props();

	const providerLabel = $derived.by(() => {
		if (!callback?.provider) return 'integration';
		const provider = findProvider(callback.provider);
		return provider?.displayName ?? callback.provider;
	});

	const tone = $derived.by(() => {
		if (!callback) return 'info';
		switch (callback.kind) {
			case 'success':
				return 'success';
			case 'denied':
				return 'warning';
			case 'provider_error':
			case 'server_error':
				return 'error';
		}
	});

	const heading = $derived.by(() => {
		if (!callback) return '';
		switch (callback.kind) {
			case 'success':
				return `${providerLabel} connected`;
			case 'denied':
				return `${providerLabel} connection cancelled`;
			case 'provider_error':
				return `${providerLabel} couldn't complete the connection`;
			case 'server_error':
				return `Something went wrong connecting ${providerLabel}`;
		}
	});

	const body = $derived.by(() => {
		if (!callback) return '';
		switch (callback.kind) {
			case 'success':
				return 'You can manage settings, run a manual sync, or disconnect at any time.';
			case 'denied':
				return 'You declined the authorization request, so no connection was created. Try again whenever you’re ready.';
			case 'provider_error':
				return 'The provider returned an error during sign-in. No connection was created — please try again.';
			case 'server_error':
				return 'We couldn’t finish the sign-in flow. Please try again, and contact support if it keeps happening.';
		}
	});

	const actionLabel = $derived.by(() => {
		if (!callback) return null;
		if (callback.kind === 'success' && callback.provider === 'notion') {
			return 'Open Notion settings';
		}
		return null;
	});
</script>

{#if callback}
	<aside
		class="banner banner-{tone}"
		role={tone === 'success' ? 'status' : 'alert'}
		data-testid="integration-callback-banner"
		data-kind={callback.kind}
	>
		<div class="copy">
			<h3 class="heading">{heading}</h3>
			<p class="body">{body}</p>
		</div>
		<div class="banner-actions">
			{#if actionLabel && onAction}
				<Button variant="primary" size="sm" onclick={() => onAction?.(callback)}>
					{actionLabel}
				</Button>
			{/if}
			<Button variant="tertiary" size="sm" onclick={onDismiss}>Dismiss</Button>
		</div>
	</aside>
{/if}

<style>
	.banner {
		display: flex;
		align-items: flex-start;
		gap: 16px;
		padding: 14px 16px;
		border-radius: 12px;
		border: 0.5px solid var(--border-primary);
		background: var(--bg-secondary);
		margin-bottom: 20px;
	}

	.banner-success {
		background: var(--fill-success);
		border-color: var(--fill-success);
	}

	.banner-warning {
		background: var(--fill-warning);
		border-color: var(--fill-warning);
	}

	.banner-error {
		background: var(--fill-danger);
		border-color: var(--fill-danger);
	}

	.copy {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.heading {
		font-family: var(--font-sans);
		font-size: 14px;
		font-weight: 600;
		letter-spacing: -0.01em;
		color: var(--text-primary);
		margin: 0;
	}

	.body {
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--text-secondary);
		margin: 0;
		line-height: 1.4;
	}

	.banner-actions {
		display: flex;
		gap: 8px;
		flex-shrink: 0;
		flex-wrap: wrap;
	}
</style>
