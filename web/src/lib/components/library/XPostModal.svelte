<script lang="ts">
	import * as apiSdk from '$lib/api';
	import type { FeedSourceResponse } from '$lib/api';
	import { t } from '$lib/i18n';
	import { getModalStore } from '$lib/stores/addItemModal.svelte';

	const modal = getModalStore();

	let dialogEl = $state<HTMLDialogElement | undefined>(undefined);
	let inputEl = $state<HTMLInputElement | undefined>(undefined);
	let username = $state('');
	let submitting = $state(false);
	let submitError = $state('');
	let suggestions = $state<FeedSourceResponse[]>([]);
	let suggestionsVisible = $state(false);
	let highlightedIndex = $state(-1);

	const isOpen = $derived(modal.active === 'x');

	function cleanHandle(value: string): string {
		const trimmed = value.trim();
		if (/^https?:\/\//i.test(trimmed)) {
			const xMatch = trimmed.match(/^https?:\/\/(www\.)?(x|twitter)\.com\/([^/?#]+)/i);
			if (!xMatch) return '';
			return xMatch[3] ?? '';
		}
		return trimmed.replace(/^@/, '');
	}

	function validate(value: string): string {
		const trimmed = value.trim();
		if (!trimmed) return $t('library_x_username_required');
		const handle = cleanHandle(trimmed);
		if (!handle) return $t('library_x_only_urls');
		if (/\s/.test(handle)) return $t('library_x_username_no_spaces');
		if (!/^[A-Za-z0-9_]{1,50}$/.test(handle)) return $t('library_x_invalid_username');
		return '';
	}

	const validationError = $derived(username.trim() ? validate(username) : '');
	const canFollow = $derived(!validationError && !submitting);

	$effect(() => {
		if (!dialogEl) return;
		if (isOpen) {
			username = '';
			submitting = false;
			submitError = '';
			suggestions = [];
			suggestionsVisible = false;
			highlightedIndex = -1;
			dialogEl.showModal();
			setTimeout(() => inputEl?.focus(), 50);
		} else {
			dialogEl.close();
		}
	});

	$effect(() => {
		const handle = cleanHandle(username);
		if (handle.length < 2) {
			suggestions = [];
			suggestionsVisible = false;
			return;
		}
		let cancelled = false;
		const timer = setTimeout(async () => {
			const { data } = await apiSdk.searchSources({
				query: { query: handle, surface: 'twitter', limit: 5 }
			});
			if (cancelled) return;
			suggestions = data?.items ?? [];
			suggestionsVisible = suggestions.length > 0;
			highlightedIndex = -1;
		}, 300);
		return () => {
			cancelled = true;
			clearTimeout(timer);
		};
	});

	function selectSuggestion(item: FeedSourceResponse) {
		const xMatch = item.url.match(/^https?:\/\/(www\.)?(x|twitter)\.com\/([^/?#]+)/i);
		username = xMatch?.[3] ?? item.url;
		suggestions = [];
		suggestionsVisible = false;
	}

	function close() {
		modal.close();
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === dialogEl) close();
	}

	async function handleFollow() {
		if (!canFollow) return;
		submitting = true;
		submitError = '';
		suggestionsVisible = false;
		try {
			const { error } = await apiSdk.subscribe({
				body: { url: `https://x.com/${cleanHandle(username)}` }
			});
			if (error) {
				const problem = error as { detail?: string; errors?: Array<{ message: string }> } | null;
				submitError =
					problem?.detail ?? problem?.errors?.[0]?.message ?? $t('library_x_error_follow');
			} else {
				modal.notifySubscribed();
				close();
			}
		} catch (err) {
			submitError = err instanceof Error ? err.message : $t('library_error_unexpected');
		} finally {
			submitting = false;
		}
	}
</script>

<dialog
	bind:this={dialogEl}
	class="modal-backdrop"
	aria-label={$t('library_x_follow_on')}
	onclick={handleBackdropClick}
	onclose={close}
>
	<div class="cmd-card" role="document">
		<div class="cmd-brand-bar"></div>

		<div class="cmd-input-zone">
			<div class="cmd-input-wrap">
				<svg class="cmd-icon brand" viewBox="0 0 24 24" aria-hidden="true">
					<path
						d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z"
					/>
				</svg>
				<input
					bind:this={inputEl}
					bind:value={username}
					class="cmd-input"
					type="text"
					placeholder={$t('library_x_username')}
					autocomplete="off"
					onkeydown={(e) => {
						if (suggestionsVisible) {
							if (e.key === 'ArrowDown') {
								e.preventDefault();
								highlightedIndex = Math.min(highlightedIndex + 1, suggestions.length - 1);
								return;
							}
							if (e.key === 'ArrowUp') {
								e.preventDefault();
								highlightedIndex = Math.max(highlightedIndex - 1, -1);
								return;
							}
							if (e.key === 'Enter' && highlightedIndex >= 0) {
								e.preventDefault();
								const selected = suggestions[highlightedIndex];
								if (selected) selectSuggestion(selected);
								return;
							}
							if (e.key === 'Escape') {
								suggestionsVisible = false;
								return;
							}
						}
						if (e.key === 'Enter') handleFollow();
						else if (e.key === 'Escape') close();
					}}
				/>
			</div>
			{#if suggestionsVisible}
				<ul class="suggestions" role="listbox">
					{#each suggestions as item, i (item.id)}
						<li
							class="suggestion-item"
							class:highlighted={i === highlightedIndex}
							role="option"
							aria-selected={i === highlightedIndex}
							onmousedown={() => selectSuggestion(item)}
						>
							<span class="suggestion-name">{item.name}</span>
							<span class="suggestion-meta">{item.url.replace('https://', '')}</span>
						</li>
					{/each}
				</ul>
			{/if}
		</div>

		{#if validationError && username.trim()}
			<div class="cmd-body">
				<p class="error-text" role="alert">{validationError}</p>
			</div>
		{:else if submitError}
			<div class="cmd-body">
				<p class="error-text" role="alert">{submitError}</p>
			</div>
		{/if}

		<div class="cmd-controls">
			<button type="button" class="cmd-action" disabled={!canFollow} onclick={handleFollow}>
				{#if submitting}
					<span class="spinner" aria-hidden="true"></span>
					<span class="sr-only">{$t('library_x_following')}</span>
				{:else}
					{$t('library_x_follow')}
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

	.cmd-brand-bar {
		height: 3px;
		background: var(--text-primary);
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
		pointer-events: none;
	}

	.cmd-icon.brand {
		fill: var(--text-tertiary);
		stroke: none;
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
		padding: 0 16px 4px;
	}

	.error-text {
		font-size: 12px;
		color: var(--destructive);
		font-family: var(--font-sans);
		margin: 8px 0 0;
	}

	.cmd-controls {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		padding: 10px 16px 14px;
	}

	.cmd-action {
		padding: 6px 16px;
		border-radius: 980px;
		border: none;
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 600;
		cursor: pointer;
		letter-spacing: -0.01em;
		color: var(--bg-primary);
		background: var(--text-primary);
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
		border-top-color: var(--bg-primary);
		border-radius: 50%;
		animation: spin 0.6s linear infinite;
	}

	[data-theme='dark'] .spinner {
		border: 2px solid rgba(0, 0, 0, 0.35);
		border-top-color: var(--text-primary);
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

	.suggestions {
		list-style: none;
		margin: 4px 0 0;
		padding: 4px;
		background: var(--bg-secondary);
		border-radius: 8px;
		overflow: hidden;
	}

	.suggestion-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 7px 10px;
		border-radius: 6px;
		cursor: pointer;
		gap: 8px;
	}

	.suggestion-item.highlighted,
	.suggestion-item:hover {
		background: var(--fill-hover);
	}

	.suggestion-name {
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--text-primary);
		font-weight: 500;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.suggestion-meta {
		font-family: var(--font-sans);
		font-size: 12px;
		color: var(--text-tertiary);
		white-space: nowrap;
		flex-shrink: 0;
	}
</style>
