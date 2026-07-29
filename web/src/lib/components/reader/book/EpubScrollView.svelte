<script lang="ts">
	import { SvelteSet, SvelteMap } from 'svelte/reactivity';
	import type { BookSource, BookPageContent, TocEntry } from './book-source';
	import type { HighlightWithNoteResponse } from '$lib/api/generated/types.gen';
	import { getReaderPreferences } from '$lib/stores/reader-preferences.svelte';
	import { sanitizeReaderHtml } from '$lib/utils/sanitize-html';
	import { applyHighlights, type HighlightRange } from '../highlight-utils';
	import { scrollProgressPercent } from '../progress-geometry';

	interface Props {
		source: BookSource;
		highlights: HighlightWithNoteResponse[];
		initialChapterIndex?: number;
		initialCharOffset?: number;
		onChapterChange?: (index: number, chapterId: string) => void;
		onActiveEntryChange?: (entryId: string) => void;
		onProgress?: (progress: number, chapterIndex: number, charOffset: number) => void;
		scrollContainerEl?: HTMLDivElement;
	}

	let {
		source,
		highlights,
		initialChapterIndex = 0,
		initialCharOffset = 0,
		onChapterChange,
		onActiveEntryChange,
		onProgress,
		scrollContainerEl = $bindable()
	}: Props = $props();

	const prefs = getReaderPreferences();
	const sepiaTheme = $derived(prefs.theme === 'sepia' ? 'sepia' : undefined);

	// Deduplicate spine indices to get the list of chapters to render
	const chapterEntries: TocEntry[] = (() => {
		const seen = new SvelteSet<number>();
		const entries: TocEntry[] = [];
		for (const entry of source.toc) {
			if (!seen.has(entry.index)) {
				seen.add(entry.index);
				entries.push(entry);
			}
		}
		return entries.sort((a, b) => a.index - b.index);
	})();

	let containerEl = $state<HTMLDivElement | undefined>(undefined);
	let currentChapterIndex = $state(initialChapterIndex);
	let hasScrolledToInitial = $state(initialChapterIndex <= 0 && initialCharOffset <= 0);
	let loadedChapterVersion = $state(0);
	let pendingFragment = $state<string | undefined>(undefined);
	let currentActiveEntryId = $state<string>('');

	$effect(() => {
		scrollContainerEl = containerEl;
	});

	const loadedChapters = new SvelteMap<number, BookPageContent>();
	const chapterBodyEls = new SvelteMap<number, HTMLDivElement>();
	const loadingChapters = new SvelteSet<number>();
	let observer: IntersectionObserver | null = null;

	$effect(() => {
		if (!containerEl || !hasScrolledToInitial) return;

		observer = new IntersectionObserver(handleIntersection, {
			root: containerEl,
			rootMargin: '400px 0px'
		});

		const wrappers = containerEl.querySelectorAll<HTMLElement>('[data-chapter-index]');
		wrappers.forEach((el) => observer!.observe(el));

		return () => {
			observer?.disconnect();
			observer = null;
			loadedChapters.clear();
			chapterBodyEls.clear();
		};
	});

	// Scroll to initial chapter after mount
	$effect(() => {
		if (!containerEl || hasScrolledToInitial) return;
		if (initialChapterIndex <= 0 && initialCharOffset <= 0) {
			hasScrolledToInitial = true;
			return;
		}
		void loadedChapterVersion;

		const target = containerEl.querySelector<HTMLElement>(
			`[data-chapter-index="${initialChapterIndex}"]`
		);
		if (!target) return;

		if (!loadedChapters.has(initialChapterIndex)) {
			void loadChapter(initialChapterIndex, target);
			return;
		}

		requestAnimationFrame(() => {
			if (!containerEl) return;

			if (initialCharOffset > 0) {
				const bodyEl = chapterBodyEls.get(initialChapterIndex);
				if (bodyEl) {
					const walker = document.createTreeWalker(bodyEl, NodeFilter.SHOW_TEXT);
					let charCount = 0;
					let node = walker.nextNode();
					while (node) {
						const len = node.textContent?.length ?? 0;
						if (charCount + len >= initialCharOffset) {
							const range = document.createRange();
							range.setStart(node, Math.min(initialCharOffset - charCount, len));
							range.collapse(true);
							const rect = range.getBoundingClientRect();
							const scrollRect = containerEl!.getBoundingClientRect();
							containerEl!.scrollTop += rect.top - scrollRect.top - 100;
							hasScrolledToInitial = true;
							return;
						}
						charCount += len;
						node = walker.nextNode();
					}
				}
			}

			containerEl.scrollTop = Math.max(0, target.offsetTop - 8);
			hasScrolledToInitial = true;
		});
	});

	// Re-apply highlights when highlights array changes
	$effect(() => {
		void highlights;
		applyAllHighlights();
	});

	function handleIntersection(entries: IntersectionObserverEntry[]) {
		for (const entry of entries) {
			const el = entry.target as HTMLElement;
			const index = parseInt(el.dataset.chapterIndex ?? '-1', 10);
			if (index < 0) continue;

			if (entry.isIntersecting && !loadedChapters.has(index)) {
				loadChapter(index, el);
			}
		}
		if (hasScrolledToInitial) updateCurrentChapter();
	}

	function updateCurrentChapter() {
		if (!containerEl) return;
		const containerRect = containerEl.getBoundingClientRect();
		const topEdge = containerRect.top + 100;

		const wrappers = containerEl.querySelectorAll<HTMLElement>('[data-chapter-index]');
		let active = chapterEntries[0]?.index ?? 0;

		for (const wrapper of wrappers) {
			const rect = wrapper.getBoundingClientRect();
			if (rect.top <= topEdge) {
				active = parseInt(wrapper.dataset.chapterIndex ?? '0', 10);
			} else {
				break;
			}
		}

		if (active !== currentChapterIndex) {
			currentChapterIndex = active;
			const entry = chapterEntries.find((e) => e.index === active);
			onChapterChange?.(active, entry?.id ?? '');
		}

		// Determine which specific TOC entry is active by checking fragment anchors
		const bodyEl = chapterBodyEls.get(active);
		if (bodyEl) {
			const entriesForChapter = source.toc.filter((e) => e.index === active);
			let bestId = entriesForChapter[0]?.id ?? '';

			for (const tocEntry of entriesForChapter) {
				if (!tocEntry.fragment) continue;
				const anchor =
					bodyEl.querySelector<HTMLElement>(`#${CSS.escape(tocEntry.fragment)}`) ??
					bodyEl.querySelector<HTMLElement>(`[name="${CSS.escape(tocEntry.fragment)}"]`);
				if (anchor) {
					const anchorRect = anchor.getBoundingClientRect();
					if (anchorRect.top <= topEdge) {
						bestId = tocEntry.id;
					}
				}
			}

			if (bestId !== currentActiveEntryId) {
				currentActiveEntryId = bestId;
				onActiveEntryChange?.(bestId);
			}
		}

		onProgress?.(
			scrollProgressPercent(containerEl),
			active,
			estimateFirstVisibleCharOffset(active, topEdge)
		);
	}

	function estimateFirstVisibleCharOffset(chapterIndex: number, topEdge: number): number {
		const bodyEl = chapterBodyEls.get(chapterIndex);
		if (!bodyEl) return 0;

		const walker = document.createTreeWalker(bodyEl, NodeFilter.SHOW_TEXT);
		let charCount = 0;
		let node = walker.nextNode();

		while (node) {
			const textLength = node.textContent?.length ?? 0;
			if (textLength === 0) {
				node = walker.nextNode();
				continue;
			}

			const range = document.createRange();
			range.selectNodeContents(node);
			const rects = Array.from(range.getClientRects());
			range.detach();

			const visibleRect = rects.find((rect) => rect.bottom >= topEdge);
			if (visibleRect) {
				if (visibleRect.top >= topEdge || visibleRect.height <= 0) return charCount;
				const visibleFraction = Math.min(
					1,
					Math.max(0, (topEdge - visibleRect.top) / visibleRect.height)
				);
				return charCount + Math.floor(textLength * visibleFraction);
			}

			charCount += textLength;
			node = walker.nextNode();
		}

		return bodyEl.textContent?.length ?? 0;
	}

	async function loadChapter(index: number, wrapperEl: HTMLElement) {
		if (loadedChapters.has(index) || loadingChapters.has(index)) return;
		loadingChapters.add(index);

		try {
			const content = await source.loadPage(index);
			if (content.type !== 'html') return;
			loadedChapters.set(index, content);
			loadedChapterVersion += 1;

			const sanitized = sanitizeReaderHtml(content.html).replace(
				/font-family\s*:[^;"']+(;?)/gi,
				'$1'
			);

			const titleEl = wrapperEl.querySelector('.chapter-title') as HTMLElement;
			const labelEl = wrapperEl.querySelector('.chapter-label') as HTMLElement;
			const bodyEl = wrapperEl.querySelector('.book-body') as HTMLDivElement;
			const placeholder = wrapperEl.querySelector('.chapter-placeholder') as HTMLElement;

			if (titleEl) titleEl.textContent = content.title;
			if (labelEl) {
				labelEl.textContent = content.title.match(/^\d/)
					? `Chapter ${content.title.split(/\s/)[0]}`
					: '';
			}
			if (bodyEl) {
				bodyEl.innerHTML = sanitized;
				chapterBodyEls.set(index, bodyEl);
				applyHighlightsForChapter(index, bodyEl);

				if (pendingFragment) {
					const frag = pendingFragment;
					pendingFragment = undefined;
					requestAnimationFrame(() => {
						const anchor =
							bodyEl.querySelector<HTMLElement>(`#${CSS.escape(frag)}`) ??
							bodyEl.querySelector<HTMLElement>(`[name="${CSS.escape(frag)}"]`);
						if (anchor) {
							anchor.scrollIntoView({ behavior: 'smooth', block: 'start' });
						}
					});
				}
			}
			if (placeholder) placeholder.remove();
		} catch {
			// Chapter load failed
		} finally {
			loadingChapters.delete(index);
		}
	}

	function applyAllHighlights() {
		for (const [index, bodyEl] of chapterBodyEls) {
			applyHighlightsForChapter(index, bodyEl);
		}
	}

	function applyHighlightsForChapter(index: number, bodyEl: HTMLDivElement) {
		const entry = chapterEntries.find((e) => e.index === index);
		if (!entry) return;

		const ranges: HighlightRange[] = highlights.flatMap((h) => {
			const loc = h.locator;
			if (h.color === 'bookmark' || loc?.type !== 'epub' || loc.chapter !== entry.id) {
				return [];
			}
			return [
				{
					id: h.id,
					color: h.color,
					startOffset: 'start_offset' in loc ? (loc.start_offset ?? 0) : 0,
					endOffset: 'end_offset' in loc ? (loc.end_offset ?? 0) : 0
				}
			];
		});

		applyHighlights(bodyEl, ranges);
	}

	function handleScroll() {
		if (!hasScrolledToInitial) return;
		updateCurrentChapter();
	}

	function estimateChapterHeight(entry: TocEntry): number {
		const words = entry.wordCount ?? 200;
		const linesPerWord = 1 / 8;
		const lineHeight = (prefs.fontSize ?? 18) * (prefs.lineHeight ?? 1.75);
		return Math.max(200, words * linesPerWord * lineHeight + 120);
	}

	function handleContentClick(e: MouseEvent) {
		const anchor = (e.target as HTMLElement).closest('a');
		if (!anchor) return;

		const rawHref = anchor.getAttribute('href');
		if (!rawHref) return;

		// External links: open in new tab, don't hijack
		if (/^https?:\/\/|^mailto:/i.test(rawHref)) {
			e.preventDefault();
			window.open(rawHref, '_blank', 'noopener,noreferrer');
			return;
		}

		// All intra-EPUB links: prevent browser navigation
		e.preventDefault();

		if (rawHref.startsWith('#')) {
			scrollToEpubFragment(rawHref.slice(1));
			return;
		}

		// Cross-chapter link: e.g. "notes.xhtml#fn4-ch8" or "../Text/notes.xhtml"
		const hashIdx = rawHref.indexOf('#');
		const pathPart = hashIdx >= 0 ? rawHref.slice(0, hashIdx) : rawHref;
		const fragment = hashIdx >= 0 ? rawHref.slice(hashIdx + 1) : undefined;
		const basename = pathPart.split('/').pop() ?? pathPart;

		// Find the spine entry whose spineHref basename matches the link filename
		const targetEntry = chapterEntries.find((entry) => {
			if (!entry.spineHref) return false;
			const entryBasename = entry.spineHref.split('/').pop() ?? entry.spineHref;
			return entryBasename === basename;
		});

		if (targetEntry) {
			scrollToChapter(targetEntry.index, 0, fragment);
			return;
		}

		// Fallback: if we have a fragment, scan all loaded chapter bodies for the anchor
		if (fragment) {
			scrollToEpubFragment(fragment);
		}
	}

	function scrollToEpubFragment(fragment: string) {
		for (const [, bodyEl] of chapterBodyEls) {
			const anchor =
				bodyEl.querySelector<HTMLElement>(`#${CSS.escape(fragment)}`) ??
				bodyEl.querySelector<HTMLElement>(`[name="${CSS.escape(fragment)}"]`);
			if (anchor) {
				anchor.scrollIntoView({ behavior: 'smooth', block: 'start' });
				return;
			}
		}
	}

	export function scrollToChapter(index: number, charOffset = 0, fragment?: string) {
		if (!containerEl) return;
		const target = containerEl.querySelector<HTMLElement>(`[data-chapter-index="${index}"]`);
		if (!target) return;

		if (fragment) {
			const bodyEl = chapterBodyEls.get(index);
			if (bodyEl) {
				const anchor =
					bodyEl.querySelector<HTMLElement>(`#${CSS.escape(fragment)}`) ??
					bodyEl.querySelector<HTMLElement>(`[name="${CSS.escape(fragment)}"]`);
				if (anchor) {
					anchor.scrollIntoView({ behavior: 'smooth', block: 'start' });
					return;
				}
			}
			// Chapter not loaded yet — store fragment so loadChapter can scroll after render
			pendingFragment = fragment;
		}

		if (charOffset > 0) {
			const bodyEl = chapterBodyEls.get(index);
			if (bodyEl) {
				const walker = document.createTreeWalker(bodyEl, NodeFilter.SHOW_TEXT);
				let charCount = 0;
				let node = walker.nextNode();
				while (node) {
					const len = node.textContent?.length ?? 0;
					if (charCount + len >= charOffset) {
						const range = document.createRange();
						range.setStart(node, Math.min(charOffset - charCount, len));
						range.collapse(true);
						const rect = range.getBoundingClientRect();
						const scrollRect = containerEl!.getBoundingClientRect();
						containerEl!.scrollTo({
							top: containerEl!.scrollTop + rect.top - scrollRect.top - 100,
							behavior: 'smooth'
						});
						return;
					}
					charCount += len;
					node = walker.nextNode();
				}
			}
		}

		target.scrollIntoView({ behavior: 'smooth', block: 'start' });
	}
</script>

<div class="epub-scroll-container" bind:this={containerEl} onscroll={handleScroll}>
	<div
		class="epub-scroll-content"
		data-reader-theme={sepiaTheme}
		style:--reader-font-family={prefs.fontFamily}
		style:--reader-font-size="{prefs.fontSize}px"
		style:--reader-line-height={prefs.lineHeight}
		style:--reader-content-width="{prefs.contentWidth}px"
		style:--reader-paragraph-spacing="{prefs.paragraphSpacing}em"
		style:--reader-text-align={prefs.textAlign}
		onclick={handleContentClick}
	>
		{#each chapterEntries as entry (entry.index)}
			<div class="epub-chapter-wrapper" data-chapter-index={entry.index} data-chapter-id={entry.id}>
				<div class="chapter-label"></div>
				<h1 class="chapter-title"></h1>
				<div class="book-body"></div>
				<div class="chapter-placeholder" style:height="{estimateChapterHeight(entry)}px"></div>
			</div>
		{/each}
	</div>
</div>

<style>
	.epub-scroll-container {
		flex: 1;
		overflow-y: auto;
		overflow-x: hidden;
		padding: 40px 0 80px;
	}

	.epub-scroll-container::-webkit-scrollbar {
		width: 6px;
	}

	.epub-scroll-container::-webkit-scrollbar-track {
		background: transparent;
	}

	.epub-scroll-container::-webkit-scrollbar-thumb {
		background: var(--text-quaternary);
		border-radius: 3px;
	}

	.epub-scroll-content {
		max-width: var(--reader-content-width, 680px);
		margin: 0 auto;
		padding: 0 40px;
		width: 100%;
	}

	.epub-chapter-wrapper {
		padding-bottom: 48px;
		margin-bottom: 48px;
		border-bottom: 1px solid var(--border-primary);
	}

	.epub-chapter-wrapper:last-child {
		border-bottom: none;
		margin-bottom: 0;
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

	.chapter-title:empty {
		display: none;
	}

	.chapter-placeholder {
		display: flex;
		align-items: center;
		justify-content: center;
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

	.book-body:empty {
		display: none;
	}

	.book-body :global(p) {
		margin-bottom: var(--reader-paragraph-spacing, 1.2em);
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
	.epub-scroll-content[data-reader-theme='sepia'] {
		--reader-bg: #f5edda;
		--reader-text: #5b4636;
		--reader-text-secondary: #8b7355;
	}

	.epub-scroll-content[data-reader-theme='sepia'] :global(*) {
		color: var(--reader-text);
	}

	@media (max-width: 599px) {
		.epub-scroll-content {
			padding: 0 20px;
		}
	}
</style>
