import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const mockSearch = vi.fn();
const mockSuggestions = vi.fn();
const mockListRecentSearches = vi.fn();
const mockClearRecentSearches = vi.fn();
const mockDeleteRecentSearch = vi.fn();
const mockGoto = vi.fn();

class MockIntersectionObserver {
	observe = vi.fn();
	disconnect = vi.fn();
}

vi.mock('$lib/api', () => ({
	search: (...args: unknown[]) => mockSearch(...args),
	suggestions: (...args: unknown[]) => mockSuggestions(...args),
	listRecentSearches: (...args: unknown[]) => mockListRecentSearches(...args),
	clearRecentSearches: (...args: unknown[]) => mockClearRecentSearches(...args),
	deleteRecentSearch: (...args: unknown[]) => mockDeleteRecentSearch(...args)
}));
vi.mock('$app/navigation', () => ({
	goto: (...args: unknown[]) => mockGoto(...args),
	afterNavigate: () => {},
	beforeNavigate: () => {}
}));
vi.mock('$app/paths', () => ({ resolve: (path: string) => path }));
vi.mock('$app/state', () => ({
	page: { url: new URL('http://localhost/search') }
}));

vi.mock('$lib/components/library/DetailPanel.svelte', () => ({
	default: vi.fn(() => ({
		c: vi.fn(),
		m: vi.fn(),
		p: vi.fn(),
		d: vi.fn()
	}))
}));

import SearchPage from '../../src/routes/(app)/search/+page.svelte';

async function typeQuery(text: string): Promise<HTMLInputElement> {
	const input = screen.getByLabelText('Search your library') as HTMLInputElement;
	await fireEvent.focus(input);
	await fireEvent.input(input, { target: { value: text } });
	return input;
}

describe('Search page submission', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.stubGlobal('IntersectionObserver', MockIntersectionObserver);
		mockSearch.mockResolvedValue({
			data: { results: [], has_more: false, next_cursor: null }
		});
		mockSuggestions.mockResolvedValue({
			data: {
				suggestions: [{ insert_text: 'orion', kind: 'term', label: 'orion' }]
			}
		});
		mockListRecentSearches.mockResolvedValue({ data: { data: [] } });
	});

	it('submits on Enter while suggestions are open and nothing is highlighted', async () => {
		render(SearchPage);
		const input = await typeQuery('Orion');

		// The 300ms-debounced suggestion fetch opens the dropdown — the state
		// in which submission was reported broken.
		const suggestion = await screen.findByText('orion', {}, { timeout: 2000 });
		expect(suggestion).toBeTruthy();

		await fireEvent.keyDown(input, { key: 'Enter' });

		await waitFor(() => expect(mockSearch).toHaveBeenCalled());
		const call = mockSearch.mock.calls[0][0] as { query: { q: string } };
		expect(call.query.q).toBe('Orion');
		expect(mockGoto).toHaveBeenCalled();
		expect(screen.queryByText('orion')).toBeNull();
	});

	it('still applies a highlighted suggestion on Enter', async () => {
		render(SearchPage);
		const input = await typeQuery('ori');

		await screen.findByText('orion', {}, { timeout: 2000 });
		await fireEvent.keyDown(input, { key: 'ArrowDown' });
		await fireEvent.keyDown(input, { key: 'Enter' });

		await waitFor(() => expect(mockSearch).toHaveBeenCalled());
		const call = mockSearch.mock.calls[0][0] as { query: { q: string } };
		expect(call.query.q).toBe('orion');
	});

	it('offers a visible search button that submits the query', async () => {
		render(SearchPage);
		await typeQuery('Orion');

		const button = screen.getByRole('button', { name: 'Search' });
		await fireEvent.click(button);

		await waitFor(() => expect(mockSearch).toHaveBeenCalled());
		const call = mockSearch.mock.calls[0][0] as { query: { q: string } };
		expect(call.query.q).toBe('Orion');
	});
});
