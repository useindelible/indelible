import { render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import type { DocumentListEntry } from '$lib/api';
import DetailInfo from '$lib/components/library/DetailInfo.svelte';
import BookInfoPanel from '$lib/components/reader/book/BookInfoPanel.svelte';

vi.mock('$app/paths', () => ({
	resolve: (path: string) => path
}));

vi.mock('$lib/api', () => ({
	listDocumentEntities: vi.fn(async () => ({ data: [] }))
}));

function item(overrides: Partial<DocumentListEntry> = {}): DocumentListEntry {
	return {
		created_at: '2026-08-12T00:00:00Z',
		document_id: 'doc_1',
		document_type: 'article',
		id: 'doc_1',
		is_favorite: false,
		is_shortlisted: false,
		item_type: 'article',
		object: 'library_entry',
		saved_at: '2026-08-12T00:00:00Z',
		source: 'library',
		title: 'Example entry',
		triage_state: 'inbox',
		updated_at: '2026-08-12T00:00:00Z',
		...overrides
	} as DocumentListEntry;
}

describe('summary presentation', () => {
	it('labels a resolved excerpt fallback without Mila attribution', () => {
		render(DetailInfo, {
			props: {
				item: item({ summary: 'Source excerpt', excerpt: 'Source excerpt' }),
				onEditMetadata: vi.fn()
			}
		});

		expect(screen.getByText('Excerpt')).toBeTruthy();
		expect(screen.getByText('Source excerpt')).toBeTruthy();
		expect(screen.queryByText('Summarized by Mila')).toBeNull();
	});

	it('attributes a distinct summary to Mila', () => {
		render(DetailInfo, {
			props: {
				item: item({ summary: 'Mila summary', excerpt: 'Source excerpt' }),
				onEditMetadata: vi.fn()
			}
		});

		expect(screen.getByText('Summary')).toBeTruthy();
		expect(screen.getByText('Mila summary')).toBeTruthy();
		expect(screen.getByText('Summarized by Mila')).toBeTruthy();
	});

	it('normalizes whitespace before choosing the excerpt fallback', () => {
		render(DetailInfo, {
			props: {
				item: item({ summary: '   ', excerpt: '  Source excerpt  ' }),
				onEditMetadata: vi.fn()
			}
		});

		expect(screen.getByText('Excerpt')).toBeTruthy();
		expect(screen.getByText('Source excerpt')).toBeTruthy();
		expect(screen.queryByText('Summarized by Mila')).toBeNull();
	});

	it('renders an unattributed unavailable state when both values are blank', () => {
		render(DetailInfo, {
			props: {
				item: item({ summary: ' ', excerpt: '\n' }),
				onEditMetadata: vi.fn()
			}
		});

		expect(screen.getByText('Summary unavailable.')).toBeTruthy();
		expect(screen.queryByText('Summarized by Mila')).toBeNull();
	});

	it('uses the same excerpt presentation in the book info panel', () => {
		render(BookInfoPanel, {
			props: {
				item: item({
					document_type: 'book',
					item_type: 'book',
					summary: 'Book excerpt',
					excerpt: 'Book excerpt'
				}),
				bookMetadata: { totalChapters: 1 },
				progress: 0
			}
		});

		expect(screen.getByText('Excerpt')).toBeTruthy();
		expect(screen.getByText('Book excerpt')).toBeTruthy();
		expect(screen.queryByText('Summarized by Mila')).toBeNull();
	});
});
