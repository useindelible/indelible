<script lang="ts">
	import { t } from '$lib/i18n';
	import { getAuth } from '$lib/stores/auth.svelte';

	const auth = getAuth();

	const ingestAddress = $derived(auth.user?.ingest_library_email ?? $t('common_loading'));

	let copied = $state(false);

	async function copyAddress() {
		if (!auth.user) return;
		try {
			await navigator.clipboard.writeText(ingestAddress);
			copied = true;
			setTimeout(() => {
				copied = false;
			}, 2000);
		} catch {
			// Clipboard API unavailable in some environments; graceful degradation.
		}
	}
</script>

<div class="email-tab">
	<p class="email-description">
		{$t('library_email_description')}
	</p>

	<div class="address-row">
		<span class="address-value" aria-label={$t('library_email_ingest_address')}
			>{ingestAddress}</span
		>
		<button
			type="button"
			class="copy-btn"
			onclick={copyAddress}
			aria-label={$t('library_email_copy_ingest')}
		>
			{#if copied}
				<svg
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<polyline points="20 6 9 17 4 12" />
				</svg>
				{$t('common_copied')}
			{:else}
				<svg
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
					<path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
				</svg>
				{$t('common_copy')}
			{/if}
		</button>
	</div>

	<p class="email-note">
		{$t('library_email_note')}
	</p>
</div>

<style>
	.email-tab {
		display: flex;
		flex-direction: column;
		gap: 16px;
		padding-bottom: 8px;
	}

	.email-description {
		font-size: 14px;
		color: var(--text-secondary);
		font-family: var(--font-sans);
		letter-spacing: -0.01em;
		line-height: 1.5;
		margin: 0;
	}

	.address-row {
		display: flex;
		align-items: center;
		gap: 10px;
		background: var(--input-bg, var(--bg-secondary));
		border: 1px solid var(--border-primary);
		border-radius: 10px;
		padding: 10px 14px;
	}

	.address-value {
		flex: 1;
		font-size: 14px;
		font-family: var(--font-mono, monospace);
		color: var(--text-primary);
		letter-spacing: -0.01em;
		word-break: break-all;
	}

	.copy-btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 6px 12px;
		border-radius: 7px;
		border: 1px solid var(--border-secondary);
		background: transparent;
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 500;
		color: var(--text-secondary);
		cursor: pointer;
		white-space: nowrap;
		transition:
			background 120ms ease,
			color 120ms ease;
		flex-shrink: 0;
	}

	.copy-btn:hover {
		background: var(--fill-hover);
		color: var(--text-primary);
	}

	.copy-btn svg {
		width: 14px;
		height: 14px;
	}

	.email-note {
		font-size: 12px;
		color: var(--text-tertiary);
		font-family: var(--font-sans);
		letter-spacing: -0.005em;
		line-height: 1.5;
		margin: 0;
	}
</style>
