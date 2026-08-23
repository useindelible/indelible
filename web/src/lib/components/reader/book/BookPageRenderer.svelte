<script lang="ts">
	import type { BookPageContent } from './book-source';
	import { getReaderPreferences } from '$lib/stores/reader-preferences.svelte';
	import { sanitizeReaderHtml } from '$lib/utils/sanitize-html';
	import { computeScale, renderCanvas } from './pdf/pdf-canvas-renderer';
	import { renderPdfTextLayer } from './pdf/pdf-text-layer';
	import { applyThemeRemap, type PdfThemeMode } from './pdf/pdf-dark-mode';
	import './pdf/pdf-text-layer.css';
	import { t } from '$lib/i18n';

	interface Props {
		content: BookPageContent | null;
		contentIndex?: number;
		loading: boolean;
		onScroll?: (charOffset: number, scrollPercent: number) => void;
		restoreCharOffset?: number;
		restoreRequestId?: number;
		chapterBodyEl?: HTMLDivElement;
		pdfPageContainerEl?: HTMLDivElement;
	}

	let {
		content,
		contentIndex = 0,
		loading,
		onScroll,
		restoreCharOffset = 0,
		restoreRequestId = 0,
		chapterBodyEl = $bindable(),
		pdfPageContainerEl = $bindable()
	}: Props = $props();

	const prefs = getReaderPreferences();
	let scrollEl = $state<HTMLDivElement | undefined>(undefined);
	let canvasEl = $state<HTMLCanvasElement | undefined>(undefined);
	let textLayerEl = $state<HTMLDivElement | undefined>(undefined);
	let highlightLayerEl = $state<HTMLDivElement | undefined>(undefined);
	let currentCssScale = $state(1);

	let lastRestoreKey = $state<string | null>(null);

	const sepiaTheme = $derived(prefs.theme === 'sepia' ? 'sepia' : undefined);

	$effect(() => {
		if (!content) {
			lastRestoreKey = null;
			return;
		}
		const restoreKey =
			content.type === 'html'
				? `html:${content.id}:${restoreRequestId}`
				: `pdf:${contentIndex}:${restoreRequestId}`;

		if (restoreKey === lastRestoreKey) return;
		lastRestoreKey = restoreKey;

		if (content.type === 'html' && scrollEl) {
			requestAnimationFrame(() => {
				if (!scrollEl || !chapterBodyEl) return;
				if (restoreCharOffset > 0) {
					const walker = document.createTreeWalker(chapterBodyEl, NodeFilter.SHOW_TEXT);
					let charCount = 0;
					let node = walker.nextNode();
					while (node) {
						const len = node.textContent?.length ?? 0;
						if (charCount + len >= restoreCharOffset) {
							const range = document.createRange();
							range.setStart(node, Math.min(restoreCharOffset - charCount, len));
							range.collapse(true);
							const rect = range.getBoundingClientRect();
							const scrollRect = scrollEl!.getBoundingClientRect();
							scrollEl!.scrollTop += rect.top - scrollRect.top - 100;
							break;
						}
						charCount += len;
						node = walker.nextNode();
					}
				} else {
					scrollEl!.scrollTop = 0;
				}
			});
		} else if (scrollEl) {
			scrollEl.scrollTop = 0;
		}
	});

	const pdfThemeMode: PdfThemeMode = $derived.by(() => {
		if (
			prefs.theme === 'dark' ||
			(prefs.theme === 'auto' &&
				typeof window !== 'undefined' &&
				window.matchMedia('(prefers-color-scheme: dark)').matches)
		) {
			return 'dark';
		}
		if (prefs.theme === 'sepia') return 'sepia';
		return 'light';
	});

	let cancelTextLayer: (() => void) | null = null;
	let renderGeneration = 0;

	$effect(() => {
		const _theme = pdfThemeMode;
		if (content?.type === 'pdf' && canvasEl && textLayerEl && scrollEl) {
			const gen = ++renderGeneration;
			cancelTextLayer?.();
			cancelTextLayer = null;

			renderPdfPageFull(content, _theme, gen);
		}

		return () => {
			cancelTextLayer?.();
			cancelTextLayer = null;
		};
	});

	async function renderPdfPageFull(
		pdfContent: Extract<BookPageContent, { type: 'pdf' }>,
		themeMode: PdfThemeMode,
		gen: number
	) {
		const containerWidth = (scrollEl?.clientWidth ?? 800) - 80;
		const { cssScale, dpr } = computeScale(pdfContent.width, containerWidth);
		currentCssScale = cssScale;

		await renderCanvas(pdfContent.page, canvasEl!, cssScale, dpr);
		if (gen !== renderGeneration) return;

		const viewport = pdfContent.page.getViewport({ scale: cssScale });
		const result = await renderPdfTextLayer(pdfContent.page, textLayerEl!, viewport);
		if (gen !== renderGeneration) return;
		cancelTextLayer = result.cancel;

		applyThemeRemap(canvasEl!, themeMode);

		onScroll?.(0, 0);
	}

	function handleScroll() {
		if (!scrollEl || !chapterBodyEl || !onScroll) return;
		const { scrollTop, scrollHeight, clientHeight } = scrollEl;
		const maxScroll = scrollHeight - clientHeight;
		if (maxScroll <= 0) return;
		const scrollPercent = scrollTop / maxScroll;

		const totalChars = chapterBodyEl.textContent?.length ?? 0;
		const estimatedCharOffset = Math.round(scrollPercent * totalChars);
		onScroll(estimatedCharOffset, scrollPercent);
	}

	// DOMPurify strips active content; the font-family strip keeps the reader's chosen font from
	// being overridden by the EPUB's own inline styles.
	const sanitizedHtml = $derived(
		content?.type === 'html'
			? sanitizeReaderHtml(content.html).replace(/font-family\s*:[^;"']+(;?)/gi, '$1')
			: ''
	);
</script>

{#if loading}
	<div class="book-page-loading">
		<span class="loading-text">{$t('reader_loading_chapter')}</span>
	</div>
{:else if content?.type === 'html'}
	<div class="book-page-scroll" bind:this={scrollEl} onscroll={handleScroll}>
		<div
			class="book-page-content"
			data-reader-theme={sepiaTheme}
			style:--reader-font-family={prefs.fontFamily}
			style:--reader-font-size="{prefs.fontSize}px"
			style:--reader-line-height={prefs.lineHeight}
			style:--reader-content-width="{prefs.contentWidth}px"
			style:--reader-paragraph-spacing="{prefs.paragraphSpacing}em"
			style:--reader-text-align={prefs.textAlign}
		>
			<div class="chapter-label">
				{content.title.match(/^\d/)
					? $t('reader_chapter_fallback', { values: { number: content.title.split(/\s/)[0] } })
					: ''}
			</div>
			<h1 class="chapter-title">{content.title}</h1>

			<div class="book-body" bind:this={chapterBodyEl}>
				<!-- eslint-disable-next-line svelte/no-at-html-tags -- sanitizeReaderHtml (DOMPurify) strips active content client-side -->
				{@html sanitizedHtml}
			</div>
		</div>
	</div>
{:else if content?.type === 'pdf'}
	<div class="pdf-page-scroll" bind:this={scrollEl}>
		<div
			class="pdf-page-container"
			bind:this={pdfPageContainerEl}
			style:--scale-factor={currentCssScale}
			style:--total-scale-factor={currentCssScale}
			style:--scale-round-x="1px"
			style:--scale-round-y="1px"
		>
			<canvas bind:this={canvasEl}></canvas>
			<div bind:this={textLayerEl}></div>
			<div class="pdf-highlight-layer" bind:this={highlightLayerEl}></div>
		</div>
	</div>
{:else}
	<div class="book-page-loading">
		<span class="loading-text">{$t('reader_no_content')}</span>
	</div>
{/if}

<style>
	.book-page-loading {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.loading-text {
		font-size: 14px;
		color: var(--text-secondary);
		font-family: var(--font-sans);
	}

	.book-page-scroll,
	.pdf-page-scroll {
		flex: 1;
		overflow-y: auto;
		overflow-x: hidden;
		padding: 40px 0 80px;
	}

	.book-page-scroll::-webkit-scrollbar,
	.pdf-page-scroll::-webkit-scrollbar {
		width: 6px;
	}

	.book-page-scroll::-webkit-scrollbar-track,
	.pdf-page-scroll::-webkit-scrollbar-track {
		background: transparent;
	}

	.book-page-scroll::-webkit-scrollbar-thumb,
	.pdf-page-scroll::-webkit-scrollbar-thumb {
		background: var(--text-quaternary);
		border-radius: 3px;
	}

	.book-page-content {
		max-width: var(--reader-content-width, 680px);
		margin: 0 auto;
		padding: 0 40px;
		width: 100%;
	}

	.chapter-label {
		font-size: 12px;
		font-weight: 600;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		color: var(--text-tertiary);
		margin-bottom: 8px;
		font-family: var(--font-sans);
	}

	.chapter-label:empty {
		display: none;
	}

	.chapter-title {
		font-size: 32px;
		font-weight: 700;
		letter-spacing: -0.03em;
		line-height: 1.15;
		color: var(--text-primary);
		margin: 0 0 32px;
		font-family: var(--reader-font-family, var(--font-sans));
	}

	.book-body {
		font-size: var(--reader-font-size, 18px);
		font-weight: 400;
		line-height: var(--reader-line-height, 1.75);
		color: var(--text-primary);
		letter-spacing: -0.01em;
		font-family: var(--reader-font-family, var(--font-sans));
		text-align: var(--reader-text-align, left);
		position: relative;
	}

	.book-body :global(p) {
		margin-bottom: var(--reader-paragraph-spacing, 1.2em);
	}

	.book-body :global(p:first-child::first-letter) {
		font-size: 3.2em;
		float: left;
		line-height: 0.8;
		padding-right: 8px;
		padding-top: 4px;
		font-weight: 700;
		color: var(--text-primary);
	}

	.book-body :global(p:last-child) {
		margin-bottom: 0;
	}

	.book-body :global(img) {
		max-width: 100%;
		height: auto;
		border-radius: 8px;
	}

	.book-body :global(a) {
		color: var(--accent);
		text-decoration: none;
	}

	.book-body :global(a:hover) {
		text-decoration: underline;
	}

	.book-body :global(blockquote) {
		border-left: 3px solid var(--border-secondary);
		padding-left: 16px;
		margin: 1.2em 0;
		color: var(--text-secondary);
	}

	.book-body :global(code) {
		font-family: 'SF Mono', 'Fira Code', 'Menlo', monospace;
		font-size: 0.88em;
		background: var(--fill-hover);
		padding: 2px 5px;
		border-radius: 4px;
	}

	.book-body :global(pre) {
		background: var(--bg-secondary);
		padding: 16px;
		border-radius: 8px;
		overflow-x: auto;
		margin: 1.2em 0;
	}

	.book-body :global(pre code) {
		background: none;
		padding: 0;
	}

	.book-body :global(h2),
	.book-body :global(h3),
	.book-body :global(h4) {
		font-family: var(--reader-font-family, var(--font-sans));
		color: var(--text-primary);
		margin-top: 1.5em;
		margin-bottom: 0.5em;
	}

	.book-body :global(.highlight-yellow) {
		background: var(--highlight-yellow-bg);
		padding: 1px 2px;
		border-radius: 3px;
	}

	.book-body :global(.highlight-blue) {
		background: var(--highlight-blue-bg);
		padding: 1px 2px;
		border-radius: 3px;
	}

	.book-body :global(.highlight-green) {
		background: var(--highlight-green-bg);
		padding: 1px 2px;
		border-radius: 3px;
	}

	.book-body :global(.highlight-pink) {
		background: var(--highlight-pink-bg);
		padding: 1px 2px;
		border-radius: 3px;
	}

	.book-body :global(.highlight-purple) {
		background: var(--highlight-purple-bg);
		padding: 1px 2px;
		border-radius: 3px;
	}

	/* Strip any background/color baked into EPUB HTML so themes always win */
	.book-body :global(*) {
		background-color: transparent !important;
		color: inherit;
	}

	/* Sepia theme overrides */
	.book-page-content[data-reader-theme='sepia'] {
		--reader-bg: #f5edda;
		--reader-text: #5b4636;
		--reader-text-secondary: #8b7355;
	}

	.book-page-content[data-reader-theme='sepia'] :global(*) {
		color: var(--reader-text);
	}

	/* PDF */
	.pdf-page-scroll {
		display: flex;
		justify-content: center;
		align-items: flex-start;
	}

	.pdf-page-container {
		position: relative;
	}

	.pdf-page-container canvas {
		display: block;
		border-radius: 2px;
	}

	.pdf-highlight-layer {
		position: absolute;
		inset: 0;
		z-index: 1;
		pointer-events: none;
	}
</style>
