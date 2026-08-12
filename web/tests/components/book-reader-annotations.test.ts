import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { DocumentListEntry, DocumentReaderAssetResponse, HighlightResponse } from '$lib/api';
import type { BookSource } from '$lib/components/reader/book/book-source';
import { isImageOnlyPdf } from '$lib/components/reader/book/book-reader-model';

const mocks = vi.hoisted(() => ({
	createHighlight: vi.fn(),
	createEpubSource: vi.fn(),
	createPdfSource: vi.fn(),
	scrollToChapter: vi.fn(),
	streamAsset: vi.fn(),
	updateProgress: vi.fn(async () => ({ data: {} }))
}));

vi.mock('$app/navigation', () => ({ goto: vi.fn(), afterNavigate: vi.fn() }));
vi.mock('$app/paths', () => ({ resolve: (path: string) => path }));
vi.mock('$app/environment', () => ({ browser: true }));
vi.mock('$lib/styles/theme', () => ({ applyTheme: vi.fn(), getSavedTheme: vi.fn(() => 'system') }));
vi.mock('$lib/api', async (importOriginal) => {
	const original = await importOriginal<typeof import('$lib/api')>();
	return {
		...original,
		createHighlight: (...args: unknown[]) => mocks.createHighlight(...args),
		streamAsset: (...args: unknown[]) => mocks.streamAsset(...args),
		updateProgress: (...args: unknown[]) => mocks.updateProgress(...args)
	};
});
vi.mock('$lib/components/reader/book/book-source', async (importOriginal) => {
	const original = await importOriginal<typeof import('$lib/components/reader/book/book-source')>();
	return {
		...original,
		createEpubSource: (...args: unknown[]) => mocks.createEpubSource(...args),
		createPdfSource: (...args: unknown[]) => mocks.createPdfSource(...args)
	};
});
vi.mock('$lib/components/reader/book/PdfScrollView.svelte', () => ({
	default: vi.fn(() => ({ c: vi.fn(), m: vi.fn(), p: vi.fn(), d: vi.fn() }))
}));
vi.mock('$lib/components/reader/book/EpubScrollView.svelte', () => ({
	default: vi.fn(() => ({ scrollToChapter: mocks.scrollToChapter }))
}));

import BookReader from '$lib/components/reader/book/BookReader.svelte';

function source(): BookSource {
	return {
		metadata: {
			title: 'Annotation Boundaries',
			author: 'Indelible',
			totalChapters: 1
		},
		toc: [
			{
				id: 'chapter-1',
				title: 'Chapter 1',
				depth: 1,
				index: 0,
				wordCount: 100,
				startPage: 1
			}
		],
		async loadPage() {
			return {
				type: 'html',
				html: '<p>Chapter body</p>',
				id: 'chapter-1',
				title: 'Chapter 1',
				wordCount: 100
			};
		},
		destroy() {}
	};
}

function mixedDepthSource(): BookSource {
	return {
		...source(),
		metadata: { title: 'Mixed Depth', author: 'Indelible', totalChapters: 3 },
		toc: [
			{ id: 'opening', title: 'Opening', depth: 1, index: 0 },
			{ id: 'signal', title: 'Signal', depth: 2, index: 1, fragment: 'signal' },
			{ id: 'closing', title: 'Closing', depth: 1, index: 2 }
		]
	};
}

function item(): DocumentListEntry {
	return {
		id: 'doc_1',
		document_id: 'doc_1',
		title: 'Annotation Boundaries',
		document_type: 'book',
		item_type: 'book',
		object: 'library_entry',
		source: 'manual',
		created_at: '2026-08-11T00:00:00Z',
		updated_at: '2026-08-11T00:00:00Z',
		saved_at: '2026-08-11T00:00:00Z',
		triage_state: 'inbox',
		is_favorite: false,
		is_shortlisted: false,
		progress_percent: 0
	} as DocumentListEntry;
}

function createdBookmark(): HighlightResponse {
	return {
		id: 'hl_1',
		color: 'bookmark',
		text_content: 'Bookmark',
		locator: {
			type: 'epub',
			chapter: 'chapter-1',
			start_offset: 0,
			end_offset: 0
		},
		created_at: '2026-08-11T00:00:00Z',
		updated_at: '2026-08-11T00:00:00Z'
	};
}

beforeEach(() => {
	mocks.createHighlight.mockReset();
	mocks.createEpubSource.mockReset().mockResolvedValue(source());
	mocks.createPdfSource.mockReset().mockResolvedValue(source());
	mocks.scrollToChapter.mockReset();
	mocks.streamAsset.mockReset().mockResolvedValue({ data: new Blob(['pdf']) });
	mocks.updateProgress.mockClear();
});

describe('BookReader chapter navigation', () => {
	it('forwards the representative fragment when moving to the next EPUB spine', async () => {
		mocks.createEpubSource.mockResolvedValue(mixedDepthSource());

		render(BookReader, { props: { item: item(), assets: [], highlights: [] } });
		await fireEvent.click(await screen.findByRole('button', { name: /Ch\. 2: Signal/ }));

		expect(mocks.scrollToChapter).toHaveBeenCalledWith(1, 0, 'signal');
	});
});

describe('BookReader image-only PDFs', () => {
	it('recognizes only the controlled image-only PDF extraction failures', () => {
		const failedExtraction: DocumentReaderAssetResponse = {
			id: 'asset_text',
			asset_kind: 'extracted_text',
			content_type: 'text/plain',
			created_at: '2026-08-12T00:00:00Z',
			size_bytes: 0,
			status: 'failed',
			failed_reason: 'PDF text extraction produced no text'
		};

		expect(isImageOnlyPdf({ ...item(), item_type: 'pdf' }, [failedExtraction])).toBe(true);
		expect(
			isImageOnlyPdf({ ...item(), item_type: 'pdf' }, [
				{
					...failedExtraction,
					failed_reason:
						'PDF text extraction failed: failed to extract text from PDF: no extractable text'
				}
			])
		).toBe(true);
		expect(isImageOnlyPdf(item(), [failedExtraction])).toBe(false);
		expect(
			isImageOnlyPdf({ ...item(), item_type: 'pdf' }, [
				{ ...failedExtraction, status: 'completed' }
			])
		).toBe(false);
		expect(
			isImageOnlyPdf({ ...item(), item_type: 'pdf' }, [
				{ ...failedExtraction, failed_reason: 'PDF extraction failed' }
			])
		).toBe(false);
		expect(
			isImageOnlyPdf({ ...item(), item_type: 'pdf' }, [
				{
					...failedExtraction,
					failed_reason:
						'PDF text extraction failed: failed to extract text from PDF: malformed content'
				}
			])
		).toBe(false);
	});

	it('explains which reading features remain available without extracted text', () => {
		const pdfItem = { ...item(), item_type: 'pdf', document_type: 'pdf' } as DocumentListEntry;
		const assets: DocumentReaderAssetResponse[] = [
			{
				id: 'asset_pdf',
				asset_kind: 'pdf',
				content_type: 'application/pdf',
				created_at: '2026-08-12T00:00:00Z',
				size_bytes: 100,
				status: 'completed'
			},
			{
				id: 'asset_text',
				asset_kind: 'extracted_text',
				content_type: 'text/plain',
				created_at: '2026-08-12T00:00:00Z',
				size_bytes: 0,
				status: 'failed',
				failed_reason: 'PDF text extraction produced no text'
			}
		];

		render(BookReader, { props: { item: pdfItem, assets, highlights: [] } });

		expect(screen.getByText(/This PDF has no searchable text/)).toBeTruthy();
		expect(
			screen.getByText(
				/Visual reading and bookmarks still work.*Mila chat and text actions are unavailable.*OCR is not available in this release/
			)
		).toBeTruthy();
	});
});

describe('BookReader annotation failures', () => {
	it('keeps failed bookmarks out of local state and clears the server error after a later success', async () => {
		mocks.createHighlight
			.mockResolvedValueOnce({
				data: undefined,
				error: { detail: 'Canonical annotation source is not ready.' }
			})
			.mockResolvedValueOnce({ data: createdBookmark(), error: undefined });

		render(BookReader, { props: { item: item(), assets: [], highlights: [] } });
		const addBookmark = await screen.findByRole('button', { name: 'Add bookmark' });

		await fireEvent.click(addBookmark);
		expect(mocks.createHighlight).toHaveBeenNthCalledWith(
			1,
			expect.objectContaining({
				body: expect.objectContaining({
					color: 'bookmark',
					locator: expect.objectContaining({ start_offset: 0, end_offset: 1 })
				})
			})
		);
		expect((await screen.findByRole('alert')).textContent).toContain(
			'Canonical annotation source is not ready.'
		);
		await fireEvent.click(screen.getByRole('button', { name: 'Bookmarks' }));
		expect(screen.getByText('No bookmarks yet')).toBeTruthy();

		await fireEvent.click(addBookmark);
		await waitFor(() => expect(screen.queryByRole('alert')).toBeNull());
		expect(await screen.findByText('Bookmark')).toBeTruthy();
	});

	it('shows a stable fallback when the annotation request throws', async () => {
		mocks.createHighlight.mockRejectedValueOnce(new TypeError('Failed to fetch'));

		render(BookReader, { props: { item: item(), assets: [], highlights: [] } });
		await fireEvent.click(await screen.findByRole('button', { name: 'Add bookmark' }));

		expect((await screen.findByRole('alert')).textContent).toContain(
			'Could not save annotation. Please try again.'
		);
	});
});
