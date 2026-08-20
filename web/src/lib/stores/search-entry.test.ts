import { describe, expect, it } from 'vitest';
import type { SearchResultResponse } from '$lib/api/generated/types.gen';
import { getDomain, toPlaceholderEntry } from './search-entry';

function hit(overrides: Partial<SearchResultResponse> = {}): SearchResultResponse {
	return {
		content_type: 'article',
		document_id: 'doc_1',
		result_kind: 'document',
		saved_at: '2026-08-19T10:00:00Z',
		score: 1,
		snippet: 'The <mark>Beatles</mark> for Sale',
		title: 'Beatles for Sale',
		updated_at: '2026-08-19T11:00:00Z',
		url: 'https://en.wikipedia.org/wiki/Beatles_for_Sale',
		...overrides
	};
}

describe('getDomain', () => {
	it('extracts bare domains safely', () => {
		expect(getDomain('https://www.example.com/post')).toBe('example.com');
		expect(getDomain('not a url')).toBe('');
	});
});

describe('toPlaceholderEntry', () => {
	it('carries over only what the search hit actually knows', () => {
		const entry = toPlaceholderEntry(hit());

		expect(entry.id).toBe('doc_1');
		expect(entry.document_id).toBe('doc_1');
		expect(entry.title).toBe('Beatles for Sale');
		expect(entry.item_type).toBe('article');
		expect(entry.url).toBe('https://en.wikipedia.org/wiki/Beatles_for_Sale');
		expect(entry.domain).toBe('en.wikipedia.org');
		expect(entry.saved_at).toBe('2026-08-19T10:00:00Z');
		expect(entry.excerpt).toBe('The Beatles for Sale');
	});

	it('leaves unknown metadata empty instead of inventing it', () => {
		const entry = toPlaceholderEntry(hit());

		expect(entry.published_at).toBeNull();
		expect(entry.word_count).toBeNull();
		expect(entry.reading_time_minutes).toBeNull();
		expect(entry.language).toBeNull();
		expect(entry.author).toBeNull();
		expect(entry.summary).toBeNull();
		expect(entry.progress_percent).toBeNull();
		expect(entry.last_read_at).toBeNull();
		expect(entry.library_entry_id).toBeNull();
	});

	it('keys feed previews by delivery id', () => {
		const entry = toPlaceholderEntry(
			hit({ document_id: null, delivery_id: 'dlv_9', result_kind: 'feed_preview', url: null })
		);

		expect(entry.id).toBe('dlv_9');
		expect(entry.document_id).toBe('dlv_9');
		expect(entry.domain).toBeNull();
		expect(entry.url).toBeNull();
	});
});
