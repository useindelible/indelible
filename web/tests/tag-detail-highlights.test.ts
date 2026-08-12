import { fireEvent, render, screen } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const tagStore = vi.hoisted(() => ({
	currentTag: {
		id: 'tag_1',
		object: 'tag',
		name: 'Research',
		color: null,
		parent_id: null,
		item_count: 0,
		highlight_count: 1,
		aliases: [],
		created_at: '2026-08-12T00:00:00Z'
	},
	tagItems: [],
	tagHighlights: [
		{
			id: 'hl_1',
			document_id: 'doc_1',
			color: 'yellow',
			text_content: 'A linked passage',
			locator: { type: 'pdf', page: 7 },
			item_title: 'Systems Field Guide',
			item_domain: 'example.com',
			item_type: 'pdf',
			note: 'Compare this with the architecture notes.',
			created_at: '2026-08-12T00:00:00Z',
			updated_at: '2026-08-12T00:00:00Z'
		}
	],
	loading: false,
	itemsLoading: false,
	itemsLoadingMore: false,
	itemsHasMore: false,
	highlightsLoading: false,
	highlightsLoadingMore: false,
	highlightsHasMore: false,
	fetchError: null,
	loadTag: vi.fn(),
	loadTagItems: vi.fn(),
	loadTagHighlights: vi.fn(),
	updateTag: vi.fn(),
	deleteTag: vi.fn(),
	deleteTagItem: vi.fn()
}));

const originalHighlight = { ...tagStore.tagHighlights[0] };

vi.mock('$app/state', () => ({ page: { params: { id: 'tag_1' } } }));
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$app/paths', () => ({
	resolve: (path: string, params?: Record<string, string>) =>
		params ? path.replace('[documentId]', params.documentId) : path
}));
vi.mock('$lib/stores/tags.svelte', () => ({ getTags: () => tagStore }));
vi.mock('$lib/stores/library.svelte', () => ({ getLibrary: () => ({ triageMode: false }) }));
vi.mock('$lib/stores/viewport.svelte', () => ({
	getViewport: () => ({ openMobileNav: vi.fn() })
}));
vi.mock('$lib/components/library/ItemList.svelte', () => ({
	default: vi.fn(() => ({ c: vi.fn(), m: vi.fn(), p: vi.fn(), d: vi.fn() }))
}));
vi.mock('$lib/components/tags/TagColorPicker.svelte', () => ({
	default: vi.fn(() => ({ c: vi.fn(), m: vi.fn(), p: vi.fn(), d: vi.fn() }))
}));

import TagDetailPage from '../src/routes/(app)/tags/[id]/+page.svelte';

beforeEach(() => {
	vi.clearAllMocks();
	tagStore.tagHighlights.splice(0, tagStore.tagHighlights.length, { ...originalHighlight });
});

describe('tag highlight results', () => {
	it('renders each result as a keyboard link with source, note, date, and locator context', async () => {
		render(TagDetailPage);
		await fireEvent.click(screen.getByRole('button', { name: 'Highlights (1)' }));

		const link = screen.getByRole('link', { name: /Systems Field Guide/ });
		expect(link.getAttribute('href')).toBe('/(app)/reader/doc_1?highlight=hl_1');
		expect(screen.getByText('example.com · PDF')).toBeTruthy();
		expect(screen.getByText('Compare this with the architecture notes.')).toBeTruthy();
		expect(screen.getByText('Page 7')).toBeTruthy();
		expect(screen.getByText(/12 Aug 2026/)).toBeTruthy();
	});

	it('falls back to domain without an empty type separator', async () => {
		tagStore.tagHighlights[0] = {
			...originalHighlight,
			item_title: null,
			item_type: null
		};
		render(TagDetailPage);
		await fireEvent.click(screen.getByRole('button', { name: 'Highlights (1)' }));

		expect(screen.getByRole('link', { name: /example.com/ })).toBeTruthy();
		expect(screen.getByText('example.com').textContent).toBe('example.com');
	});

	it('falls back to type and identifies the EPUB chapter', async () => {
		tagStore.tagHighlights[0] = {
			...originalHighlight,
			item_title: null,
			item_domain: null,
			item_type: 'epub',
			locator: { type: 'epub', chapter: 'signal', start_offset: 10, end_offset: 20 }
		};
		render(TagDetailPage);
		await fireEvent.click(screen.getByRole('button', { name: 'Highlights (1)' }));

		expect(screen.getByRole('link', { name: /EPUB/ })).toBeTruthy();
		expect(screen.getByText('Chapter signal')).toBeTruthy();
	});
});
