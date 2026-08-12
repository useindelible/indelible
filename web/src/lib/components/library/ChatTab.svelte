<script lang="ts">
	import { resolve } from '$app/paths';
	import { onMount } from 'svelte';
	import { createMilaChat, getMilaConfig, type ChatScope } from '$lib/stores/mila.svelte';
	import { renderMilaMessageMarkdown } from '$lib/utils/mila-citations';

	interface Props {
		scope: ChatScope;
		label: string;
	}

	let { scope, label }: Props = $props();

	const config = getMilaConfig();
	const scopeType = $derived(scope.type);
	const scopeId = $derived(scope.type === 'collection' ? scope.collectionId : scope.documentId);

	let chat = $state(createMilaChat({ type: 'single_document', documentId: '' }));

	let messagesEl = $state<HTMLDivElement | undefined>(undefined);
	let inputValue = $state('');

	onMount(() => {
		config.load();
	});

	$effect(() => {
		const currentScope: ChatScope =
			scopeType === 'collection'
				? { type: 'collection', collectionId: scopeId }
				: { type: 'single_document', documentId: scopeId };
		if (!config.configured) return;
		const c = createMilaChat(currentScope);
		chat = c;
		c.initialize();
		return () => c.destroy();
	});

	$effect(() => {
		if (messagesEl && chat.messages.length > 0) {
			messagesEl.scrollTop = messagesEl.scrollHeight;
		}
	});

	function handleSend() {
		const q = inputValue.trim();
		if (!q || chat.streaming) return;
		inputValue = '';
		chat.sendMessage(q);
	}

	function readerHref(itemId: string) {
		return resolve('/(app)/reader/[documentId]', { documentId: itemId });
	}

	const placeholder = $derived(
		scope.type === 'collection'
			? 'Ask Mila about this collection…'
			: 'Ask Mila about this article...'
	);
	const responsePhaseLabel = $derived(
		chat.phase === 'generating'
			? 'Generating response'
			: chat.elapsedSeconds >= 30
				? 'Still preparing — the provider may be starting'
				: 'Preparing response'
	);
</script>

<div class="chat-panel-inner">
	<!-- Scope chip -->
	<div class="chat-scope-bar">
		<div class="chat-scope-chip">
			<svg viewBox="0 0 24 24" aria-hidden="true">
				<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
				<polyline points="14 2 14 8 20 8" />
				<line x1="16" y1="13" x2="8" y2="13" />
			</svg>
			{label.length > 30 ? label.slice(0, 30) + '…' : label}
		</div>
	</div>

	<!-- Messages -->
	<div class="chat-messages" bind:this={messagesEl}>
		{#if !config.loaded || config.loading}
			<div class="chat-state-center">
				<div class="chat-spinner" aria-label="Loading…"></div>
			</div>
		{:else if !config.configured}
			<div class="chat-state-center">
				<p class="chat-empty-hint">
					Mila is not configured. <a href={resolve('/preferences/ai')}>Set up Mila</a> to start chatting.
				</p>
			</div>
		{:else if chat.loading}
			<div class="chat-state-center">
				<div class="chat-spinner" aria-label="Loading session…"></div>
			</div>
		{:else if scope.type === 'collection' && chat.messages.length === 0}
			<div class="chat-state-center">
				<p class="chat-empty-hint">Ask Mila about <strong>{label}</strong>…</p>
			</div>
		{:else}
			{#each chat.messages.filter((m) => m.role === 'user') as userMsg (userMsg.id)}
				{@const assistantMsg = chat.messages[chat.messages.indexOf(userMsg) + 1]}
				<div class="chat-exchange">
					<p class="chat-query">{userMsg.content}</p>
					{#if assistantMsg}
						<div class="chat-response">
							<div class="chat-response-header">
								<span class="chat-response-dot" aria-hidden="true"></span>
								Mila
							</div>
							{#if assistantMsg.streaming}
								<div class="chat-response-progress">
									<span role="status" aria-live="polite">
										{responsePhaseLabel}
										<span aria-hidden="true"> · {chat.elapsedSeconds}s</span>
									</span>
									<button type="button" onclick={() => chat.cancel()} aria-label="Cancel response"
										>Cancel</button
									>
								</div>
							{/if}
							<div class="chat-response-text">
								<!-- eslint-disable-next-line svelte/no-at-html-tags -- markdown sanitized by DOMPurify -->
								{@html renderMilaMessageMarkdown(
									assistantMsg.content,
									assistantMsg.source_refs,
									readerHref
								)}{#if assistantMsg.streaming}<span class="chat-cursor" aria-hidden="true"
									></span>{/if}
							</div>
							{#if !assistantMsg.streaming && assistantMsg.source_refs.length > 0}
								<div class="chat-sources">
									{#each assistantMsg.source_refs as ref (`${ref.source_label}-${ref.document_id}`)}
										<a
											class="chat-source-chip"
											href={resolve('/(app)/reader/[documentId]', { documentId: ref.document_id })}
											title={ref.item_title}
										>
											{ref.item_title.length > 24
												? ref.item_title.slice(0, 24) + '…'
												: ref.item_title}
										</a>
									{/each}
								</div>
							{/if}
						</div>
					{/if}
				</div>
			{/each}
			{#if chat.error}
				<div class="chat-error-row">
					<span class="chat-error-text">{chat.error}</span>
					<button type="button" class="chat-retry-btn" onclick={() => chat.retry()}>Retry</button>
				</div>
			{/if}
		{/if}
	</div>
	{#if chat.retrievalWarning}
		<div class="chat-warning" role="status">{chat.retrievalWarning}</div>
	{/if}

	<!-- Input -->
	<div class="chat-input-row">
		<input
			class="chat-input"
			type="text"
			{placeholder}
			bind:value={inputValue}
			disabled={!config.configured || chat.loading}
			onkeydown={(e) => {
				if (e.key === 'Enter') handleSend();
			}}
		/>
		<button
			type="button"
			class="chat-send"
			aria-label="Send message"
			disabled={!inputValue.trim() || chat.streaming || !config.configured || chat.loading}
			onclick={handleSend}
		>
			<svg viewBox="0 0 24 24" aria-hidden="true">
				<line x1="22" y1="2" x2="11" y2="13" />
				<polygon points="22 2 15 22 11 13 2 9 22 2" />
			</svg>
		</button>
	</div>
</div>

<style>
	.chat-panel-inner {
		flex: 1;
		display: flex;
		flex-direction: column;
		overflow: hidden;
		min-height: 0;
	}

	.chat-scope-bar {
		padding: 10px 16px;
		border-bottom: 0.5px solid var(--border-primary);
		display: flex;
		align-items: center;
		gap: 6px;
		flex-shrink: 0;
	}

	.chat-scope-chip {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		font-size: 11px;
		font-weight: 500;
		letter-spacing: -0.005em;
		color: var(--accent);
		background: var(--fill-selected);
		border-radius: 980px;
		padding: 3px 9px;
		font-family: var(--font-sans);
	}

	.chat-scope-chip svg {
		width: 11px;
		height: 11px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.5;
		stroke-linecap: round;
		stroke-linejoin: round;
		flex-shrink: 0;
	}

	.chat-messages {
		flex: 1;
		overflow-y: auto;
		overflow-x: hidden;
		padding: 16px;
		display: flex;
		flex-direction: column;
		gap: 0;
		min-height: 0;
	}

	.chat-state-center {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 32px 0;
	}

	.chat-spinner {
		width: 20px;
		height: 20px;
		border: 2px solid var(--border-secondary);
		border-top-color: var(--accent);
		border-radius: 50%;
		animation: spin 0.7s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.chat-empty-hint {
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--text-tertiary);
		text-align: center;
		margin: 0;
		line-height: 1.5;
	}

	.chat-empty-hint a {
		color: var(--accent);
		text-decoration: none;
	}

	.chat-empty-hint a:hover {
		text-decoration: underline;
	}

	.chat-exchange {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 14px 0;
		border-bottom: 0.5px solid var(--border-primary);
	}

	.chat-exchange:first-child {
		padding-top: 0;
	}
	.chat-exchange:last-child {
		border-bottom: none;
	}

	.chat-query {
		font-size: 13px;
		font-weight: 500;
		letter-spacing: -0.01em;
		line-height: 1.45;
		color: var(--text-primary);
		font-family: var(--font-sans);
		margin: 0;
		word-break: break-word;
	}

	.chat-response {
		display: flex;
		flex-direction: column;
		gap: 5px;
	}

	.chat-response-header {
		display: flex;
		align-items: center;
		gap: 5px;
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--accent);
		font-family: var(--font-sans);
	}

	.chat-response-dot {
		width: 5px;
		height: 5px;
		border-radius: 50%;
		background: var(--accent);
		flex-shrink: 0;
	}

	.chat-response-text {
		font-size: 13px;
		font-weight: 400;
		letter-spacing: -0.01em;
		line-height: 1.6;
		color: var(--text-secondary);
		font-family: var(--font-sans);
		word-break: break-word;
	}

	.chat-response-progress {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		font-size: 11px;
		color: var(--text-tertiary);
	}

	.chat-response-progress button {
		border: 0;
		background: none;
		color: var(--accent);
		font: inherit;
		cursor: pointer;
		padding: 2px 0;
	}

	.chat-response-text :global(p) {
		margin: 0 0 8px 0;
	}
	.chat-response-text :global(p:last-child) {
		margin-bottom: 0;
	}
	.chat-response-text :global(ul),
	.chat-response-text :global(ol) {
		margin: 4px 0 8px 0;
		padding-left: 18px;
	}
	.chat-response-text :global(li) {
		margin-bottom: 3px;
	}
	.chat-response-text :global(strong) {
		font-weight: 600;
		color: var(--text-primary);
	}
	.chat-response-text :global(code) {
		font-family: ui-monospace, 'SF Mono', Menlo, monospace;
		font-size: 11.5px;
		background: var(--bg-tertiary);
		border: 0.5px solid var(--border-secondary);
		border-radius: 3px;
		padding: 1px 4px;
	}
	.chat-response-text :global(pre) {
		background: var(--bg-tertiary);
		border: 0.5px solid var(--border-secondary);
		border-radius: 6px;
		padding: 10px 12px;
		overflow-x: auto;
		margin: 6px 0;
	}
	.chat-response-text :global(pre code) {
		background: none;
		border: none;
		padding: 0;
		font-size: 11.5px;
	}

	.chat-response-text :global(.chat-inline-source) {
		display: inline-flex;
		align-items: center;
		max-width: 180px;
		vertical-align: baseline;
		font-size: 10px;
		line-height: 1.2;
		font-weight: 600;
		color: var(--accent);
		background: var(--fill-selected);
		border: 0.5px solid var(--border-secondary);
		border-radius: 4px;
		padding: 1px 5px;
		text-decoration: none;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.chat-response-text :global(.chat-inline-source:hover) {
		background: var(--fill-hover);
	}

	.chat-cursor {
		display: inline-block;
		width: 2px;
		height: 13px;
		background: var(--accent);
		margin-left: 2px;
		vertical-align: text-bottom;
		animation: blink 1s step-end infinite;
	}

	@keyframes blink {
		50% {
			opacity: 0;
		}
	}

	.chat-sources {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
		margin-top: 4px;
	}

	.chat-source-chip {
		display: inline-flex;
		align-items: center;
		font-size: 10px;
		font-family: var(--font-sans);
		font-weight: 500;
		color: var(--text-secondary);
		background: var(--bg-tertiary);
		border: 0.5px solid var(--border-secondary);
		border-radius: 4px;
		padding: 2px 7px;
		text-decoration: none;
		transition: background 0.1s ease;
	}

	.chat-source-chip:hover {
		background: var(--fill-hover);
	}

	.chat-error-row {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 0;
	}

	.chat-error-text {
		font-size: 12px;
		font-family: var(--font-sans);
		color: var(--destructive);
		flex: 1;
	}

	.chat-retry-btn {
		font-size: 12px;
		font-family: var(--font-sans);
		font-weight: 500;
		color: var(--accent);
		background: none;
		border: none;
		cursor: pointer;
		padding: 2px 0;
		text-decoration: underline;
	}

	.chat-warning {
		margin: 0 16px 10px;
		padding: 8px 10px;
		border: 0.5px solid var(--warning);
		border-radius: 6px;
		background: var(--fill-warning);
		color: var(--text-secondary);
		font-size: 12px;
		line-height: 1.4;
		flex-shrink: 0;
	}

	.chat-input-row {
		padding: 12px 16px;
		border-top: 0.5px solid var(--border-primary);
		display: flex;
		align-items: center;
		gap: 8px;
		flex-shrink: 0;
	}

	.chat-input {
		flex: 1;
		background: transparent;
		border: none;
		border-bottom: 1px solid var(--border-secondary);
		padding: 5px 0;
		font-size: 13px;
		font-weight: 400;
		letter-spacing: -0.01em;
		color: var(--text-primary);
		font-family: var(--font-sans);
		outline: none;
	}

	.chat-input::placeholder {
		color: var(--text-tertiary);
	}

	.chat-input:disabled {
		opacity: 0.5;
	}

	.chat-send {
		width: 28px;
		height: 28px;
		border-radius: 50%;
		background: var(--accent);
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: pointer;
		flex-shrink: 0;
		border: none;
		transition: opacity 120ms ease;
	}

	.chat-send:disabled {
		opacity: 0.35;
		cursor: not-allowed;
	}

	.chat-send svg {
		width: 12px;
		height: 12px;
		stroke: var(--text-on-color);
		fill: none;
		stroke-width: 2;
		stroke-linecap: round;
		stroke-linejoin: round;
	}
</style>
