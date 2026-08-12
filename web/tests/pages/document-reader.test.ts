import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { tick } from 'svelte';
import { SvelteMap } from 'svelte/reactivity';
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
const mockRetryMilaDocumentAction = vi.fn();
const mockGetArticleToc = vi.fn();
let readerRealtimeCallbacks:
	| {
			onAiCompleted: (completion: { action: string; aiRunId: string }) => void;
			onAiFailed: (failure: {
				documentId: string;
				action: string;
				aiRunId: string;
				message: string;
			}) => void;
	  }
	| undefined;
const mockRouteParams = new SvelteMap<string, string>([['documentId', 'doc_1']]);

vi.mock('$app/state', () => ({
	page: {
		params: new Proxy({}, { get: (_target, key) => mockRouteParams.get(String(key)) }),
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
	setHighlightTags: vi.fn(),
	retryMilaDocumentAction: (...a: unknown[]) => mockRetryMilaDocumentAction(...a),
	getArticleToc: (...a: unknown[]) => mockGetArticleToc(...a)
}));
vi.mock('../../src/routes/(app)/reader/[documentId]/reader-realtime', () => ({
	subscribeReaderRealtime: (_documentId: string, callbacks: typeof readerRealtimeCallbacks) => {
		readerRealtimeCallbacks = callbacks;
		return vi.fn();
	}
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
	mockRouteParams.set('documentId', 'doc_1');
	readerRealtimeCallbacks = undefined;
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
	mockRetryMilaDocumentAction.mockResolvedValue({ data: { queued: true, action: 'summary' } });
	mockGetArticleToc.mockResolvedValue({
		data: { status: 'none', truncated: false, entries: [] }
	});
});

afterEach(() => {
	vi.useRealTimers();
});

describe('document reader page', () => {
	it('reserves a safe reader gutter when the table of contents rail is visible', async () => {
		mockGetArticleToc.mockResolvedValue({
			data: {
				status: 'ready',
				truncated: false,
				entries: [
					{
						id: 'introduction',
						title: 'Introduction',
						depth: 0,
						source_heading_index: 0,
						word_count: 120
					}
				]
			}
		});
		mockGetDocumentEntry.mockResolvedValue({ data: readModel() });

		const { container } = render(DocumentReaderPage);

		await waitFor(() =>
			expect(mockGetArticleToc).toHaveBeenCalledWith({ path: { document_id: 'doc_1' } })
		);
		await waitFor(() =>
			expect(container.querySelector('.content-area')?.classList.contains('with-toc')).toBe(true)
		);
	});

	it('keeps the standard reader gutter when the table of contents is unavailable', async () => {
		mockGetDocumentEntry.mockResolvedValue({ data: readModel() });

		const { container } = render(DocumentReaderPage);

		await waitFor(() =>
			expect(mockGetArticleToc).toHaveBeenCalledWith({ path: { document_id: 'doc_1' } })
		);
		expect(container.querySelector('.content-area')?.classList.contains('with-toc')).toBe(false);
	});

	it('clears a Mila failure when the reader navigates to another document', async () => {
		mockGetDocumentEntry.mockResolvedValue({ data: readModel() });
		render(DocumentReaderPage);
		await waitFor(() => expect(readerRealtimeCallbacks).toBeTruthy());

		readerRealtimeCallbacks?.onAiFailed({
			documentId: 'doc_1',
			action: 'summary',
			aiRunId: 'airun_summary',
			message: 'provider timeout'
		});
		expect(await screen.findByText("Mila couldn't create a summary.")).toBeTruthy();

		mockRouteParams.set('documentId', 'doc_2');
		await waitFor(() => expect(screen.queryByText("Mila couldn't create a summary.")).toBeNull());
	});

	it('does not apply a stale retry result to a newer failure', async () => {
		let resolveRetry!: (value: { data: { queued: boolean; action: string } }) => void;
		mockRetryMilaDocumentAction.mockReturnValueOnce(
			new Promise((resolve) => {
				resolveRetry = resolve;
			})
		);
		mockGetDocumentEntry.mockResolvedValue({ data: readModel() });
		render(DocumentReaderPage);
		await waitFor(() => expect(readerRealtimeCallbacks).toBeTruthy());

		readerRealtimeCallbacks?.onAiFailed({
			documentId: 'doc_1',
			action: 'summary',
			aiRunId: 'airun_summary',
			message: 'summary timeout'
		});
		await fireEvent.click(await screen.findByRole('button', { name: 'Retry' }));
		readerRealtimeCallbacks?.onAiFailed({
			documentId: 'doc_1',
			action: 'tags',
			aiRunId: 'airun_tags',
			message: 'tags timeout'
		});
		resolveRetry({ data: { queued: true, action: 'summary' } });

		await waitFor(() => expect(screen.getByText("Mila couldn't suggest tags.")).toBeTruthy());
		expect(screen.queryByText('Retry queued.')).toBeNull();
	});

	it('clears a Mila failure only when the same action completes', async () => {
		mockGetDocumentEntry.mockResolvedValue({ data: readModel() });
		render(DocumentReaderPage);
		await waitFor(() => expect(readerRealtimeCallbacks).toBeTruthy());

		readerRealtimeCallbacks?.onAiFailed({
			documentId: 'doc_1',
			action: 'summary',
			aiRunId: 'airun_summary',
			message: 'provider timeout'
		});
		expect(await screen.findByText("Mila couldn't create a summary.")).toBeTruthy();

		readerRealtimeCallbacks?.onAiCompleted({ action: 'tags', aiRunId: 'airun_tags' });
		await tick();
		expect(screen.getByText("Mila couldn't create a summary.")).toBeTruthy();

		readerRealtimeCallbacks?.onAiCompleted({ action: 'summary', aiRunId: 'airun_summary_retry' });
		await waitFor(() => expect(screen.queryByText("Mila couldn't create a summary.")).toBeNull());
	});

	it('queues the failed Mila action and reports success', async () => {
		mockGetDocumentEntry.mockResolvedValue({ data: readModel() });
		render(DocumentReaderPage);
		await waitFor(() => expect(readerRealtimeCallbacks).toBeTruthy());

		readerRealtimeCallbacks?.onAiFailed({
			documentId: 'doc_1',
			action: 'summary',
			aiRunId: 'airun_1',
			message: 'provider timeout'
		});
		await fireEvent.click(await screen.findByRole('button', { name: 'Retry' }));

		await waitFor(() =>
			expect(mockRetryMilaDocumentAction).toHaveBeenCalledWith({
				path: { document_id: 'doc_1', action: 'summary' }
			})
		);
		expect(screen.getByText('Retry queued.')).toBeTruthy();
	});

	it('keeps the Mila failure actionable when retry enqueue fails', async () => {
		mockRetryMilaDocumentAction.mockResolvedValueOnce({
			error: { code: 'service_unavailable', message: 'provider unavailable' }
		});
		mockGetDocumentEntry.mockResolvedValue({ data: readModel() });
		render(DocumentReaderPage);
		await waitFor(() => expect(readerRealtimeCallbacks).toBeTruthy());

		readerRealtimeCallbacks?.onAiFailed({
			documentId: 'doc_1',
			action: 'entities',
			aiRunId: 'airun_2',
			message: 'provider unavailable'
		});
		await fireEvent.click(await screen.findByRole('button', { name: 'Retry' }));

		await waitFor(() => expect(screen.getByText('Could not queue retry. Try again.')).toBeTruthy());
		expect(screen.getByRole('button', { name: 'Retry' })).toBeTruthy();
	});
	it('shows a preparing state until the readable asset is ready', async () => {
		mockGetDocumentEntry.mockResolvedValue({ data: readModel({ readable_ready: false }) });
		mockListAssets.mockResolvedValue({ data: { data: [], page: { has_more: false } } });
		render(DocumentReaderPage);
		await waitFor(() => expect(screen.getByTestId('preparing-reader')).toBeTruthy());
	});

	it('explains a rendering outage without blaming the source and exposes diagnostics', async () => {
		const writeText = vi.fn().mockResolvedValue(undefined);
		Object.defineProperty(navigator, 'clipboard', {
			configurable: true,
			value: { writeText }
		});
		const reason = 'external service error from renderer: error sending request for url';
		mockGetDocumentEntry.mockResolvedValue({
			data: readModel({ readable_ready: false, available_assets: [] })
		});
		mockListAssets.mockResolvedValue({
			data: {
				data: assets([
					{
						id: 'asset_diagnostic',
						status: 'failed',
						failed_reason: reason,
						created_at: '2026-08-12T05:00:00Z'
					}
				]),
				page: { has_more: false }
			}
		});

		render(DocumentReaderPage);

		expect(await screen.findByText('Rendering service unavailable')).toBeTruthy();
		expect(screen.queryByText('The source may be blocking automated access.')).toBeNull();
		expect(screen.getByText('asset_diagnostic')).toBeTruthy();
		const attemptedAt = screen.getByTestId('reader-failure-attempt');
		expect(attemptedAt.getAttribute('datetime')).toBe('2026-08-12T05:00:00Z');
		const details = screen.getByText('Technical details').closest('details');
		expect(details?.open).toBe(false);
		expect(details?.textContent).toContain(reason);

		await fireEvent.click(screen.getByRole('button', { name: 'Copy diagnostic ID' }));
		expect(writeText).toHaveBeenCalledWith('asset_diagnostic');
		const copyStatus = screen.getByTestId('reader-diagnostic-copy-status');
		expect(copyStatus.textContent).toBe('Diagnostic ID copied.');
		expect(copyStatus.closest('.diagnostic-id')).toBeTruthy();
		expect(copyStatus.classList.contains('sr-only')).toBe(false);
	});

	it('copies the diagnostic ID when the Clipboard API is unavailable', async () => {
		Object.defineProperty(navigator, 'clipboard', { configurable: true, value: undefined });
		const execCommand = vi.fn().mockReturnValue(true);
		Object.defineProperty(document, 'execCommand', { configurable: true, value: execCommand });
		mockGetDocumentEntry.mockResolvedValue({
			data: readModel({ readable_ready: false, available_assets: [] })
		});
		mockListAssets.mockResolvedValue({
			data: {
				data: assets([{ id: 'asset_fallback', status: 'failed', failed_reason: 'unknown' }]),
				page: { has_more: false }
			}
		});

		render(DocumentReaderPage);
		await fireEvent.click(await screen.findByRole('button', { name: 'Copy diagnostic ID' }));

		expect(execCommand).toHaveBeenCalledWith('copy');
		expect(screen.getByTestId('reader-diagnostic-copy-status').textContent).toBe(
			'Diagnostic ID copied.'
		);
	});

	it('announces when the diagnostic ID cannot be copied', async () => {
		Object.defineProperty(navigator, 'clipboard', { configurable: true, value: undefined });
		Object.defineProperty(document, 'execCommand', {
			configurable: true,
			value: vi.fn().mockReturnValue(false)
		});
		mockGetDocumentEntry.mockResolvedValue({
			data: readModel({ readable_ready: false, available_assets: [] })
		});
		mockListAssets.mockResolvedValue({
			data: {
				data: assets([{ id: 'asset_copy_failure', status: 'failed', failed_reason: 'unknown' }]),
				page: { has_more: false }
			}
		});

		render(DocumentReaderPage);
		await fireEvent.click(await screen.findByRole('button', { name: 'Copy diagnostic ID' }));

		const copyStatus = screen.getByTestId('reader-diagnostic-copy-status');
		expect(copyStatus.textContent).toBe('Could not copy diagnostic ID.');
		expect(copyStatus.closest('.diagnostic-id')).toBeTruthy();
		expect(copyStatus.classList.contains('sr-only')).toBe(false);
	});

	it('uses neutral fallback copy for an unclassified preparation failure', async () => {
		mockGetDocumentEntry.mockResolvedValue({
			data: readModel({ readable_ready: false, available_assets: [] })
		});
		mockListAssets.mockResolvedValue({
			data: {
				data: assets([{ status: 'failed', failed_reason: 'unexpected preparation failure' }]),
				page: { has_more: false }
			}
		});

		render(DocumentReaderPage);

		expect(await screen.findByText('Readable content unavailable')).toBeTruthy();
		expect(
			screen.getByText('Preparation failed, but the cause could not be determined.')
		).toBeTruthy();
	});

	it('announces completion only after an accepted reader retry becomes ready', async () => {
		mockGetDocumentEntry
			.mockResolvedValueOnce({ data: readModel({ readable_ready: false, available_assets: [] }) })
			.mockResolvedValue({ data: readModel() });
		mockListAssets
			.mockResolvedValueOnce({
				data: {
					data: assets([
						{ status: 'failed', failed_reason: 'renderer returned no readable_html artifact' }
					]),
					page: { has_more: false }
				}
			})
			.mockResolvedValue({
				data: {
					data: assets([{ status: 'completed', failed_reason: null }]),
					page: { has_more: false }
				}
			});

		render(DocumentReaderPage);
		const outcome = await screen.findByTestId('reader-retry-outcome');
		expect(outcome.textContent).toBe('');
		await fireEvent.click(await screen.findByTestId('reader-retry'));

		await waitFor(() => expect(outcome.textContent).toBe('Readable content is ready.'));
	});

	it('does not announce a queued retry after navigating to another ready document', async () => {
		mockGetDocumentEntry.mockImplementation(({ path }: { path: { document_id: string } }) =>
			Promise.resolve({
				data:
					path.document_id === 'doc_1'
						? readModel({ readable_ready: false, available_assets: [] })
						: readModel({
								id: 'doc_2',
								document_id: 'doc_2',
								title: 'Second article'
							})
			})
		);
		mockListAssets.mockImplementation(({ path }: { path: { document_id: string } }) =>
			Promise.resolve({
				data: {
					data:
						path.document_id === 'doc_1'
							? assets([{ status: 'failed', failed_reason: 'unknown' }])
							: assets([{ status: 'completed', failed_reason: null }]),
					page: { has_more: false }
				}
			})
		);

		render(DocumentReaderPage);
		await screen.findByTestId('reader-retry-outcome');
		await fireEvent.click(await screen.findByTestId('reader-retry'));
		await waitFor(() => expect(screen.getByText('Reprocessing queued.')).toBeTruthy());

		mockRouteParams.set('documentId', 'doc_2');

		await waitFor(() =>
			expect(mockGetDocumentEntry).toHaveBeenCalledWith({ path: { document_id: 'doc_2' } })
		);
		expect(screen.getByTestId('reader-retry-outcome').textContent).toBe('');
	});

	it('discards a stale document load before it can complete the current retry', async () => {
		let resolveStaleEntry!: (value: { data: DocumentListEntry }) => void;
		let resolveStaleAssets!: (value: {
			data: { data: DocumentReaderAssetResponse[]; page: { has_more: boolean } };
		}) => void;
		let docOneEntryCalls = 0;
		let docOneAssetCalls = 0;
		mockGetDocumentEntry.mockImplementation(({ path }: { path: { document_id: string } }) => {
			if (path.document_id === 'doc_1' && ++docOneEntryCalls === 2) {
				return new Promise((resolve) => {
					resolveStaleEntry = resolve;
				});
			}
			return Promise.resolve({
				data: readModel({
					id: path.document_id,
					document_id: path.document_id,
					title: path.document_id === 'doc_2' ? 'Second article' : 'First article',
					readable_ready: false,
					available_assets: []
				})
			});
		});
		mockListAssets.mockImplementation(({ path }: { path: { document_id: string } }) => {
			if (path.document_id === 'doc_1' && ++docOneAssetCalls === 2) {
				return new Promise((resolve) => {
					resolveStaleAssets = resolve;
				});
			}
			return Promise.resolve({
				data: {
					data: assets([{ status: 'failed', failed_reason: 'unknown' }]),
					page: { has_more: false }
				}
			});
		});

		render(DocumentReaderPage);
		await waitFor(() => expect(readerRealtimeCallbacks).toBeTruthy());
		readerRealtimeCallbacks?.onAiCompleted({ action: 'summary', aiRunId: 'airun_1' });
		await waitFor(() => expect(docOneEntryCalls).toBe(2));

		mockRouteParams.set('documentId', 'doc_2');
		await waitFor(() =>
			expect(mockGetDocumentEntry).toHaveBeenCalledWith({ path: { document_id: 'doc_2' } })
		);
		expect(screen.getAllByText('Second article').length).toBeGreaterThan(0);
		await fireEvent.click(await screen.findByTestId('reader-retry'));
		await waitFor(() => expect(screen.getByText('Reprocessing queued.')).toBeTruthy());

		resolveStaleEntry({ data: readModel() });
		resolveStaleAssets({
			data: {
				data: assets([{ status: 'completed', failed_reason: null }]),
				page: { has_more: false }
			}
		});
		await new Promise((resolve) => setTimeout(resolve, 0));
		await tick();

		expect(screen.getAllByText('Second article').length).toBeGreaterThan(0);
		expect(screen.getByTestId('reader-retry-outcome').textContent).toBe('');
	});

	it('keeps a newer same-document success when an older load fails later', async () => {
		let rejectOlderLoad!: (reason: Error) => void;
		mockGetDocumentEntry
			.mockReturnValueOnce(
				new Promise((_resolve, reject) => {
					rejectOlderLoad = reject;
				})
			)
			.mockResolvedValue({ data: readModel({ title: 'Fresh article' }) });

		render(DocumentReaderPage);
		await waitFor(() => expect(readerRealtimeCallbacks).toBeTruthy());
		readerRealtimeCallbacks?.onAiCompleted({ action: 'summary', aiRunId: 'airun_newer' });
		await waitFor(() => expect(mockGetDocumentEntry).toHaveBeenCalledTimes(2));

		rejectOlderLoad(new Error('older request failed'));

		await waitFor(() => expect(screen.getAllByText('Fresh article').length).toBeGreaterThan(0));
		expect(screen.queryByText('Failed to load item.')).toBeNull();
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
