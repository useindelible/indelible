<script lang="ts">
	import * as apiSdk from '$lib/api';
	import { getModalStore } from '$lib/stores/addItemModal.svelte';

	type AuthType = 'none' | 'basic' | 'bearer' | 'apikey';

	const modal = getModalStore();

	let dialogEl = $state<HTMLDialogElement | undefined>(undefined);
	let inputEl = $state<HTMLInputElement | undefined>(undefined);
	let feedUrl = $state('');
	let submitting = $state(false);
	let submitError = $state('');
	let isDragOver = $state(false);
	let opmlInputEl = $state<HTMLInputElement | undefined>(undefined);

	// Auth state
	let showAuth = $state(false);
	let authType = $state<AuthType>('none');
	let authUsername = $state('');
	let authPassword = $state('');
	let authToken = $state('');
	let authKeyName = $state('');
	let authKeyValue = $state('');

	const isOpen = $derived(modal.active === 'rss');
	const canSubscribe = $derived(feedUrl.trim().length > 0 && !submitting);

	$effect(() => {
		if (!dialogEl) return;
		if (isOpen) {
			feedUrl = '';
			submitting = false;
			submitError = '';
			isDragOver = false;
			showAuth = false;
			authType = 'none';
			authUsername = '';
			authPassword = '';
			authToken = '';
			authKeyName = '';
			authKeyValue = '';
			dialogEl.showModal();
			setTimeout(() => inputEl?.focus(), 50);
		} else {
			dialogEl.close();
		}
	});

	function close() {
		modal.close();
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === dialogEl) close();
	}

	function handleOpmlDrop(e: DragEvent) {
		e.preventDefault();
		isDragOver = false;
		// OPML import handled when backend endpoint exists
	}

	async function handleSubscribe() {
		if (!canSubscribe) return;
		submitting = true;
		submitError = '';
		try {
			const { data, error } = await apiSdk.subscribe({
				body: { url: feedUrl.trim() }
			});
			if (data) {
				modal.notifySubscribed();
				close();
				return;
			}
			const problem = error as Record<string, unknown> | null | undefined;
			const detail = typeof problem?.['detail'] === 'string' ? problem['detail'] : undefined;
			const errors = Array.isArray(problem?.['errors']) ? problem['errors'] : undefined;
			const firstError = errors?.[0] as Record<string, unknown> | undefined;
			submitError =
				(typeof firstError?.['message'] === 'string' ? firstError['message'] : undefined) ??
				detail ??
				'Failed to subscribe. Please try again.';
		} catch (err) {
			submitError = err instanceof Error ? err.message : 'An unexpected error occurred.';
		} finally {
			submitting = false;
		}
	}
</script>

<dialog
	bind:this={dialogEl}
	class="modal-backdrop"
	aria-label="Add RSS Feed"
	onclick={handleBackdropClick}
	onclose={close}
>
	<div class="cmd-card" role="document">
		<!-- URL input -->
		<div class="cmd-input-zone">
			<div class="cmd-input-wrap">
				<svg class="cmd-icon" viewBox="0 0 24 24" aria-hidden="true">
					<path d="M4 11a9 9 0 0 1 9 9" />
					<path d="M4 4a16 16 0 0 1 16 16" />
					<circle cx="5" cy="19" r="1" fill="currentColor" stroke="none" />
				</svg>
				<input
					bind:this={inputEl}
					bind:value={feedUrl}
					class="cmd-input"
					type="url"
					placeholder="Feed URL or website address…"
					autocomplete="off"
					onkeydown={(e) => {
						if (e.key === 'Enter') handleSubscribe();
						else if (e.key === 'Escape') close();
					}}
				/>
			</div>
		</div>

		<div class="cmd-body">
			{#if submitError}
				<p class="error-text" role="alert">{submitError}</p>
			{/if}

			<!-- Auth disclosure -->
			<button
				type="button"
				class="auth-toggle"
				class:open={showAuth}
				onclick={() => {
					showAuth = !showAuth;
				}}
			>
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
					<path d="M7 11V7a5 5 0 0 1 10 0v4" />
				</svg>
				Requires authentication
				<svg class="chev" viewBox="0 0 24 24" aria-hidden="true">
					<polyline points="6 9 12 15 18 9" />
				</svg>
			</button>

			{#if showAuth}
				<div class="auth-section">
					<div class="auth-type-row">
						<span class="auth-type-label">Type</span>
						<div class="auth-type-tabs">
							<button
								type="button"
								class="auth-tab"
								class:active={authType === 'none'}
								onclick={() => (authType = 'none')}>None</button
							>
							<button
								type="button"
								class="auth-tab"
								class:active={authType === 'basic'}
								onclick={() => (authType = 'basic')}>Basic</button
							>
							<button
								type="button"
								class="auth-tab"
								class:active={authType === 'bearer'}
								onclick={() => (authType = 'bearer')}>Token</button
							>
							<button
								type="button"
								class="auth-tab"
								class:active={authType === 'apikey'}
								onclick={() => (authType = 'apikey')}>API Key</button
							>
						</div>
					</div>

					{#if authType === 'basic'}
						<div class="auth-fields">
							<input
								type="text"
								class="auth-input"
								placeholder="Username"
								bind:value={authUsername}
								autocomplete="username"
							/>
							<input
								type="password"
								class="auth-input"
								placeholder="Password"
								bind:value={authPassword}
								autocomplete="current-password"
							/>
						</div>
						<p class="auth-hint">Used for feeds protected by HTTP Basic authentication.</p>
					{:else if authType === 'bearer'}
						<div class="auth-fields">
							<input
								type="text"
								class="auth-input"
								placeholder="Bearer token"
								bind:value={authToken}
								autocomplete="off"
							/>
						</div>
						<p class="auth-hint">Sent as <code>Authorization: Bearer …</code> with each request.</p>
					{:else if authType === 'apikey'}
						<div class="auth-fields auth-fields-row">
							<input
								type="text"
								class="auth-input auth-input-key"
								placeholder="Header name"
								bind:value={authKeyName}
								autocomplete="off"
							/>
							<input
								type="text"
								class="auth-input auth-input-val"
								placeholder="Value"
								bind:value={authKeyValue}
								autocomplete="off"
							/>
						</div>
						<p class="auth-hint">E.g. header <code>X-API-Key</code> with your key value.</p>
					{:else}
						<p class="auth-hint auth-hint-none">
							Most feeds are public. Use authentication only if the feed requires a login, API key,
							or private token.
						</p>
					{/if}
				</div>
			{/if}

			<!-- OPML import -->
			<div class="cmd-divider">
				<div class="cmd-divider-line"></div>
				<span class="cmd-divider-text">or import</span>
				<div class="cmd-divider-line"></div>
			</div>

			<div
				class="opml-row"
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
				/>
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
					<polyline points="17 8 12 3 7 8" />
					<line x1="12" y1="3" x2="12" y2="15" />
				</svg>
				Drop an <span>OPML file</span> to import feeds
			</div>
		</div>

		<!-- Controls strip -->
		<div class="cmd-controls">
			<button type="button" class="cmd-collection" aria-label="Choose collection">
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
				</svg>
				Inbox
				<svg class="chev" viewBox="0 0 24 24" aria-hidden="true"
					><polyline points="9 6 15 12 9 18" /></svg
				>
			</button>

			<button type="button" class="cmd-action" disabled={!canSubscribe} onclick={handleSubscribe}>
				{#if submitting}
					<span class="spinner" aria-hidden="true"></span>
					<span class="sr-only">Subscribing...</span>
				{:else}
					Subscribe
				{/if}
			</button>
		</div>
	</div>
</dialog>

<style>
	.modal-backdrop {
		position: fixed;
		inset: 0;
		width: 100%;
		height: 100%;
		max-width: 100%;
		max-height: 100%;
		margin: 0;
		padding: 0;
		border: none;
		background: transparent;
		display: flex;
		align-items: flex-start;
		justify-content: center;
		padding-top: 80px;
		box-sizing: border-box;
	}

	.modal-backdrop::backdrop {
		background: rgba(0, 0, 0, 0.4);
		backdrop-filter: blur(4px);
		-webkit-backdrop-filter: blur(4px);
	}

	.cmd-card {
		width: 460px;
		max-width: calc(100vw - 32px);
		background: var(--bg-elevated);
		border-radius: 14px;
		box-shadow:
			0 24px 80px rgba(0, 0, 0, 0.22),
			0 0 0 0.5px rgba(0, 0, 0, 0.06);
		overflow: hidden;
	}

	[data-theme='dark'] .cmd-card {
		box-shadow:
			0 24px 80px rgba(0, 0, 0, 0.55),
			0 0 0 0.5px rgba(255, 255, 255, 0.08);
	}

	.cmd-input-zone {
		padding: 8px 8px 0;
	}

	.cmd-input-wrap {
		position: relative;
	}

	.cmd-icon {
		position: absolute;
		left: 14px;
		top: 50%;
		transform: translateY(-50%);
		width: 16px;
		height: 16px;
		stroke: var(--text-tertiary);
		fill: none;
		stroke-width: 1.5;
		stroke-linecap: round;
		stroke-linejoin: round;
		pointer-events: none;
	}

	.cmd-input {
		width: 100%;
		height: 48px;
		border-radius: 10px;
		background: var(--bg-secondary);
		border: none;
		padding: 0 16px 0 40px;
		font-family: var(--font-sans);
		font-size: 15px;
		color: var(--text-primary);
		outline: none;
		letter-spacing: -0.01em;
	}

	.cmd-input::placeholder {
		color: var(--text-tertiary);
	}

	.cmd-body {
		padding: 8px 8px 4px;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.error-text {
		font-size: 12px;
		color: var(--destructive);
		font-family: var(--font-sans);
		margin: 4px 8px 0;
	}

	/* Auth disclosure toggle */
	.auth-toggle {
		display: flex;
		align-items: center;
		gap: 6px;
		width: 100%;
		padding: 7px 10px;
		border-radius: 8px;
		border: none;
		background: none;
		cursor: pointer;
		font-family: var(--font-sans);
		font-size: 12px;
		font-weight: 500;
		color: var(--text-secondary);
		text-align: left;
		transition:
			background 120ms ease,
			color 120ms ease;
	}

	.auth-toggle:hover {
		background: var(--fill-hover);
		color: var(--text-primary);
	}

	.auth-toggle.open {
		color: var(--accent);
	}

	.auth-toggle svg:first-child {
		width: 13px;
		height: 13px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.75;
		stroke-linecap: round;
		stroke-linejoin: round;
		flex-shrink: 0;
	}

	.auth-toggle .chev {
		width: 11px;
		height: 11px;
		stroke: var(--text-quaternary);
		fill: none;
		stroke-width: 2;
		stroke-linecap: round;
		stroke-linejoin: round;
		margin-left: auto;
		transition: transform 150ms ease;
	}

	.auth-toggle.open .chev {
		transform: rotate(180deg);
	}

	/* Auth section */
	.auth-section {
		background: var(--bg-secondary);
		border-radius: 10px;
		padding: 12px;
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin: 0 0 2px;
	}

	.auth-type-row {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.auth-type-label {
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.05em;
		text-transform: uppercase;
		color: var(--text-tertiary);
		font-family: var(--font-sans);
		flex-shrink: 0;
	}

	.auth-type-tabs {
		display: flex;
		gap: 2px;
		background: var(--fill-hover);
		border-radius: 7px;
		padding: 2px;
	}

	.auth-tab {
		padding: 4px 10px;
		border-radius: 5px;
		border: none;
		background: none;
		font-family: var(--font-sans);
		font-size: 12px;
		font-weight: 400;
		color: var(--text-secondary);
		cursor: pointer;
		transition:
			background 100ms ease,
			color 100ms ease;
	}

	.auth-tab.active {
		background: var(--bg-elevated);
		color: var(--text-primary);
		font-weight: 500;
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
	}

	.auth-fields {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.auth-fields-row {
		flex-direction: row;
	}

	.auth-input {
		width: 100%;
		height: 36px;
		border-radius: 8px;
		border: 1px solid var(--border-secondary);
		background: var(--bg-elevated);
		padding: 0 10px;
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--text-primary);
		outline: none;
		letter-spacing: -0.005em;
		transition: border-color 120ms ease;
		box-sizing: border-box;
	}

	.auth-input:focus {
		border-color: var(--accent);
	}

	.auth-input::placeholder {
		color: var(--text-tertiary);
	}

	.auth-input-key {
		flex: 0 0 44%;
	}

	.auth-input-val {
		flex: 1;
	}

	.auth-hint {
		font-size: 11px;
		font-weight: 400;
		color: var(--text-tertiary);
		font-family: var(--font-sans);
		line-height: 1.45;
		margin: 0;
	}

	.auth-hint code {
		font-family: 'SF Mono', 'Fira Code', 'Menlo', monospace;
		font-size: 10.5px;
		background: var(--fill-hover);
		border-radius: 3px;
		padding: 0 3px;
	}

	.auth-hint-none {
		color: var(--text-quaternary);
	}

	/* Divider */
	.cmd-divider {
		display: flex;
		align-items: center;
		gap: 8px;
		margin: 6px 8px 2px;
	}

	.cmd-divider-line {
		flex: 1;
		height: 0.5px;
		background: var(--border-primary);
	}

	.cmd-divider-text {
		font-size: 10px;
		color: var(--text-tertiary);
		white-space: nowrap;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		font-weight: 500;
		font-family: var(--font-sans);
	}

	.opml-row {
		border-radius: 8px;
		border: 1.5px dashed var(--border-secondary);
		padding: 10px 12px;
		display: flex;
		align-items: center;
		gap: 8px;
		cursor: pointer;
		background: var(--bg-secondary);
		margin: 0 0 2px;
		font-size: 12px;
		color: var(--text-secondary);
		font-family: var(--font-sans);
		transition: border-color 0.15s ease;
	}

	.opml-row:hover,
	.opml-row.drag-over {
		border-color: var(--accent);
	}

	.opml-row svg {
		width: 14px;
		height: 14px;
		stroke: var(--text-tertiary);
		stroke-width: 1.5;
		fill: none;
		stroke-linecap: round;
		stroke-linejoin: round;
		flex-shrink: 0;
	}

	.opml-row span {
		color: var(--accent);
		font-weight: 500;
	}

	/* Controls */
	.cmd-controls {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 10px 16px 14px;
	}

	.cmd-collection {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 5px 8px 5px 7px;
		border-radius: 7px;
		background: var(--bg-secondary);
		border: none;
		cursor: pointer;
		font-family: var(--font-sans);
		font-size: 12px;
		font-weight: 500;
		color: var(--text-primary);
		flex-shrink: 0;
	}

	.cmd-collection:hover {
		background: var(--bg-tertiary);
	}

	.cmd-collection svg {
		width: 13px;
		height: 13px;
		stroke: var(--text-secondary);
		fill: none;
		stroke-width: 1.5;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.cmd-collection .chev {
		width: 10px;
		height: 10px;
		stroke: var(--text-quaternary);
		stroke-width: 2;
		margin-left: 2px;
	}

	.cmd-action {
		margin-left: auto;
		padding: 6px 16px;
		border-radius: 980px;
		border: none;
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 600;
		cursor: pointer;
		letter-spacing: -0.01em;
		color: var(--text-on-color);
		background: var(--accent);
		display: flex;
		align-items: center;
		gap: 6px;
		flex-shrink: 0;
		transition: opacity 120ms ease;
	}

	.cmd-action:hover:not(:disabled) {
		opacity: 0.88;
	}
	.cmd-action:disabled {
		opacity: 0.32;
		cursor: not-allowed;
	}

	.spinner {
		display: inline-block;
		width: 12px;
		height: 12px;
		border: 2px solid rgba(255, 255, 255, 0.35);
		border-top-color: var(--text-on-color);
		border-radius: 50%;
		animation: spin 0.6s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.sr-only {
		position: absolute;
		width: 1px;
		height: 1px;
		padding: 0;
		margin: -1px;
		overflow: hidden;
		clip: rect(0, 0, 0, 0);
		white-space: nowrap;
		border: 0;
	}
</style>
