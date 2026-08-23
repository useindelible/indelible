<script lang="ts">
	import IntegrationConnectionCard from '$lib/components/integrations/IntegrationConnectionCard.svelte';
	import { t } from '$lib/i18n';

	interface Props {
		inboxAddress: string;
		feedAddress: string;
		copiedInbox: boolean;
		copiedFeed: boolean;
		onCopy: (address: string, which: 'inbox' | 'feed') => void;
	}

	let { inboxAddress, feedAddress, copiedInbox, copiedFeed, onCopy }: Props = $props();
</script>

<IntegrationConnectionCard
	title={$t('integrations_hub_email_forwarding')}
	tagline={$t('integrations_hub_email_forwarding_hint')}
	variant="banner"
	markClass="email"
	testId="email-connection-card"
>
	{#snippet mark()}
		<svg viewBox="0 0 24 24" aria-hidden="true">
			<rect x="2" y="4" width="20" height="16" rx="2" />
			<path d="M22 4L12 13 2 4" />
		</svg>
	{/snippet}
	{#snippet body()}
		<div class="addresses">
			<div class="slot">
				<span class="slot-label">{$t('integrations_hub_library_inbox')}</span>
				<div class="address-row">
					{#if inboxAddress}
						<span class="mono-address">{inboxAddress}</span>
					{:else}
						<span class="unavailable-note">{$t('email_unavailable_hint')}</span>
					{/if}
					<button
						type="button"
						class="copy-btn"
						class:copied={copiedInbox}
						disabled={!inboxAddress}
						onclick={() => onCopy(inboxAddress, 'inbox')}
						aria-label={$t('integrations_hub_copy_library_inbox')}
					>
						{#if copiedInbox}
							<svg viewBox="0 0 24 24" aria-hidden="true"><polyline points="20 6 9 17 4 12" /></svg>
							{$t('common_copied')}
						{:else}
							<svg viewBox="0 0 24 24" aria-hidden="true">
								<rect x="9" y="9" width="13" height="13" rx="2" />
								<path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
							</svg>
							{$t('common_copy')}
						{/if}
					</button>
				</div>
			</div>
			<div class="slot">
				<span class="slot-label">{$t('common_feed')}</span>
				<div class="address-row">
					{#if feedAddress}
						<span class="mono-address">{feedAddress}</span>
					{:else}
						<span class="unavailable-note">{$t('email_unavailable_hint')}</span>
					{/if}
					<button
						type="button"
						class="copy-btn"
						class:copied={copiedFeed}
						disabled={!feedAddress}
						onclick={() => onCopy(feedAddress, 'feed')}
						aria-label={$t('integrations_hub_copy_feed')}
					>
						{#if copiedFeed}
							<svg viewBox="0 0 24 24" aria-hidden="true"><polyline points="20 6 9 17 4 12" /></svg>
							{$t('common_copied')}
						{:else}
							<svg viewBox="0 0 24 24" aria-hidden="true">
								<rect x="9" y="9" width="13" height="13" rx="2" />
								<path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
							</svg>
							{$t('common_copy')}
						{/if}
					</button>
				</div>
			</div>
		</div>
	{/snippet}
</IntegrationConnectionCard>

<style>
	.addresses {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 12px;
	}

	.slot {
		display: flex;
		flex-direction: column;
		gap: 4px;
		min-width: 0;
	}

	.slot-label {
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		color: var(--text-tertiary);
	}

	.address-row {
		display: flex;
		align-items: center;
		gap: 6px;
		background: var(--int-code-bg);
		border-radius: 7px;
		padding: 6px 10px;
		overflow-x: auto;
		scrollbar-width: none;
	}

	.address-row::-webkit-scrollbar {
		display: none;
	}

	.mono-address {
		flex-shrink: 0;
		font-family: 'SF Mono', 'Fira Code', 'Menlo', ui-monospace, monospace;
		font-size: 11.5px;
		color: var(--int-code-text);
		white-space: nowrap;
		user-select: all;
	}

	.unavailable-note {
		flex: 1;
		font-size: 11.5px;
		color: var(--text-tertiary);
		line-height: 1.4;
	}

	.copy-btn:disabled {
		opacity: 0.45;
		cursor: not-allowed;
	}

	.copy-btn {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 3px 8px;
		border-radius: 6px;
		font-size: 11px;
		font-weight: 500;
		color: var(--text-secondary);
		background: var(--bg-elevated);
		box-shadow: 0 0 0 0.5px var(--border-primary);
		cursor: pointer;
		flex-shrink: 0;
		border: none;
		position: sticky;
		right: 0;
		letter-spacing: -0.005em;
	}

	.copy-btn:hover {
		color: var(--text-primary);
	}

	.copy-btn.copied {
		color: var(--int-copy-success);
	}

	.copy-btn svg {
		width: 11px;
		height: 11px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.7;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	@media (max-width: 899px) {
		.addresses {
			grid-template-columns: 1fr;
		}
	}
</style>
