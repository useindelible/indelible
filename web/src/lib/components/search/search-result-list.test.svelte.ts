import { render } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import type { SearchResultResponse } from '$lib/api/generated/types.gen';
import { rowKey } from '$lib/stores/search-entry';
import SearchResultList from './SearchResultList.svelte';

function chapterHit(section: string): SearchResultResponse {
	return {
		content_type: 'book',
		document_id: 'doc_1',
		result_kind: 'document',
		saved_at: '2026-08-19T10:00:00Z',
		score: 1,
		snippet: `snippet ${section}`,
		title: 'A book',
		updated_at: '2026-08-19T11:00:00Z',
		url: 'https://example.com/book',
		section: { key: section, kind: 'chapter' }
	};
}

describe('SearchResultList', () => {
	it('marks only the chosen row when one item matches in two sections', () => {
		const results = [chapterHit('c1'), chapterHit('c2')];
		const { container } = render(SearchResultList, {
			props: {
				results,
				loading: false,
				loadingMore: false,
				hasMore: false,
				isEmpty: false,
				selectedId: rowKey(results[1]!),
				query: 'q',
				onLoadMore: vi.fn(),
				onSelect: vi.fn(),
				onOpen: vi.fn()
			}
		});

		const rows = [...container.querySelectorAll('[aria-selected]')];
		expect(rows.map((row) => row.getAttribute('aria-selected'))).toEqual(['false', 'true']);
	});
});
