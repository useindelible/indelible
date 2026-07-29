<script lang="ts">
	import { subscribe } from '$lib/api';
	import type { FeedSubscriptionResponse, OpmlImportResponse } from '$lib/api';
	import { getAuth } from '$lib/stores/auth.svelte';
	import { uploadOpml } from '$lib/api/feeds';
	import { resolve } from '$app/paths';

	const auth = getAuth();

	let feedUrl = $state('');
	let subscribing = $state(false);
	let subscribeError = $state<string | null>(null);
	let subscribeResult = $state<{ subscription: FeedSubscriptionResponse; isNew: boolean } | null>(
		null
	);

	let opmlFile = $state<File | null>(null);
	let opmlUploading = $state(false);
	let opmlError = $state<string | null>(null);
	let opmlResult = $state<OpmlImportResponse | null>(null);
	let isDragOver = $state(false);
	let opmlInputEl = $state<HTMLInputElement | undefined>(undefined);

	let copied = $state(false);

	const newsletterEmail = $derived(auth.user?.ingest_email ?? null);

	function isValidUrl(input: string): boolean {
		try {
			const url = new URL(input);
			return url.protocol === 'http:' || url.protocol === 'https:';
		} catch {
			return false;
		}
	}

	async function handleSubscribe() {
		const url = feedUrl.trim();
		if (!url) return;
		if (!isValidUrl(url)) {
			subscribeError = 'Please enter a valid URL starting with http:// or https://';
			return;
		}

		subscribing = true;
		subscribeError = null;
		subscribeResult = null;

		try {
			const { data, error: apiError, response } = await subscribe({ body: { url } });
			if (data) {
				subscribeResult = {
					subscription: data.subscription,
					isNew: data.is_new
				};
				feedUrl = '';
			} else {
				subscribeError = extractErrorMessage(apiError, response, 'Failed to subscribe to feed');
			}
		} catch {
			subscribeError = 'An unexpected error occurred';
		} finally {
			subscribing = false;
		}
	}

	function extractErrorMessage(
		apiError: unknown,
		response: Response | undefined,
		fallback: string
	): string {
		if (response?.status === 422) {
			const err = apiError as Record<string, unknown> | undefined;
			return (err?.detail as string) ?? (err?.message as string) ?? 'Invalid feed URL';
		}
		if (apiError && typeof apiError === 'object') {
			const err = apiError as Record<string, unknown>;
			return (err.detail as string) ?? (err.message as string) ?? fallback;
		}
		return fallback;
	}

	async function handleOpmlUpload(file: File) {
		opmlFile = file;
		opmlUploading = true;
		opmlError = null;
		opmlResult = null;

		try {
			const result = await uploadOpml(file);
			if (result.ok) {
				opmlResult = result.data;
			} else {
				opmlError = result.error;
			}
		} catch {
			opmlError = 'An unexpected error occurred during upload';
		} finally {
			opmlUploading = false;
		}
	}

	function handleOpmlDrop(e: DragEvent) {
		e.preventDefault();
		isDragOver = false;
		const file = e.dataTransfer?.files[0];
		if (file) handleOpmlUpload(file);
	}

	function handleOpmlFileSelect(e: Event) {
		const input = e.target as HTMLInputElement;
		const file = input.files?.[0];
		if (file) handleOpmlUpload(file);
		input.value = '';
	}

	async function copyEmail() {
		if (!newsletterEmail) return;
		try {
			await navigator.clipboard.writeText(newsletterEmail);
			copied = true;
			setTimeout(() => {
				copied = false;
			}, 2000);
		} catch {
			// clipboard API unavailable — silently ignore
		}
	}

	function extractDomain(url: string): string {
		try {
			return new URL(url).hostname;
		} catch {
			return url;
		}
	}
</script>

<div class="settings-content">
	<h1 class="settings-title">Add to Feed</h1>

	<div class="settings-section">
		<h2 class="section-heading">Subscribe to Feed</h2>
		<p class="section-desc">Enter an RSS, Atom, or YouTube channel URL to subscribe.</p>

		<form
			class="subscribe-form"
			onsubmit={(e) => {
				e.preventDefault();
				handleSubscribe();
			}}
		>
			<input
				type="url"
				class="text-input subscribe-input"
				placeholder="https://example.com/feed.xml"
				bind:value={feedUrl}
				disabled={subscribing}
			/>
			<button type="submit" class="btn-primary" disabled={subscribing || !feedUrl.trim()}>
				{subscribing ? 'Subscribing...' : 'Subscribe'}
			</button>
		</form>

		{#if subscribeError}
			<p class="form-error">{subscribeError}</p>
		{/if}

		{#if subscribeResult}
			{@const sub = subscribeResult.subscription}
			<div class="subscribe-result" class:already-subscribed={!subscribeResult.isNew}>
				{#if subscribeResult.isNew}
					<div class="result-preview">
						{#if sub.source.image_url}
							<img class="result-icon" src={sub.source.image_url} alt="" />
						{:else}
							<div class="result-icon-placeholder">
								{(sub.title_override ?? sub.source.name).substring(0, 2).toUpperCase()}
							</div>
						{/if}
						<div class="result-info">
							<span class="result-name">{sub.title_override ?? sub.source.name}</span>
							<span class="result-domain">{sub.source.domain ?? extractDomain(sub.input_url)}</span>
							<span class="result-kind">{sub.source.source_kind}</span>
						</div>
					</div>
					<p class="result-message">
						Subscribed. <a href={resolve('/preferences/feed-management')}>Go to Feed Management</a>
					</p>
				{:else}
					<p class="result-message">
						Already subscribed. <a href={resolve('/preferences/feed-management')}
							>Manage in Feed Management</a
						>
					</p>
				{/if}
			</div>
		{/if}
	</div>

	<div class="section-divider"></div>

	<div class="settings-section">
		<h2 class="section-heading">Import OPML</h2>
		<p class="section-desc">Upload an OPML file to import feeds from another service.</p>

		<div
			class="drop-zone"
			class:drag-over={isDragOver}
			role="button"
			tabindex="0"
			aria-label="Drop OPML file to import feeds"
			ondrop={handleOpmlDrop}
			ondragover={(e) => {
				e.preventDefault();
				isDragOver = true;
			}}
			ondragleave={() => {
				isDragOver = false;
			}}
			onclick={() => opmlInputEl?.click()}
			onkeydown={(e) => {
				if (e.key === 'Enter' || e.key === ' ') {
					e.preventDefault();
					opmlInputEl?.click();
				}
			}}
		>
			<input
				bind:this={opmlInputEl}
				type="file"
				accept=".opml,.xml"
				aria-hidden="true"
				tabindex="-1"
				style="display:none"
				onchange={handleOpmlFileSelect}
			/>
			{#if opmlUploading}
				<span class="dz-text">Uploading {opmlFile?.name}...</span>
			{:else}
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
					<polyline points="17 8 12 3 7 8" />
					<line x1="12" y1="3" x2="12" y2="15" />
				</svg>
				<span class="dz-text">Drop OPML file here or click to browse</span>
				<span class="dz-formats">Supported: .opml, .xml</span>
			{/if}
		</div>

		{#if opmlError}
			<p class="form-error">{opmlError}</p>
		{/if}

		{#if opmlResult}
			<div class="opml-result">
				<p class="opml-summary">
					Imported {opmlResult.created} feed{opmlResult.created === 1 ? '' : 's'}, skipped {opmlResult.skipped}
				</p>
				{#if opmlResult.errors.length > 0}
					<details class="opml-errors">
						<summary
							>{opmlResult.errors.length} error{opmlResult.errors.length === 1 ? '' : 's'}</summary
						>
						<ul>
							{#each opmlResult.errors as err, i (i)}
								<li>{err}</li>
							{/each}
						</ul>
					</details>
				{/if}
			</div>
		{/if}
	</div>

	<div class="section-divider"></div>

	<div class="settings-section">
		<h2 class="section-heading">Newsletter Email</h2>
		<p class="section-desc">
			Subscribe to newsletters with this email address. Incoming emails will appear in your Feed.
		</p>

		<div class="email-display-row">
			<code class="email-address">{newsletterEmail ?? '...'}</code>
			<button
				type="button"
				class="copy-btn"
				disabled={!newsletterEmail}
				onclick={copyEmail}
				aria-label="Copy email address"
			>
				{#if copied}
					<svg viewBox="0 0 24 24" aria-hidden="true">
						<polyline points="20 6 9 17 4 12" />
					</svg>
					Copied
				{:else}
					<svg viewBox="0 0 24 24" aria-hidden="true">
						<rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
						<path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
					</svg>
					Copy
				{/if}
			</button>
		</div>
	</div>
</div>

<style>
	.settings-content {
		padding: 32px 40px 48px;
	}

	.settings-title {
		font-size: 28px;
		font-weight: 700;
		letter-spacing: -0.03em;
		color: var(--text-primary);
		font-family: var(--font-sans);
		margin: 0 0 28px;
	}

	.settings-section {
		margin-bottom: 0;
	}

	.section-heading {
		font-size: 17px;
		font-weight: 600;
		letter-spacing: -0.01em;
		color: var(--text-primary);
		font-family: var(--font-sans);
		margin: 0 0 4px;
	}

	.section-desc {
		font-size: 13px;
		font-weight: 400;
		color: var(--text-secondary);
		font-family: var(--font-sans);
		margin: 0 0 12px;
		line-height: 1.45;
	}

	.section-divider {
		height: 0.5px;
		background: var(--border-primary);
		margin: 24px 0;
	}

	/* Subscribe form */
	.subscribe-form {
		display: flex;
		gap: 8px;
		align-items: center;
	}

	.text-input {
		height: 34px;
		border-radius: 8px;
		border: 1px solid rgba(0, 0, 0, 0.08);
		background: #ffffff;
		box-shadow: 0 0.5px 1px rgba(0, 0, 0, 0.04);
		padding: 0 10px;
		font-family: var(--font-sans);
		font-size: 13px;
		letter-spacing: -0.01em;
		color: var(--text-primary);
		outline: none;
		transition:
			border-color 120ms ease,
			box-shadow 120ms ease;
		box-sizing: border-box;
	}

	:global([data-theme='dark']) .text-input {
		background: var(--bg-tertiary);
		border-color: var(--border-primary);
		box-shadow: none;
	}

	.text-input::placeholder {
		color: var(--text-tertiary);
	}

	.text-input:focus {
		border-color: var(--accent);
		box-shadow:
			0 0 0 3.5px rgba(0, 113, 227, 0.1),
			0 0.5px 1px rgba(0, 0, 0, 0.04);
	}

	.subscribe-input {
		flex: 1;
	}

	.btn-primary {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 8px 16px;
		border-radius: 8px;
		background: var(--accent);
		color: #fff;
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 500;
		border: none;
		cursor: pointer;
		letter-spacing: -0.01em;
		transition: opacity 120ms ease;
		box-shadow: 0 1px 3px rgba(0, 113, 227, 0.2);
		flex-shrink: 0;
		height: 34px;
	}

	.btn-primary:hover:not(:disabled) {
		opacity: 0.88;
	}
	.btn-primary:disabled {
		opacity: 0.45;
		cursor: not-allowed;
	}

	/* Form error */
	.form-error {
		font-size: 13px;
		font-weight: 400;
		color: var(--destructive);
		font-family: var(--font-sans);
		margin: 8px 0 0;
		line-height: 1.4;
	}

	/* Subscribe result */
	.subscribe-result {
		margin-top: 12px;
		padding: 14px 16px;
		border-radius: 10px;
		background: var(--bg-secondary);
		box-shadow: inset 0 0 0 0.5px rgba(0, 0, 0, 0.04);
	}

	:global([data-theme='dark']) .subscribe-result {
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
	}

	.result-preview {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.result-icon {
		width: 40px;
		height: 40px;
		border-radius: 10px;
		object-fit: cover;
		flex-shrink: 0;
	}

	.result-icon-placeholder {
		width: 40px;
		height: 40px;
		border-radius: 10px;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
		font-size: 13px;
		font-weight: 700;
		font-family: var(--font-sans);
		background: rgba(0, 113, 227, 0.08);
		color: var(--accent);
	}

	.result-info {
		display: flex;
		flex-direction: column;
		gap: 1px;
		min-width: 0;
	}

	.result-name {
		font-size: 14px;
		font-weight: 600;
		letter-spacing: -0.01em;
		color: var(--text-primary);
		font-family: var(--font-sans);
	}

	.result-domain {
		font-size: 12px;
		font-weight: 400;
		color: var(--text-secondary);
		font-family: var(--font-sans);
	}

	.result-kind {
		font-size: 11px;
		font-weight: 500;
		color: var(--text-tertiary);
		font-family: var(--font-sans);
		text-transform: capitalize;
	}

	.result-message {
		font-size: 13px;
		font-weight: 400;
		color: var(--text-secondary);
		font-family: var(--font-sans);
		margin: 10px 0 0;
		line-height: 1.4;
	}

	.already-subscribed .result-message {
		margin-top: 0;
	}

	.result-message a {
		color: var(--accent);
		text-decoration: none;
	}

	.result-message a:hover {
		text-decoration: underline;
	}

	/* OPML drop zone */
	.drop-zone {
		border-radius: 10px;
		border: 1.5px dashed var(--border-secondary);
		padding: 24px;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 4px;
		cursor: pointer;
		transition: border-color 150ms ease;
	}

	.drop-zone:hover,
	.drop-zone.drag-over {
		border-color: var(--accent);
	}

	.drop-zone svg {
		width: 24px;
		height: 24px;
		stroke: var(--text-tertiary);
		stroke-width: 1.5;
		fill: none;
		stroke-linecap: round;
		stroke-linejoin: round;
		margin-bottom: 4px;
	}

	.dz-text {
		font-size: 13px;
		font-weight: 400;
		color: var(--text-secondary);
		font-family: var(--font-sans);
	}

	.dz-formats {
		font-size: 11px;
		font-weight: 400;
		color: var(--text-tertiary);
		font-family: var(--font-sans);
	}

	/* OPML result */
	.opml-result {
		margin-top: 12px;
		padding: 14px 16px;
		border-radius: 10px;
		background: var(--bg-secondary);
		box-shadow: inset 0 0 0 0.5px rgba(0, 0, 0, 0.04);
	}

	:global([data-theme='dark']) .opml-result {
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
	}

	.opml-summary {
		font-size: 14px;
		font-weight: 500;
		color: var(--text-primary);
		font-family: var(--font-sans);
		margin: 0;
	}

	.opml-errors {
		margin-top: 8px;
		font-size: 12px;
		color: var(--text-secondary);
		font-family: var(--font-sans);
	}

	.opml-errors summary {
		cursor: pointer;
		color: var(--destructive);
		font-weight: 500;
	}

	.opml-errors ul {
		margin: 4px 0 0;
		padding-left: 18px;
	}

	.opml-errors li {
		line-height: 1.5;
		color: var(--text-secondary);
	}

	/* Newsletter email */
	.email-display-row {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.email-address {
		flex: 1;
		padding: 9px 12px;
		border-radius: 8px;
		background: var(--bg-secondary);
		box-shadow: inset 0 0 0 0.5px rgba(0, 0, 0, 0.04);
		font-family: var(--font-mono, 'SF Mono', 'Menlo', monospace);
		font-size: 13px;
		color: var(--text-primary);
		letter-spacing: 0;
		user-select: all;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	:global([data-theme='dark']) .email-address {
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
	}

	.copy-btn {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 8px 14px;
		border-radius: 8px;
		border: 1px solid rgba(0, 0, 0, 0.08);
		background: #ffffff;
		box-shadow: 0 0.5px 1px rgba(0, 0, 0, 0.04);
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 500;
		color: var(--text-primary);
		cursor: pointer;
		flex-shrink: 0;
		height: 34px;
		transition:
			background 120ms ease,
			border-color 120ms ease;
		letter-spacing: -0.01em;
	}

	:global([data-theme='dark']) .copy-btn {
		background: var(--bg-tertiary);
		border-color: var(--border-primary);
		box-shadow: none;
	}

	.copy-btn:hover:not(:disabled) {
		background: var(--fill-hover);
	}

	.copy-btn:disabled {
		opacity: 0.45;
		cursor: not-allowed;
	}

	.copy-btn svg {
		width: 14px;
		height: 14px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.75;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	@media (max-width: 599px) {
		.settings-content {
			padding: 20px 16px 40px;
		}

		.settings-title {
			font-size: 22px;
		}
	}
</style>
