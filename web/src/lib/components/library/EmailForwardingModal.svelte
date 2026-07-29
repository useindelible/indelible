<script lang="ts">
	import { getModalStore } from '$lib/stores/addItemModal.svelte';
	import { getAuth } from '$lib/stores/auth.svelte';

	const modal = getModalStore();
	const auth = getAuth();

	let dialogEl = $state<HTMLDialogElement | undefined>(undefined);
	let feedCopied = $state(false);
	let libraryCopied = $state(false);

	const FEED_EMAIL = $derived(auth.user?.ingest_email ?? '');
	const LIBRARY_EMAIL = $derived(auth.user?.ingest_library_email ?? '');

	const isOpen = $derived(modal.active === 'email');

	$effect(() => {
		if (!dialogEl) return;
		if (isOpen) {
			feedCopied = false;
			libraryCopied = false;
			dialogEl.showModal();
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

	async function copyToClipboard(text: string, which: 'feed' | 'library') {
		if (!text) return;
		await navigator.clipboard.writeText(text);
		if (which === 'feed') {
			feedCopied = true;
			setTimeout(() => {
				feedCopied = false;
			}, 2000);
		} else {
			libraryCopied = true;
			setTimeout(() => {
				libraryCopied = false;
			}, 2000);
		}
	}
</script>

<dialog
	bind:this={dialogEl}
	class="modal-backdrop"
	aria-label="Email Forwarding"
	onclick={handleBackdropClick}
	onclose={close}
>
	<div class="cmd-card" role="document">
		<div class="cmd-body">
			<!-- Feed Email -->
			<div class="email-card">
				<div class="email-card-icon">
					<svg viewBox="0 0 24 24" aria-hidden="true">
						<path d="M4 11a9 9 0 0 1 9 9" />
						<path d="M4 4a16 16 0 0 1 16 16" />
						<circle cx="5" cy="19" r="1" fill="currentColor" stroke="none" />
					</svg>
				</div>
				<div class="email-card-body">
					<div class="email-card-title">Feed Email</div>
					<div class="email-card-desc">New items appear in your Feed for review</div>
				</div>
				<div class="email-card-right">
					<span class="email-card-address">{FEED_EMAIL}</span>
					<button
						type="button"
						class="copy-btn"
						class:copied={feedCopied}
						aria-label="Copy feed email"
						onclick={() => copyToClipboard(FEED_EMAIL, 'feed')}
					>
						{#if feedCopied}
							<svg viewBox="0 0 24 24" aria-hidden="true">
								<polyline points="20 6 9 17 4 12" />
							</svg>
						{:else}
							<svg viewBox="0 0 24 24" aria-hidden="true">
								<rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
								<path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
							</svg>
						{/if}
					</button>
				</div>
			</div>

			<!-- Library Email -->
			<div class="email-card">
				<div class="email-card-icon">
					<svg viewBox="0 0 24 24" aria-hidden="true">
						<path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20" />
						<path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" />
					</svg>
				</div>
				<div class="email-card-body">
					<div class="email-card-title">Library Email</div>
					<div class="email-card-desc">Items are saved directly to your Library</div>
				</div>
				<div class="email-card-right">
					<span class="email-card-address">{LIBRARY_EMAIL}</span>
					<button
						type="button"
						class="copy-btn"
						class:copied={libraryCopied}
						aria-label="Copy library email"
						onclick={() => copyToClipboard(LIBRARY_EMAIL, 'library')}
					>
						{#if libraryCopied}
							<svg viewBox="0 0 24 24" aria-hidden="true">
								<polyline points="20 6 9 17 4 12" />
							</svg>
						{:else}
							<svg viewBox="0 0 24 24" aria-hidden="true">
								<rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
								<path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
							</svg>
						{/if}
					</button>
				</div>
			</div>
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
		width: 440px;
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

	.cmd-body {
		padding: 16px;
	}

	.email-card {
		border-radius: 10px;
		background: var(--bg-secondary);
		padding: 14px;
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.email-card + .email-card {
		margin-top: 8px;
	}

	.email-card-icon {
		width: 32px;
		height: 32px;
		border-radius: 8px;
		flex-shrink: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		background: var(--fill-selected);
	}

	.email-card-icon svg {
		width: 14px;
		height: 14px;
		stroke: var(--accent);
		fill: none;
		stroke-width: 1.5;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.email-card-body {
		flex: 1;
		min-width: 0;
	}

	.email-card-title {
		font-size: 13px;
		font-weight: 600;
		color: var(--text-primary);
		letter-spacing: -0.01em;
		font-family: var(--font-sans);
	}

	.email-card-desc {
		font-size: 11px;
		color: var(--text-secondary);
		font-family: var(--font-sans);
		margin-top: 2px;
	}

	.email-card-right {
		display: flex;
		align-items: center;
		gap: 6px;
		flex-shrink: 0;
	}

	.email-card-address {
		font-family: 'SF Mono', SFMono-Regular, Menlo, monospace;
		font-size: 10.5px;
		font-weight: 500;
		padding: 4px 8px;
		border-radius: 6px;
		white-space: nowrap;
		background: var(--fill-selected);
		color: var(--accent);
	}

	.copy-btn {
		width: 26px;
		height: 26px;
		border-radius: 6px;
		border: none;
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: pointer;
		flex-shrink: 0;
		background: rgba(0, 0, 0, 0.04);
		color: var(--text-secondary);
		transition:
			background 0.1s ease,
			color 0.1s ease;
	}

	[data-theme='dark'] .copy-btn {
		background: rgba(255, 255, 255, 0.08);
	}

	.copy-btn:hover {
		background: rgba(0, 0, 0, 0.08);
		color: var(--text-primary);
	}

	[data-theme='dark'] .copy-btn:hover {
		background: rgba(255, 255, 255, 0.14);
	}

	.copy-btn.copied {
		color: var(--accent);
	}

	.copy-btn svg {
		width: 13px;
		height: 13px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.5;
		stroke-linecap: round;
		stroke-linejoin: round;
	}
</style>
