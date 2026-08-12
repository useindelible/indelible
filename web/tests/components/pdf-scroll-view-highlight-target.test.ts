import { render, waitFor } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { HighlightWithNoteResponse } from '$lib/api';
import type { BookSource, PDFPageProxy } from '$lib/components/reader/book/book-source';

const mocks = vi.hoisted(() => ({
	renderCanvas: vi.fn(async () => undefined),
	renderPdfTextLayer: vi.fn(async () => ({ hasText: true, cancel: vi.fn() }))
}));

vi.mock('$lib/components/reader/book/pdf/pdf-canvas-renderer', () => ({
	computeScale: () => ({ cssScale: 1, dpr: 1 }),
	renderCanvas: (...args: unknown[]) => mocks.renderCanvas(...args)
}));
vi.mock('$lib/components/reader/book/pdf/pdf-text-layer', () => ({
	renderPdfTextLayer: (...args: unknown[]) => mocks.renderPdfTextLayer(...args)
}));
vi.mock('$lib/stores/reader-preferences.svelte', () => ({
	getReaderPreferences: () => ({ theme: 'light' })
}));

import PdfScrollView from '$lib/components/reader/book/PdfScrollView.svelte';

let intersectionCallback: IntersectionObserverCallback;

class StubIntersectionObserver implements IntersectionObserver {
	readonly root = null;
	readonly rootMargin = '';
	readonly thresholds = [];
	disconnect = vi.fn();
	observe = vi.fn();
	takeRecords = vi.fn(() => []);
	unobserve = vi.fn();

	constructor(callback: IntersectionObserverCallback) {
		intersectionCallback = callback;
	}
}

function source(): BookSource {
	const page = {
		getViewport: () => ({ width: 612, height: 792 })
	} as PDFPageProxy;
	return {
		metadata: { title: 'Target PDF', totalChapters: 1 },
		toc: [],
		loadPage: vi.fn(async () => ({ type: 'pdf' as const, page, width: 612, height: 792 })),
		destroy: vi.fn()
	};
}

function targetHighlight(id = 'hl_target', y = 0.2): HighlightWithNoteResponse {
	return {
		id,
		document_id: 'doc_1',
		color: 'yellow',
		text_content: 'Target passage',
		locator: {
			type: 'pdf',
			page: 1,
			x: 0.1,
			y,
			width: 0.3,
			height: 0.1,
			text_snapshot: 'Target passage'
		},
		note: null,
		tags: [],
		created_at: '2026-08-12T00:00:00Z',
		updated_at: '2026-08-12T00:00:00Z'
	};
}

describe('PdfScrollView highlight targets', () => {
	beforeEach(() => {
		vi.stubGlobal('IntersectionObserver', StubIntersectionObserver);
		Element.prototype.scrollIntoView = vi.fn();
		mocks.renderCanvas.mockClear();
		mocks.renderPdfTextLayer.mockClear();
	});

	it('scrolls to the exact target overlay once after the page renders', async () => {
		const pdfSource = source();
		const highlight = targetHighlight();
		const rendered = render(PdfScrollView, {
			props: {
				source: pdfSource,
				highlights: [highlight],
				targetHighlightId: highlight.id
			}
		});

		await waitFor(() => expect(intersectionCallback).toBeTypeOf('function'));
		const page = rendered.container.querySelector<HTMLElement>('[data-pdf-page="1"]')!;
		intersectionCallback(
			[{ isIntersecting: true, target: page } as IntersectionObserverEntry],
			{} as IntersectionObserver
		);

		await waitFor(() => {
			const overlay = rendered.container.querySelector<HTMLElement>(
				'[data-highlight-id="hl_target"]'
			);
			expect(overlay).toBeTruthy();
			expect(overlay!.scrollIntoView).toHaveBeenCalledOnce();
		});

		await rendered.rerender({
			source: pdfSource,
			highlights: [{ ...highlight }],
			targetHighlightId: highlight.id
		});

		expect(Element.prototype.scrollIntoView).toHaveBeenCalledOnce();
	});

	it('scrolls once to each new target on an already-rendered page', async () => {
		const first = targetHighlight('hl_first', 0.2);
		const second = targetHighlight('hl_second', 0.6);
		const pdfSource = source();
		const highlights = [first, second];
		const rendered = render(PdfScrollView, {
			props: {
				source: pdfSource,
				highlights,
				targetHighlightId: first.id
			}
		});

		await waitFor(() => expect(intersectionCallback).toBeTypeOf('function'));
		const page = rendered.container.querySelector<HTMLElement>('[data-pdf-page="1"]')!;
		intersectionCallback(
			[{ isIntersecting: true, target: page } as IntersectionObserverEntry],
			{} as IntersectionObserver
		);
		await waitFor(() => expect(Element.prototype.scrollIntoView).toHaveBeenCalledOnce());

		await rendered.rerender({
			source: pdfSource,
			highlights,
			targetHighlightId: second.id
		});

		await waitFor(() => expect(Element.prototype.scrollIntoView).toHaveBeenCalledTimes(2));
		const secondOverlay = rendered.container.querySelector<HTMLElement>(
			'[data-highlight-id="hl_second"]'
		);
		expect(vi.mocked(Element.prototype.scrollIntoView).mock.contexts[1]).toBe(secondOverlay);
		expect(Element.prototype.scrollIntoView).toHaveBeenLastCalledWith({
			behavior: 'smooth',
			block: 'center'
		});

		await rendered.rerender({
			source: pdfSource,
			highlights: [{ ...first }, { ...second }],
			targetHighlightId: second.id
		});
		expect(Element.prototype.scrollIntoView).toHaveBeenCalledTimes(2);
	});
});
