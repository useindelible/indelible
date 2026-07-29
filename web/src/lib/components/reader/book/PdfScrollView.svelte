<script lang="ts">
	import { onMount, untrack } from 'svelte';
	import { SvelteSet, SvelteMap } from 'svelte/reactivity';
	import type { BookSource } from './book-source';
	import type { HighlightWithNoteResponse } from '$lib/api/generated/types.gen';
	import { computeScale, renderCanvas } from './pdf/pdf-canvas-renderer';
	import { renderPdfTextLayer } from './pdf/pdf-text-layer';
	import { applyThemeRemap, type PdfThemeMode } from './pdf/pdf-dark-mode';
	import {
		renderHighlightOverlays,
		type PdfHighlightData,
		type PdfLocator
	} from './pdf/pdf-highlight-overlay';
	import './pdf/pdf-text-layer.css';
	import { getReaderPreferences } from '$lib/stores/reader-preferences.svelte';
	import { scrollProgressPercent } from '../progress-geometry';

	interface Props {
		source: BookSource;
		highlights: HighlightWithNoteResponse[];
		initialPage?: number;
		onPageChange?: (pageIndex: number) => void;
		onProgress?: (progress: number, pageIndex: number) => void;
		scrollContainerEl?: HTMLDivElement;
	}

	let {
		source,
		highlights,
		initialPage = 0,
		onPageChange,
		onProgress,
		scrollContainerEl = $bindable()
	}: Props = $props();

	const prefs = getReaderPreferences();
	const totalPages = source.metadata.totalChapters;

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

	let containerEl = $state<HTMLDivElement | undefined>(undefined);
	let currentPage = $state(initialPage);
	let hasScrolledToInitial = $state(initialPage <= 0);

	$effect(() => {
		scrollContainerEl = containerEl;
	});

	// Page rendering state tracked imperatively (not reactive per-page)
	const renderedPages = new SvelteSet<number>();
	const pageCanvases = new SvelteMap<number, HTMLCanvasElement>();
	const cancelFns = new SvelteMap<number, () => void>();
	let observer: IntersectionObserver | null = null;

	// Estimate page dimensions from first page (most PDFs are uniform)
	let defaultPageWidth = $state(612);
	let defaultPageHeight = $state(792);
	let dimensionsReady = $state(false);

	onMount(() => {
		loadFirstPageDimensions();
	});

	async function loadFirstPageDimensions() {
		try {
			const content = await source.loadPage(0);
			if (content.type === 'pdf') {
				defaultPageWidth = content.width;
				defaultPageHeight = content.height;
			}
		} catch {
			// Use defaults
		}
		dimensionsReady = true;
	}

	$effect(() => {
		if (!dimensionsReady || !containerEl || !hasScrolledToInitial) return;

		observer = new IntersectionObserver(handleIntersection, {
			root: containerEl,
			rootMargin: '200px 0px'
		});

		const pageContainers = containerEl.querySelectorAll<HTMLElement>('[data-pdf-page]');
		pageContainers.forEach((el) => observer!.observe(el));

		return () => {
			observer?.disconnect();
			observer = null;
			for (const cancel of cancelFns.values()) cancel();
			cancelFns.clear();
			renderedPages.clear();
			pageCanvases.clear();
		};
	});

	// Scroll to initial page after placeholders are created
	$effect(() => {
		if (!dimensionsReady || !containerEl || hasScrolledToInitial || initialPage <= 0) return;
		const target = containerEl.querySelector<HTMLElement>(`[data-pdf-page="${initialPage + 1}"]`);
		if (!target) return;

		requestAnimationFrame(() => {
			if (!containerEl) return;
			containerEl.scrollTop = Math.max(0, target.offsetTop - 8);
			hasScrolledToInitial = true;
		});
	});

	// Re-render visible pages when theme changes.
	// `reRenderAllVisible` reads and writes `renderedPages`, so it must run
	// untracked — otherwise Svelte subscribes this effect to `renderedPages`
	// and the delete+add cycle during re-render causes effect_update_depth_exceeded.
	$effect(() => {
		void pdfThemeMode;
		untrack(() => reRenderAllVisible());
	});

	// Re-render highlight overlays when highlights change.
	// `updateAllHighlightOverlays` iterates `renderedPages`; untrack so this
	// effect is driven by `highlights`, not by pages being rendered.
	$effect(() => {
		void highlights;
		if (!containerEl) return;
		untrack(() => updateAllHighlightOverlays());
	});

	function handleIntersection(entries: IntersectionObserverEntry[]) {
		for (const entry of entries) {
			const pageEl = entry.target as HTMLElement;
			const pageNum = parseInt(pageEl.dataset.pdfPage ?? '0', 10);
			if (pageNum < 1) continue;
			const pageIndex = pageNum - 1;

			if (entry.isIntersecting) {
				if (!renderedPages.has(pageIndex)) {
					renderPage(pageIndex, pageEl);
				}
			}
		}

		updateCurrentPage();
	}

	function updateCurrentPage() {
		if (!containerEl) return;
		const containerRect = containerEl.getBoundingClientRect();
		const centerY = containerRect.top + containerRect.height / 2;

		const wrappers = containerEl.querySelectorAll<HTMLElement>('[data-pdf-page]');
		let closest = 0;
		let closestDist = Infinity;

		for (const wrapper of wrappers) {
			const rect = wrapper.getBoundingClientRect();
			const pageCenterY = rect.top + rect.height / 2;
			const dist = Math.abs(pageCenterY - centerY);
			const pageNum = parseInt(wrapper.dataset.pdfPage ?? '1', 10);
			if (dist < closestDist) {
				closestDist = dist;
				closest = pageNum - 1;
			}
		}

		if (closest !== currentPage) {
			currentPage = closest;
			onPageChange?.(closest);
		}

		onProgress?.(scrollProgressPercent(containerEl), closest);
	}

	async function renderPage(pageIndex: number, pageContainerEl: HTMLElement) {
		renderedPages.add(pageIndex);

		const canvas = pageContainerEl.querySelector('canvas') as HTMLCanvasElement;
		const textLayerDiv = pageContainerEl.querySelector('.pdf-text-container') as HTMLDivElement;
		if (!canvas || !textLayerDiv) return;

		pageCanvases.set(pageIndex, canvas);

		try {
			const content = await source.loadPage(pageIndex);
			if (content.type !== 'pdf') return;

			const containerWidth = (containerEl?.clientWidth ?? 800) - 80;
			const { cssScale, dpr } = computeScale(content.width, containerWidth);

			// Set CSS custom properties on the page container for text layer scaling
			pageContainerEl.style.setProperty('--scale-factor', String(cssScale));
			pageContainerEl.style.setProperty('--total-scale-factor', String(cssScale));
			pageContainerEl.style.setProperty('--scale-round-x', '1px');
			pageContainerEl.style.setProperty('--scale-round-y', '1px');

			await renderCanvas(content.page, canvas, cssScale, dpr);

			const viewport = content.page.getViewport({ scale: cssScale });
			const result = await renderPdfTextLayer(content.page, textLayerDiv, viewport);
			cancelFns.set(pageIndex, result.cancel);

			applyThemeRemap(canvas, pdfThemeMode);

			// Update wrapper height to match actual rendered size
			const wrapperEl = pageContainerEl.parentElement;
			if (wrapperEl) wrapperEl.style.height = `${viewport.height}px`;

			updateHighlightOverlaysForPage(pageIndex, pageContainerEl);
		} catch {
			renderedPages.delete(pageIndex);
		}
	}

	async function reRenderAllVisible() {
		if (!containerEl) return;
		const pagesToRerender = [...renderedPages];
		for (const pageIndex of pagesToRerender) {
			const wrapperEl = containerEl.querySelector(
				`[data-pdf-page="${pageIndex + 1}"]`
			) as HTMLElement;
			if (!wrapperEl) continue;
			renderedPages.delete(pageIndex);
			cancelFns.get(pageIndex)?.();
			cancelFns.delete(pageIndex);
			renderPage(pageIndex, wrapperEl);
		}
	}

	function updateAllHighlightOverlays() {
		if (!containerEl) return;
		for (const pageIndex of renderedPages) {
			const wrapperEl = containerEl.querySelector(
				`[data-pdf-page="${pageIndex + 1}"]`
			) as HTMLElement;
			if (!wrapperEl) continue;
			updateHighlightOverlaysForPage(pageIndex, wrapperEl);
		}
	}

	function updateHighlightOverlaysForPage(pageIndex: number, wrapperEl: HTMLElement) {
		const highlightLayer = wrapperEl.querySelector('.pdf-highlight-layer') as HTMLElement;
		if (!highlightLayer) return;

		const pdfHighlights: PdfHighlightData[] = highlights.flatMap((h) =>
			h.color !== 'bookmark' && h.locator?.type === 'pdf'
				? [
						{
							id: h.id,
							color: h.color,
							locator: h.locator as PdfLocator
						}
					]
				: []
		);

		renderHighlightOverlays(highlightLayer, pdfHighlights, pageIndex + 1);
	}

	function handleScroll() {
		if (!hasScrolledToInitial) return;
		updateCurrentPage();
	}

	const placeholderHeight = $derived(
		containerEl
			? defaultPageHeight * ((containerEl.clientWidth - 80) / defaultPageWidth)
			: defaultPageHeight
	);

	export function scrollToPage(pageIndex: number) {
		if (!containerEl) return;
		const target = containerEl.querySelector(`[data-pdf-page="${pageIndex + 1}"]`);
		if (target) {
			target.scrollIntoView({ behavior: 'smooth', block: 'start' });
		}
	}
</script>

{#if dimensionsReady}
	<div class="pdf-scroll-container" bind:this={containerEl} onscroll={handleScroll}>
		{#each Array.from({ length: totalPages }, (_, i) => i) as pageIndex (pageIndex)}
			<div class="pdf-page-wrapper" style:height="{placeholderHeight}px">
				<div class="pdf-page-container" data-pdf-page={pageIndex + 1}>
					<canvas></canvas>
					<div class="pdf-text-container"></div>
					<div class="pdf-highlight-layer"></div>
				</div>
			</div>
		{/each}
	</div>
{:else}
	<div class="pdf-scroll-loading">
		<span class="loading-text">Loading PDF...</span>
	</div>
{/if}

<style>
	.pdf-scroll-container {
		flex: 1;
		overflow-y: auto;
		overflow-x: hidden;
		padding: 20px 0;
	}

	.pdf-scroll-container::-webkit-scrollbar {
		width: 6px;
	}

	.pdf-scroll-container::-webkit-scrollbar-track {
		background: transparent;
	}

	.pdf-scroll-container::-webkit-scrollbar-thumb {
		background: var(--text-quaternary);
		border-radius: 3px;
	}

	.pdf-page-wrapper {
		margin: 0 auto 16px;
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

	.pdf-scroll-loading {
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
</style>
