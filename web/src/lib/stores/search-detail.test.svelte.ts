import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { SearchResultResponse } from '$lib/api/generated/types.gen';

const mocks = vi.hoisted(() => ({
	search: vi.fn(),
	getDocumentEntry: vi.fn()
}));

vi.mock('$lib/api', () => mocks);

import { getSearch } from './search.svelte';

function hit(id: string, overrides: Partial<SearchResultResponse> = {}): SearchResultResponse {
	return {
		content_type: 'article',
		document_id: id,
		result_kind: 'document',
		saved_at: '2026-08-19T10:00:00Z',
		score: 1,
		snippet: `snippet ${id}`,
		title: `Title ${id}`,
		updated_at: '2026-08-19T11:00:00Z',
		url: `https://example.com/${id}`,
		...overrides
	};
}

function loadedEntry(id: string) {
	return {
		data: {
			id,
			document_id: id,
			document_type: 'article',
			item_type: 'article',
			library_entry_id: `lib_${id}`,
			title: `Title ${id}`,
			object: 'library_entry',
			url: `https://example.com/${id}`,
			canonical_url: null,
			domain: 'example.com',
			saved_at: '2026-08-19T10:00:00Z',
			updated_at: '2026-08-19T11:00:00Z',
			created_at: '2026-08-19T10:00:00Z',
			excerpt: 'full excerpt',
			triage_state: 'later',
			is_favorite: false,
			is_shortlisted: false,
			source: 'manual',
			word_count: 7899,
			reading_time_minutes: 34,
			published_at: '2002-08-22T00:00:00Z'
		},
		status: 200
	};
}

async function searched(...results: SearchResultResponse[]) {
	mocks.search.mockResolvedValue({ data: { results, has_more: false, query: 'q' } });
	const search = getSearch();
	await search.submitSearch('q');
	return search;
}

describe('search detail entry loading', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.useFakeTimers();
		mocks.getDocumentEntry.mockImplementation(({ path }: { path: { document_id: string } }) =>
			Promise.resolve(loadedEntry(path.document_id))
		);
	});

	afterEach(() => {
		getSearch().clearSearch();
		vi.useRealTimers();
	});

	it('loads the auto-selected first result as soon as results arrive', async () => {
		const search = await searched(hit('doc_1'), hit('doc_2'));
		await vi.runAllTimersAsync();

		expect(mocks.getDocumentEntry).toHaveBeenCalledTimes(1);
		expect(mocks.getDocumentEntry).toHaveBeenCalledWith({ path: { document_id: 'doc_1' } });
		expect(search.selectedEntry?.word_count).toBe(7899);
	});

	it('shows a placeholder built from the hit until the entry loads', async () => {
		let release!: () => void;
		mocks.getDocumentEntry.mockImplementation(
			() =>
				new Promise((resolve) => {
					release = () => resolve(loadedEntry('doc_1'));
				})
		);
		const search = await searched(hit('doc_1'));
		await vi.advanceTimersByTimeAsync(0);

		expect(search.selectedEntry?.title).toBe('Title doc_1');
		expect(search.selectedEntry?.word_count).toBeNull();

		release();
		await vi.runAllTimersAsync();
		expect(search.selectedEntry?.word_count).toBe(7899);
	});

	it('fetches once per document and serves repeat selections from cache', async () => {
		const search = await searched(hit('doc_1'), hit('doc_2'));
		await vi.runAllTimersAsync();

		search.setSelectedId('doc_2');
		await vi.runAllTimersAsync();
		search.setSelectedId('doc_1');
		await vi.runAllTimersAsync();

		expect(mocks.getDocumentEntry).toHaveBeenCalledTimes(2);
		expect(search.selectedEntry?.id).toBe('doc_1');
	});

	it('only loads the row the selection settles on during a quick sweep', async () => {
		const search = await searched(hit('doc_1'), hit('doc_2'), hit('doc_3'), hit('doc_4'));
		await vi.runAllTimersAsync();
		mocks.getDocumentEntry.mockClear();

		search.setSelectedId('doc_2');
		search.setSelectedId('doc_3');
		search.setSelectedId('doc_4');
		await vi.runAllTimersAsync();

		expect(mocks.getDocumentEntry).toHaveBeenCalledTimes(1);
		expect(mocks.getDocumentEntry).toHaveBeenCalledWith({ path: { document_id: 'doc_4' } });
	});

	it('never fetches for feed previews, which have no library entry', async () => {
		const search = await searched(
			hit('dlv_1', { document_id: null, delivery_id: 'dlv_1', result_kind: 'feed_preview' })
		);
		await vi.runAllTimersAsync();

		expect(mocks.getDocumentEntry).not.toHaveBeenCalled();
		expect(search.selectedEntry?.id).toBe('dlv_1');
	});

	it('refetches after a new search so stale entries are not reused', async () => {
		await searched(hit('doc_1'));
		await vi.runAllTimersAsync();
		await searched(hit('doc_1'));
		await vi.runAllTimersAsync();

		expect(mocks.getDocumentEntry).toHaveBeenCalledTimes(2);
	});

	it('keeps the placeholder when the entry fetch fails', async () => {
		mocks.getDocumentEntry.mockRejectedValue(new Error('network'));
		const search = await searched(hit('doc_1'));
		await vi.runAllTimersAsync();

		expect(search.selectedEntry?.title).toBe('Title doc_1');
	});
});
