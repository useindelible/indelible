<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { createDocumentEntry } from '$lib/api';
	import StepLayout from '$lib/components/onboarding/StepLayout.svelte';
	import { getAuth } from '$lib/stores/auth.svelte';
	import { getOnboarding } from '$lib/stores/onboarding.svelte';

	const onboarding = getOnboarding();
	const auth = getAuth();

	let articleUrl = $state('');
	let submitting = $state(false);
	let copied = $state<'feed' | 'library' | null>(null);
	let saveMessage = $state('');
	let saving = $state(false);

	const isValidUrl = $derived.by(() => {
		try {
			const url = new URL(articleUrl.trim());
			if (url.protocol !== 'https:' && url.protocol !== 'http:') return false;
			const parts = url.hostname.split('.').filter((part) => part.length > 0);
			const tld = parts[parts.length - 1];
			return parts.length >= 2 && tld !== undefined && tld.length >= 2;
		} catch {
			return false;
		}
	});

	const feedEmail = $derived(auth.user?.ingest_email ?? '');
	const libraryEmail = $derived(auth.user?.ingest_library_email ?? '');

	async function saveArticle() {
		if (!isValidUrl) return;
		saving = true;
		saveMessage = '';
		try {
			const { data, error } = await createDocumentEntry({ body: { url: articleUrl.trim() } });
			saveMessage =
				data && !error
					? 'Saved to your library.'
					: 'Could not save this URL. You can continue and try again later.';
			if (data && !error) articleUrl = '';
		} catch {
			saveMessage = 'Could not save this URL. You can continue and try again later.';
		} finally {
			saving = false;
		}
	}

	async function handleContinue() {
		submitting = true;
		try {
			if (await onboarding.completeStep(2)) goto(resolve('/onboarding/feeds'));
		} finally {
			submitting = false;
		}
	}

	async function handleSkip() {
		submitting = true;
		try {
			if (await onboarding.completeStep(2)) goto(resolve('/onboarding/feeds'));
		} finally {
			submitting = false;
		}
	}

	async function copyEmail(kind: 'feed' | 'library', address: string) {
		if (!address) return;
		try {
			if (navigator.clipboard?.writeText) {
				await navigator.clipboard.writeText(address);
			} else {
				copyWithSelection(address);
			}
			copied = kind;
			saveMessage = '';
			setTimeout(() => (copied = null), 2000);
		} catch {
			try {
				copyWithSelection(address);
				copied = kind;
				saveMessage = '';
				setTimeout(() => (copied = null), 2000);
			} catch {
				saveMessage = 'Could not copy the address. Select it and copy it manually.';
			}
		}
	}

	function copyWithSelection(text: string) {
		const input = document.createElement('textarea');
		input.value = text;
		input.setAttribute('readonly', '');
		input.style.position = 'fixed';
		input.style.opacity = '0';
		document.body.appendChild(input);
		input.select();
		const copiedText = document.execCommand('copy');
		input.remove();
		if (!copiedText) throw new Error('Copy command failed');
	}
</script>

<StepLayout
	title="Save your first article"
	description="Paste a URL below to save it to your library, or explore other ways to add content."
	currentStep={2}
	showSkip
	{submitting}
	onContinue={handleContinue}
	onSkip={handleSkip}
>
	<div class="content">
		<div class="url-row">
			<input
				type="url"
				class="url-input"
				bind:value={articleUrl}
				placeholder="https://example.com/article"
				aria-label="Article URL"
			/>
			<button type="button" class="save-btn" disabled={!isValidUrl || saving} onclick={saveArticle}>
				{saving ? 'Saving…' : 'Save'}
			</button>
		</div>
		{#if saveMessage}<p class="status-message">{saveMessage}</p>{/if}

		<p class="section-label">Other ways to save</p>

		<div class="method-list">
			<div class="method-card">
				<div class="method-icon" aria-hidden="true">
					<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
						<rect x="3" y="5" width="18" height="14" rx="2" />
						<path d="m3 5 9 7 9-7" stroke-linecap="round" stroke-linejoin="round" />
					</svg>
				</div>
				<div class="method-text">
					<span class="method-label">Feed email</span>
					<span class="method-desc">Forward newsletters to your feed</span>
				</div>
				{#if feedEmail}
					<div class="email-badge">
						<span class="email-address" title={feedEmail}>{feedEmail}</span>
						<button
							type="button"
							class="copy-btn"
							onclick={() => copyEmail('feed', feedEmail)}
							aria-label={copied === 'feed' ? 'Feed email copied' : 'Copy feed email address'}
						>
							<svg
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="1.7"
								aria-hidden="true"
							>
								{#if copied === 'feed'}
									<path d="m5 12 4 4 10-10" stroke-linecap="round" stroke-linejoin="round" />
								{:else}
									<rect x="9" y="9" width="12" height="12" rx="2" />
									<path
										d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"
										stroke-linecap="round"
									/>
								{/if}
							</svg>
						</button>
					</div>
				{/if}
			</div>

			<div class="method-card">
				<div class="method-icon library-icon" aria-hidden="true">
					<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
						<path d="M6 3h12v18l-6-4-6 4z" stroke-linejoin="round" />
					</svg>
				</div>
				<div class="method-text">
					<span class="method-label">Library email</span>
					<span class="method-desc">Forward articles directly to your library</span>
				</div>
				{#if libraryEmail}
					<div class="email-badge">
						<span class="email-address" title={libraryEmail}>{libraryEmail}</span>
						<button
							type="button"
							class="copy-btn"
							onclick={() => copyEmail('library', libraryEmail)}
							aria-label={copied === 'library'
								? 'Library email copied'
								: 'Copy library email address'}
						>
							<svg
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="1.7"
								aria-hidden="true"
							>
								{#if copied === 'library'}
									<path d="m5 12 4 4 10-10" stroke-linecap="round" stroke-linejoin="round" />
								{:else}
									<rect x="9" y="9" width="12" height="12" rx="2" />
									<path
										d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"
										stroke-linecap="round"
									/>
								{/if}
							</svg>
						</button>
					</div>
				{/if}
			</div>
		</div>

		{#if onboarding.error}<p class="status-message error">{onboarding.error}</p>{/if}
	</div>
</StepLayout>

<style>
	.content {
		display: flex;
		flex-direction: column;
		width: 100%;
	}

	.url-row {
		display: flex;
		margin-bottom: 8px;
		border: 1px solid var(--border-secondary);
		border-radius: 8px;
		overflow: hidden;
	}

	.url-input {
		flex: 1;
		height: 40px;
		padding: 0 14px;
		border: none;
		outline: none;
		background: var(--bg-elevated);
		color: var(--text-primary);
		font: inherit;
		font-size: 15px;
	}

	.url-input::placeholder {
		color: var(--text-secondary);
	}

	.save-btn {
		height: 40px;
		padding: 0 16px;
		border: none;
		background: var(--accent);
		color: var(--text-on-color);
		font: inherit;
		font-size: 15px;
		font-weight: 500;
		cursor: pointer;
	}

	.save-btn:hover:not(:disabled) {
		background: var(--accent-hover);
	}

	.save-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.section-label {
		margin: 24px 0 8px;
		color: var(--text-primary);
		font-size: 13px;
		font-weight: 600;
	}

	.status-message {
		margin: 0;
		color: var(--success);
		font-size: 12px;
		line-height: 1.4;
	}

	.status-message.error {
		margin-top: 10px;
		color: var(--destructive);
	}

	.method-list {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.method-card {
		display: flex;
		align-items: center;
		gap: 14px;
		min-width: 0;
		padding: 14px 16px;
		border-radius: 10px;
		background: var(--fill-secondary);
	}

	.method-icon {
		display: grid;
		place-items: center;
		width: 40px;
		height: 40px;
		flex: 0 0 40px;
		border-radius: 10px;
		background: var(--fill-success);
		color: var(--success);
	}

	.method-icon.library-icon {
		background: var(--fill-selected);
		color: var(--accent);
	}

	.method-icon svg {
		width: 20px;
		height: 20px;
	}

	.method-text {
		display: flex;
		flex: 1 1 auto;
		min-width: 0;
		flex-direction: column;
		gap: 2px;
	}

	.method-label {
		color: var(--text-primary);
		font-size: 14px;
		font-weight: 600;
	}

	.method-desc {
		color: var(--text-secondary);
		font-size: 13px;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.email-badge {
		display: flex;
		align-items: center;
		min-width: 0;
		max-width: 230px;
		gap: 6px;
	}

	.email-address {
		min-width: 0;
		padding: 5px 8px;
		border-radius: 6px;
		background: var(--fill-selected);
		color: var(--accent);
		font-family: var(--font-mono);
		font-size: 11px;
		font-weight: 500;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.copy-btn {
		display: grid;
		place-items: center;
		width: 30px;
		height: 30px;
		flex: 0 0 30px;
		padding: 0;
		border: none;
		border-radius: 6px;
		background: var(--fill-hover);
		color: var(--text-secondary);
		cursor: pointer;
	}

	.copy-btn:hover {
		background: var(--fill-selected-strong);
		color: var(--text-primary);
	}

	.copy-btn svg {
		width: 15px;
		height: 15px;
	}
</style>
