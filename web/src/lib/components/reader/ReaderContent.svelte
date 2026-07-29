<script lang="ts">
	import { getReaderPreferences } from '$lib/stores/reader-preferences.svelte';
	import type { SearchEmbeddedSenderResponse } from '$lib/api';
	import { hasScrollableOverflow, scrollProgressPercent } from './progress-geometry';
	import { sanitizeReaderHtml } from '$lib/utils/sanitize-html';

	interface Props {
		htmlContent: string;
		title: string;
		author?: string | null;
		domain?: string | null;
		publishedAt?: string | null;
		readingTimeMinutes?: number | null;
		onScroll: (percent: number) => void;
		initialProgress?: number;
		articleBodyEl?: HTMLDivElement;
		scrollEl?: HTMLDivElement;
		sender?: SearchEmbeddedSenderResponse | null;
		onSenderBlockToggle?: (sender: SearchEmbeddedSenderResponse, blocked: boolean) => Promise<void>;
	}

	let {
		htmlContent,
		title,
		author,
		domain,
		publishedAt,
		readingTimeMinutes,
		onScroll,
		initialProgress = 0,
		articleBodyEl = $bindable(),
		scrollEl = $bindable(),
		sender = null,
		onSenderBlockToggle
	}: Props = $props();

	let senderActionPending = $state(false);

	async function toggleSenderBlock() {
		if (!sender || !onSenderBlockToggle || senderActionPending) return;
		senderActionPending = true;
		try {
			await onSenderBlockToggle(sender, !sender.blocked);
		} finally {
			senderActionPending = false;
		}
	}

	const prefs = getReaderPreferences();

	let contentEl = $state<HTMLDivElement | undefined>(undefined);
	let hasRestoredScroll = $state(false);

	function reportProgress(onlyWhenUnscrollable = false) {
		if (!scrollEl) return;
		if (onlyWhenUnscrollable && hasScrollableOverflow(scrollEl)) return;

		const percent = Math.round(scrollProgressPercent(scrollEl));
		onScroll(percent);
	}

	function handleScroll() {
		reportProgress();
	}

	$effect(() => {
		if (initialProgress <= 0) {
			hasRestoredScroll = true;
			return;
		}
		if (scrollEl && initialProgress > 0 && !hasRestoredScroll && htmlContent) {
			requestAnimationFrame(() => {
				if (!scrollEl) return;
				const maxScroll = scrollEl.scrollHeight - scrollEl.clientHeight;
				if (maxScroll > 0) {
					scrollEl.scrollTop = (initialProgress / 100) * maxScroll;
				}
				hasRestoredScroll = true;
			});
		}
	});

	$effect(() => {
		if (!scrollEl || !htmlContent || !hasRestoredScroll) return;

		let frame: ReturnType<typeof requestAnimationFrame> | undefined;
		const queueUnscrollableProgressCheck = () => {
			if (frame) cancelAnimationFrame(frame);
			frame = requestAnimationFrame(() => {
				frame = undefined;
				reportProgress(true);
			});
		};

		queueUnscrollableProgressCheck();

		let observer: ResizeObserver | undefined;
		if (typeof ResizeObserver !== 'undefined') {
			observer = new ResizeObserver(queueUnscrollableProgressCheck);
			observer.observe(scrollEl);
			if (contentEl) observer.observe(contentEl);
		}

		return () => {
			observer?.disconnect();
			if (frame) cancelAnimationFrame(frame);
		};
	});

	const formattedDate = $derived(
		publishedAt
			? new Date(publishedAt).toLocaleDateString('en-US', {
					year: 'numeric',
					month: 'long',
					day: 'numeric'
				})
			: null
	);

	const readingTime = $derived(readingTimeMinutes ? `${readingTimeMinutes} min read` : null);

	const sepiaTheme = $derived(prefs.theme === 'sepia' ? 'sepia' : undefined);

	const sanitizedHtml = $derived(sanitizeReaderHtml(htmlContent));
</script>

<div class="reader-scroll" bind:this={scrollEl} onscroll={handleScroll}>
	<div
		bind:this={contentEl}
		class="reader-content"
		data-reader-theme={sepiaTheme}
		style:--reader-font-family={prefs.fontFamily}
		style:--reader-font-size="{prefs.fontSize}px"
		style:--reader-line-height={prefs.lineHeight}
		style:--reader-content-width="{prefs.contentWidth}px"
		style:--reader-paragraph-spacing="{prefs.paragraphSpacing}em"
		style:--reader-text-align={prefs.textAlign}
	>
		<h1 class="article-title">{title}</h1>

		{#if author || domain || formattedDate || readingTime || sender}
			<div class="article-meta">
				{#if author}
					<span>{author}</span>
				{/if}
				{#if author && (domain || formattedDate || readingTime)}
					<span class="meta-dot"></span>
				{/if}
				{#if domain}
					<span>{domain}</span>
				{/if}
				{#if domain && (formattedDate || readingTime)}
					<span class="meta-dot"></span>
				{/if}
				{#if formattedDate}
					<span>{formattedDate}</span>
				{/if}
				{#if formattedDate && readingTime}
					<span class="meta-dot"></span>
				{/if}
				{#if readingTime}
					<span>{readingTime}</span>
				{/if}
				{#if sender && onSenderBlockToggle}
					<button
						type="button"
						class="sender-chip"
						class:blocked={sender.blocked}
						disabled={senderActionPending}
						onclick={toggleSenderBlock}
						aria-label={sender.blocked
							? `Unblock ${sender.display_name ?? sender.canonical_addr}`
							: `Block ${sender.display_name ?? sender.canonical_addr}`}
					>
						{#if sender.blocked}
							{senderActionPending ? 'Unblocking…' : 'Sender blocked · Unblock'}
						{:else}
							{senderActionPending ? 'Blocking…' : 'Block sender'}
						{/if}
					</button>
				{/if}
			</div>
		{/if}

		<div class="article-body" bind:this={articleBodyEl}>
			<!-- eslint-disable-next-line svelte/no-at-html-tags -- sanitizeReaderHtml (DOMPurify) strips active content client-side -->
			{@html sanitizedHtml}
		</div>
	</div>
</div>

<style>
	.reader-scroll {
		flex: 1;
		overflow-y: auto;
		overflow-x: hidden;
		padding: 32px 24px 64px;
	}

	.reader-scroll::-webkit-scrollbar {
		width: 6px;
	}

	.reader-scroll::-webkit-scrollbar-track {
		background: transparent;
	}

	.reader-scroll::-webkit-scrollbar-thumb {
		background: var(--text-quaternary);
		border-radius: 3px;
	}

	.reader-content {
		max-width: var(--reader-content-width, 760px);
		margin: 0 auto;
		width: 100%;
	}

	.article-title {
		font-size: 28px;
		font-weight: 700;
		letter-spacing: -0.03em;
		line-height: 1.18;
		color: var(--text-primary);
		margin: 0 0 12px;
		font-family: var(--reader-font-family, var(--font-sans));
	}

	.article-meta {
		font-size: 13px;
		font-weight: 400;
		letter-spacing: -0.01em;
		line-height: 1.45;
		color: var(--text-secondary);
		margin-bottom: 32px;
		display: flex;
		align-items: center;
		gap: 6px;
		font-family: var(--font-sans);
	}

	.meta-dot {
		width: 3px;
		height: 3px;
		border-radius: 50%;
		background: var(--text-tertiary);
		flex-shrink: 0;
	}

	.sender-chip {
		margin-left: auto;
		padding: 3px 10px;
		border-radius: 999px;
		border: 1px solid var(--destructive-border);
		background: transparent;
		font-family: var(--font-sans);
		font-size: 11px;
		font-weight: 500;
		letter-spacing: 0;
		color: var(--destructive);
		cursor: pointer;
		white-space: nowrap;
		transition:
			background 120ms ease,
			border-color 120ms ease;
	}

	.sender-chip:hover:not(:disabled) {
		background: var(--destructive-soft);
	}

	.sender-chip.blocked {
		color: var(--text-secondary);
		border-color: var(--border-primary);
	}

	.sender-chip.blocked:hover:not(:disabled) {
		background: var(--fill-hover);
	}

	.sender-chip:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.article-body {
		font-size: var(--reader-font-size, 18px);
		font-weight: 400;
		line-height: var(--reader-line-height, 1.75);
		color: var(--text-primary);
		letter-spacing: -0.01em;
		font-family: var(--reader-font-family, var(--font-sans));
		text-align: var(--reader-text-align, left);
		position: relative;
	}

	.article-body :global(p) {
		margin-bottom: var(--reader-paragraph-spacing, 1.2em);
	}

	.article-body :global(p:last-child) {
		margin-bottom: 0;
	}

	.article-body :global(img) {
		max-width: 100%;
		height: auto;
		border-radius: 8px;
	}

	.article-body :global(a) {
		color: var(--accent);
		text-decoration: none;
	}

	.article-body :global(a:hover) {
		text-decoration: underline;
	}

	.article-body :global(blockquote) {
		border-left: 3px solid var(--border-secondary);
		padding-left: 16px;
		margin: 1.2em 0;
		color: var(--text-secondary);
	}

	.article-body :global(code) {
		font-family: 'SF Mono', 'Fira Code', 'Menlo', monospace;
		font-size: 0.88em;
		background: var(--fill-hover);
		padding: 2px 5px;
		border-radius: 4px;
	}

	.article-body :global(pre) {
		background: var(--bg-secondary);
		padding: 16px;
		border-radius: 8px;
		overflow-x: auto;
		margin: 1.2em 0;
	}

	.article-body :global(pre code) {
		background: none;
		padding: 0;
	}

	/* Articles stored before the renderer was fixed include a title <h1> as the first
	   element. The reader renders the title separately above, so suppress the duplicate. */
	.article-body :global(h1:first-child) {
		display: none;
	}

	.article-body :global(h2),
	.article-body :global(h3),
	.article-body :global(h4) {
		font-family: var(--reader-font-family, var(--font-sans));
		color: var(--text-primary);
		margin-top: 1.5em;
		margin-bottom: 0.5em;
	}

	.article-body :global(.highlight-yellow) {
		background: var(--highlight-yellow-bg);
		padding: 1px 2px;
		border-radius: 3px;
	}

	.article-body :global(.highlight-blue) {
		background: var(--highlight-blue-bg);
		padding: 1px 2px;
		border-radius: 3px;
	}

	.article-body :global(.highlight-green) {
		background: var(--highlight-green-bg);
		padding: 1px 2px;
		border-radius: 3px;
	}

	.article-body :global(.highlight-pink) {
		background: var(--highlight-pink-bg);
		padding: 1px 2px;
		border-radius: 3px;
	}

	.article-body :global(.highlight-purple) {
		background: var(--highlight-purple-bg);
		padding: 1px 2px;
		border-radius: 3px;
	}

	.article-body :global(.tts-active) {
		position: relative;
		background: var(--tts-element-bg);
		padding: 14px 18px 14px 20px;
		margin: 0 -18px 1.2em -20px;
		border-radius: 6px;
		border-left: 3px solid var(--tts-element-border);
		transition:
			background 200ms ease,
			padding 200ms ease;
	}

	.article-body :global(.tts-speaking-pill) {
		position: absolute;
		top: -12px;
		right: 14px;
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 3px 9px 3px 8px;
		background: var(--accent);
		color: var(--text-on-color);
		border-radius: 980px;
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		box-shadow: var(--tts-primary-shadow);
		font-family: var(--font-sans);
		user-select: none;
		pointer-events: none;
	}

	.article-body :global(.tts-speaking-pill .tts-pill-waves) {
		display: inline-flex;
		align-items: center;
		gap: 1.5px;
		height: 10px;
	}

	.article-body :global(.tts-speaking-pill .tts-pill-waves span) {
		width: 2px;
		background: var(--text-on-color);
		border-radius: 1px;
		transform-origin: center;
		animation: ttsPillWave 900ms ease-in-out infinite;
	}

	.article-body :global(.tts-speaking-pill .tts-pill-waves span:nth-child(1)) {
		height: 4px;
		animation-delay: 0ms;
	}

	.article-body :global(.tts-speaking-pill .tts-pill-waves span:nth-child(2)) {
		height: 9px;
		animation-delay: 120ms;
	}

	.article-body :global(.tts-speaking-pill .tts-pill-waves span:nth-child(3)) {
		height: 6px;
		animation-delay: 240ms;
	}

	@keyframes ttsPillWave {
		0%,
		100% {
			transform: scaleY(0.35);
			opacity: 0.6;
		}
		50% {
			transform: scaleY(1);
			opacity: 1;
		}
	}

	/* ---- YouTube video reader styles ---- */

	.article-body :global(.yt-embed) {
		width: 100%;
		aspect-ratio: 16 / 9;
		border-radius: 12px;
		overflow: hidden;
		background: #000;
		margin-bottom: 28px;
		box-shadow:
			0 8px 40px rgba(0, 0, 0, 0.14),
			0 0 0 0.5px rgba(0, 0, 0, 0.06);
	}

	.article-body :global(.yt-embed iframe) {
		width: 100%;
		height: 100%;
		border: none;
	}

	.article-body :global(.yt-channel-header) {
		display: flex;
		align-items: center;
		gap: 10px;
		margin-bottom: 24px;
	}

	.article-body :global(.yt-channel-avatar) {
		width: 36px;
		height: 36px;
		border-radius: 50%;
		background: linear-gradient(135deg, var(--accent) 0%, #5856d6 100%);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 14px;
		font-weight: 600;
		color: #fff;
		flex-shrink: 0;
		font-family: var(--font-sans);
	}

	.article-body :global(.yt-channel-info) {
		display: flex;
		flex-direction: column;
		gap: 1px;
	}

	.article-body :global(.yt-channel-name) {
		font-size: 14px;
		font-weight: 600;
		color: var(--text-primary);
		letter-spacing: -0.01em;
		font-family: var(--font-sans);
	}

	.article-body :global(.yt-video-stats) {
		font-size: 12px;
		color: var(--text-secondary);
		display: flex;
		align-items: center;
		gap: 6px;
		font-family: var(--font-sans);
	}

	.article-body :global(.yt-stat-dot) {
		width: 3px;
		height: 3px;
		border-radius: 50%;
		background: var(--text-tertiary);
	}

	.article-body :global(.yt-description) {
		font-size: 15px;
		line-height: 1.7;
		color: var(--text-primary);
		letter-spacing: -0.01em;
		padding-bottom: 28px;
		border-bottom: 0.5px solid var(--border-primary);
		margin-bottom: 32px;
	}

	.article-body :global(.yt-transcript h2) {
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		color: var(--text-tertiary);
		margin-bottom: 24px;
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.article-body :global(.yt-transcript h2::after) {
		content: '';
		flex: 1;
		height: 0.5px;
		background: var(--border-primary);
	}

	.article-body :global(.transcript-flow) {
		font-size: var(--reader-font-size, 17px);
		line-height: var(--reader-line-height, 1.8);
		color: var(--text-primary);
		letter-spacing: -0.01em;
	}

	.article-body :global(.transcript-flow p) {
		margin-bottom: 1.2em;
	}

	.article-body :global(.transcript-flow p:last-child) {
		margin-bottom: 0;
	}

	.article-body :global(.t-seg) {
		position: relative;
		display: inline;
		border-radius: 4px;
		transition: background 200ms ease;
		cursor: default;
	}

	.article-body :global(.t-seg::before) {
		content: attr(data-t);
		position: absolute;
		left: 0;
		bottom: calc(100% + 4px);
		font-size: 11px;
		font-weight: 500;
		font-variant-numeric: tabular-nums;
		background: var(--accent);
		color: #fff;
		padding: 1px 6px;
		border-radius: 4px;
		opacity: 0;
		transform: translateY(3px);
		transition:
			opacity 150ms ease,
			transform 150ms ease;
		pointer-events: none;
		white-space: nowrap;
		z-index: 10;
		font-family: var(--font-sans);
	}

	.article-body :global(.t-seg:hover) {
		background: var(--fill-selected);
	}

	.article-body :global(.t-seg:hover::before) {
		opacity: 1;
		transform: translateY(0);
	}

	/* Sepia theme overrides */
	.reader-content[data-reader-theme='sepia'] {
		--reader-bg: #f5edda;
		--reader-text: #5b4636;
		--reader-text-secondary: #8b7355;
	}

	.reader-content[data-reader-theme='sepia'] :global(*) {
		color: var(--reader-text);
	}

	.reader-content[data-reader-theme='sepia'] .article-meta,
	.reader-content[data-reader-theme='sepia'] .article-meta :global(*) {
		color: var(--reader-text-secondary);
	}

	.reader-content[data-reader-theme='sepia'] .meta-dot {
		background: var(--reader-text-secondary);
	}

	@media (max-width: 599px) {
		.reader-scroll {
			padding: 24px 18px 56px;
		}

		.article-title {
			font-size: 24px;
		}
	}
</style>
