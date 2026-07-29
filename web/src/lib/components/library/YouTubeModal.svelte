<script lang="ts">
	import * as apiSdk from '$lib/api';
	import type { FeedSourceResponse } from '$lib/api';
	import { getModalStore } from '$lib/stores/addItemModal.svelte';

	const modal = getModalStore();

	let dialogEl = $state<HTMLDialogElement | undefined>(undefined);
	let inputEl = $state<HTMLInputElement | undefined>(undefined);
	let channelUrl = $state('');
	let submitting = $state(false);
	let submitError = $state('');
	let suggestions = $state<FeedSourceResponse[]>([]);
	let suggestionsVisible = $state(false);
	let highlightedIndex = $state(-1);

	const isOpen = $derived(modal.active === 'youtube');

	function normalizeInput(value: string): string {
		const trimmed = value.trim();
		if (!trimmed) return '';
		try {
			new URL(trimmed);
			return trimmed;
		} catch {
			// Bare handle: treat as @handle on youtube.com
			const handle = trimmed.replace(/^@/, '');
			return `https://www.youtube.com/@${handle}`;
		}
	}

	function validate(value: string): string {
		const trimmed = value.trim();
		if (!trimmed) return 'Enter a channel or handle';
		const normalized = normalizeInput(trimmed);
		try {
			const parsed = new URL(normalized);
			if (parsed.hostname !== 'youtube.com' && parsed.hostname !== 'www.youtube.com')
				return 'Must be a youtube.com URL';
			const segments = parsed.pathname.split('/').filter(Boolean);
			if (segments.length === 0) return 'Enter a channel URL or handle (e.g. @channelname)';
			const first = segments[0] ?? '';
			if (first.startsWith('@')) {
				if (first.length <= 1) return 'Enter a channel handle (e.g. @channelname)';
			} else if (['channel', 'user', 'c'].includes(first)) {
				if (!segments[1]) return 'Unsupported URL — use a channel, user, or @handle URL';
			} else {
				return 'Unsupported URL — use a channel, user, or @handle URL';
			}
		} catch {
			return 'Enter a valid URL or handle';
		}
		return '';
	}

	function searchQuery(value: string): string {
		const trimmed = value.trim();
		try {
			const parsed = new URL(trimmed);
			const segments = parsed.pathname.split('/').filter(Boolean);
			if (!segments.length) return '';
			return (segments[segments.length - 1] ?? '').replace(/^@/, '');
		} catch {
			// Bare handle input — strip leading @ for the search query
			return trimmed.replace(/^@/, '');
		}
	}

	const validationError = $derived(channelUrl.trim() ? validate(channelUrl) : '');
	const canSubscribe = $derived(!validationError && !submitting);

	$effect(() => {
		if (!dialogEl) return;
		if (isOpen) {
			channelUrl = '';
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
		const q = searchQuery(channelUrl);
		if (q.length < 2) {
			suggestions = [];
			suggestionsVisible = false;
			return;
		}
		let cancelled = false;
		const timer = setTimeout(async () => {
			const { data } = await apiSdk.searchSources({
				query: { query: q, surface: 'youtube', limit: 5 }
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
		channelUrl = item.url;
		suggestions = [];
		suggestionsVisible = false;
	}

	function close() {
		modal.close();
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === dialogEl) close();
	}

	async function handleSubscribe() {
		if (!canSubscribe) return;
		submitting = true;
		submitError = '';
		suggestionsVisible = false;
		try {
			const { error } = await apiSdk.subscribe({
				body: { url: normalizeInput(channelUrl) }
			});
			if (error) {
				const problem = error as { detail?: string; errors?: Array<{ message: string }> } | null;
				submitError = problem?.detail ?? problem?.errors?.[0]?.message ?? 'Failed to subscribe';
			} else {
				modal.notifySubscribed();
				close();
			}
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
	aria-label="Subscribe to YouTube Channel"
	onclick={handleBackdropClick}
	onclose={close}
>
	<div class="cmd-card" role="document">
		<div class="cmd-brand-bar"></div>

		<div class="cmd-input-zone">
			<div class="cmd-input-wrap">
				<svg class="cmd-icon brand" viewBox="0 0 24 24" aria-hidden="true">
					<path
						d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12z"
					/>
				</svg>
				<input
					bind:this={inputEl}
					bind:value={channelUrl}
					class="cmd-input"
					type="text"
					placeholder="@channelname or youtube.com/..."
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
						if (e.key === 'Enter') handleSubscribe();
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
							{#if item.image_url}
								<img
									class="suggestion-thumb"
									src={item.image_url}
									alt=""
									aria-hidden="true"
									referrerpolicy="no-referrer"
								/>
							{/if}
							<span class="suggestion-name">{item.name}</span>
							<span class="suggestion-meta">{item.domain ?? ''}</span>
						</li>
					{/each}
				</ul>
			{/if}
		</div>

		{#if validationError && channelUrl.trim()}
			<div class="cmd-body">
				<p class="error-text" role="alert">{validationError}</p>
			</div>
		{:else if submitError}
			<div class="cmd-body">
				<p class="error-text" role="alert">{submitError}</p>
			</div>
		{/if}

		<div class="cmd-controls">
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
		background: #ff0000;
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
		color: var(--text-on-color);
		background: #ff0000;
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
		gap: 8px;
		padding: 7px 10px;
		border-radius: 6px;
		cursor: pointer;
	}

	.suggestion-item.highlighted,
	.suggestion-item:hover {
		background: var(--fill-hover);
	}

	.suggestion-thumb {
		width: 20px;
		height: 20px;
		border-radius: 50%;
		object-fit: cover;
		flex-shrink: 0;
	}

	.suggestion-name {
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--text-primary);
		font-weight: 500;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		flex: 1;
	}

	.suggestion-meta {
		font-family: var(--font-sans);
		font-size: 12px;
		color: var(--text-tertiary);
		white-space: nowrap;
		flex-shrink: 0;
	}
</style>
