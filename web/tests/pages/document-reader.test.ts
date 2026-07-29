import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { DocumentListEntry, DocumentReaderAssetResponse } from '$lib/api';

// TASK-232: the document reader loads the read-model, distinguishes prepared-but-unsaved from
// saved, shows a preparing state until the readable asset lands, and saves to Library by URL.

const mockGetDocumentEntry = vi.fn();
const mockListAssets = vi.fn();
const mockListHighlights = vi.fn();
const mockStreamAsset = vi.fn();
const mockCreateDocumentEntry = vi.fn();
const mockListDocumentEntities = vi.fn();
const mockReprocessDocument = vi.fn();

vi.mock('$app/state', () => ({
	page: {
		params: { documentId: 'doc_1' },
		url: { pathname: '/reader/doc_1', searchParams: { get: () => null } }
	}
}));
vi.mock('$app/navigation', () => ({ goto: vi.fn(), afterNavigate: vi.fn() }));
vi.mock('$app/paths', () => ({ resolve: (path: string) => path }));
vi.mock('$app/environment', () => ({ browser: true }));

vi.mock('$lib/api', () => ({
	getDocumentEntry: (...a: unknown[]) => mockGetDocumentEntry(...a),
	listAssets: (...a: unknown[]) => mockListAssets(...a),
	listHighlights: (...a: unknown[]) => mockListHighlights(...a),
	streamAsset: (...a: unknown[]) => mockStreamAsset(...a),
	reprocessDocument: (...a: unknown[]) => mockReprocessDocument(...a),
	updateProgress: vi.fn(),
	createDocumentEntry: (...a: unknown[]) => mockCreateDocumentEntry(...a),
	listDocumentEntities: (...a: unknown[]) => mockListDocumentEntities(...a),
	createHighlight: vi.fn(),
	deleteHighlight: vi.fn(),
	patchHighlight: vi.fn(),
	setHighlightTags: vi.fn()
}));
vi.mock('$lib/styles/theme', () => ({
	applyTheme: vi.fn(),
	getSavedTheme: vi.fn(() => 'system')
}));

// The reader content and highlight toolbar are exercised by their own tests; stub them here so
// this test focuses on the page's read-model wiring and save-state.
function stub() {
	return {
		default: vi.fn(() => ({ c: vi.fn(), m: vi.fn(), p: vi.fn(), d: vi.fn() }))
	};
}
vi.mock('$lib/components/reader/ReaderContent.svelte', () => stub());
vi.mock('$lib/components/reader/HighlightToolbar.svelte', () => stub());
vi.mock('$lib/components/reader/book/PdfScrollView.svelte', () => stub());

import DocumentReaderPage from '../../src/routes/(app)/reader/[documentId]/+page.svelte';

function readModel(overrides: Partial<DocumentListEntry> = {}): DocumentListEntry {
	return {
		id: 'doc_1',
		document_id: 'doc_1',
		title: 'A prepared article',
		url: 'https://example.com/article',
		document_type: 'article',
		item_type: 'article',
		object: 'library_entry',
		source: 'web',
		created_at: '2026-05-18T10:00:00Z',
		updated_at: '2026-05-18T10:00:00Z',
		saved_at: '2026-05-18T10:00:00Z',
		triage_state: 'inbox',
		is_favorite: false,
		is_shortlisted: false,
		library_entry_id: null,
		saved: false,
		available_assets: ['readable_html'],
		readable_ready: true,
		...overrides
	};
}

function assets(items: Partial<DocumentReaderAssetResponse>[] = []): DocumentReaderAssetResponse[] {
	return items.map((item, index) => ({
		id: `asset_${index}`,
		asset_kind: 'readable_html',
		content_type: 'text/html',
		created_at: '2026-05-18T10:00:00Z',
		size_bytes: 123,
		status: 'completed',
		...item
	}));
}

beforeEach(() => {
	vi.clearAllMocks();
	Object.defineProperty(window, 'matchMedia', {
		writable: true,
		value: vi.fn().mockImplementation((query: string) => ({
			matches: false,
			media: query,
			onchange: null,
			addListener: vi.fn(),
			removeListener: vi.fn(),
			addEventListener: vi.fn(),
			removeEventListener: vi.fn(),
			dispatchEvent: vi.fn()
		}))
	});
	mockListAssets.mockResolvedValue({ data: { data: assets(), page: { has_more: false } } });
	mockListHighlights.mockResolvedValue({ data: { highlights: [], count: 0 } });
	mockStreamAsset.mockResolvedValue({ data: '<p>body</p>' });
	mockListDocumentEntities.mockResolvedValue({ data: [] });
	mockReprocessDocument.mockResolvedValue({
		data: { queued: true, job_type: 'document.reprocess' }
	});
});

afterEach(() => {
	vi.useRealTimers();
});

describe('document reader page', () => {
	it('shows a preparing state until the readable asset is ready', async () => {
		mockGetDocumentEntry.mockResolvedValue({ data: readModel({ readable_ready: false }) });
		mockListAssets.mockResolvedValue({ data: { data: [], page: { has_more: false } } });
		render(DocumentReaderPage);
		await waitFor(() => expect(screen.getByTestId('preparing-reader')).toBeTruthy());
	});

	it('offers Save to Library for a prepared-but-unsaved document', async () => {
		mockGetDocumentEntry.mockResolvedValue({
			data: readModel({ readable_ready: true, saved: false, library_entry_id: null })
		});
		render(DocumentReaderPage);
		await waitFor(() => expect(screen.getByRole('button', { name: 'Save' })).toBeTruthy());
	});

	it('shows the Library state for a saved document', async () => {
		mockGetDocumentEntry.mockResolvedValue({
			data: readModel({ saved: true, library_entry_id: 'lib_1' })
		});
		render(DocumentReaderPage);
		await waitFor(() => expect(screen.queryByRole('button', { name: 'Save' })).toBeNull());
	});

	it('saves to Library by the document URL', async () => {
		mockGetDocumentEntry.mockResolvedValue({ data: readModel({ saved: false }) });
		mockCreateDocumentEntry.mockResolvedValue({
			data: readModel({ saved: true, library_entry_id: 'lib_1' })
		});
		render(DocumentReaderPage);
		const button = await screen.findByRole('button', { name: 'Save' });
		await fireEvent.click(button);
		await waitFor(() =>
			expect(mockCreateDocumentEntry).toHaveBeenCalledWith({
				body: {
					url: 'https://example.com/article',
					title: 'A prepared article',
					item_type: 'article'
				}
			})
		);
		await waitFor(() => expect(screen.queryByRole('button', { name: 'Save' })).toBeNull());
	});

	it.each(['failed', 'degraded'])(
		'reprocesses a %s reader asset before retrying preparation polling',
		async (status) => {
			vi.useFakeTimers();
			mockGetDocumentEntry.mockResolvedValue({
				data: readModel({ readable_ready: false, available_assets: [] })
			});
			mockListAssets.mockResolvedValue({
				data: {
					data: assets([{ status, failed_reason: 'readable extraction failed' }]),
					page: { has_more: false }
				}
			});

			render(DocumentReaderPage);
			await waitFor(() => expect(screen.getByTestId('preparing-reader')).toBeTruthy());
			await vi.advanceTimersByTimeAsync(30_001);

			const retry = await screen.findByTestId('reader-retry');
			await fireEvent.click(retry);
			await vi.advanceTimersByTimeAsync(0);

			expect(mockReprocessDocument).toHaveBeenCalledWith({ path: { document_id: 'doc_1' } });
			expect(screen.getByText('Reprocessing queued.')).toBeTruthy();
			expect((screen.getByTestId('reader-retry') as HTMLButtonElement).disabled).toBe(true);
		}
	);

	it('shows and expires the server-provided reprocess cooldown', async () => {
		vi.useFakeTimers();
		mockReprocessDocument.mockResolvedValueOnce({
			data: { queued: false, job_type: 'document.reprocess', retry_after_seconds: 45 }
		});
		mockGetDocumentEntry.mockResolvedValue({
			data: readModel({ readable_ready: false, available_assets: [] })
		});
		mockListAssets.mockResolvedValue({
			data: {
				data: assets([{ status: 'failed', failed_reason: 'readable extraction failed' }]),
				page: { has_more: false }
			}
		});

		render(DocumentReaderPage);
		await waitFor(() => expect(screen.getByTestId('preparing-reader')).toBeTruthy());
		await vi.advanceTimersByTimeAsync(30_001);
		await fireEvent.click(await screen.findByTestId('reader-retry'));
		await vi.advanceTimersByTimeAsync(0);

		expect(screen.getByText('Retry available in 45 seconds.')).toBeTruthy();
		expect((screen.getByTestId('reader-retry') as HTMLButtonElement).disabled).toBe(true);
		await vi.advanceTimersByTimeAsync(45_000);
		expect((screen.getByTestId('reader-retry') as HTMLButtonElement).disabled).toBe(false);
	});

	it('keeps retry visible and reports when reprocess enqueue fails', async () => {
		vi.useFakeTimers();
		mockReprocessDocument.mockRejectedValueOnce(new Error('network'));
		mockGetDocumentEntry.mockResolvedValue({
			data: readModel({ readable_ready: false, available_assets: [] })
		});
		mockListAssets.mockResolvedValue({
			data: {
				data: assets([{ status: 'failed', failed_reason: 'readable extraction failed' }]),
				page: { has_more: false }
			}
		});

		render(DocumentReaderPage);
		await waitFor(() => expect(screen.getByTestId('preparing-reader')).toBeTruthy());
		await vi.advanceTimersByTimeAsync(30_001);

		const retry = await screen.findByTestId('reader-retry');
		const callsBeforeRetry = mockGetDocumentEntry.mock.calls.length;
		await fireEvent.click(retry);

		await waitFor(() =>
			expect(screen.getByText('Could not queue reprocessing. Try again.')).toBeTruthy()
		);
		expect(screen.getByTestId('reader-retry')).toBeTruthy();
		expect(mockGetDocumentEntry.mock.calls.length).toBe(callsBeforeRetry);
	});

	it('keeps plain long-running preparation retry as polling only', async () => {
		vi.useFakeTimers();
		mockGetDocumentEntry.mockResolvedValue({
			data: readModel({ readable_ready: false, available_assets: [] })
		});
		mockListAssets.mockResolvedValue({ data: { data: [], page: { has_more: false } } });

		render(DocumentReaderPage);
		await waitFor(() => expect(screen.getByTestId('preparing-reader')).toBeTruthy());
		await vi.advanceTimersByTimeAsync(30_001);

		const retry = await screen.findByTestId('reader-retry');
		const callsBeforeRetry = mockGetDocumentEntry.mock.calls.length;
		await fireEvent.click(retry);

		expect(mockReprocessDocument).not.toHaveBeenCalled();
		await waitFor(() =>
			expect(mockGetDocumentEntry.mock.calls.length).toBeGreaterThan(callsBeforeRetry)
		);
	});
});
