import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const mockQueryLibraryEntries = vi.fn();
const mockGetSmartList = vi.fn();
const mockListSmartLists = vi.fn();
const mockLoadPreferencesSettings = vi.fn();
const mockSavePreferencesSettings = vi.fn();

class MockIntersectionObserver {
	observe = vi.fn();
	disconnect = vi.fn();
	unobserve = vi.fn();
}

vi.mock('$lib/api', () => ({
	queryLibraryEntries: (...args: unknown[]) => mockQueryLibraryEntries(...args),
	getSmartList: (...args: unknown[]) => mockGetSmartList(...args),
	listSmartLists: (...args: unknown[]) => mockListSmartLists(...args)
}));
vi.mock('$lib/api/settings', () => ({
	loadPreferencesSettings: (...args: unknown[]) => mockLoadPreferencesSettings(...args),
	savePreferencesSettings: (...args: unknown[]) => mockSavePreferencesSettings(...args)
}));
vi.mock('$app/navigation', () => ({
	goto: vi.fn(),
	afterNavigate: () => {},
	beforeNavigate: () => {}
}));
vi.mock('$app/paths', () => ({ resolve: (path: string) => path }));
vi.mock('$app/state', () => ({
	page: { url: new URL('http://localhost/library'), params: {} }
}));

vi.mock('$lib/components/library/DetailPanel.svelte', () => ({
	default: vi.fn(() => ({ c: vi.fn(), m: vi.fn(), p: vi.fn(), d: vi.fn() }))
}));

import LibraryPage from '../../src/routes/(app)/library/[[type]]/+page.svelte';
/* eslint-disable-next-line svelte-runes/no-external-svelte-imports -- test drives the store the rendered page reads from */
import { getLibrary } from '../../src/lib/stores/library.svelte';

type Expr = {
	type?: string;
	field?: string;
	conditions?: Expr[];
};

function lastFilterExpression(): Expr | null {
	const calls = mockQueryLibraryEntries.mock.calls;
	if (calls.length === 0) return null;
	const last = calls[calls.length - 1][0] as { body: { filter_expression: Expr | null } };
	return last.body.filter_expression;
}

// The page ANDs the user's own rules with its triage/type scope, so the connector
// under test sits on the branch holding those rules, not at the top level.
function draftBranch(): Expr | null {
	const walk = (node: Expr | null | undefined): Expr | null => {
		if (!node?.conditions) return null;
		if (node.conditions.some((child) => child.type === 'condition' && child.field === 'tag')) {
			return node;
		}
		for (const child of node.conditions) {
			const found = walk(child);
			if (found) return found;
		}
		return null;
	};
	return walk(lastFilterExpression());
}

describe('Library filter refetch', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.stubGlobal('IntersectionObserver', MockIntersectionObserver);
		mockQueryLibraryEntries.mockResolvedValue({
			data: { data: [], page: { has_more: false, next_cursor: null } }
		});
		mockListSmartLists.mockResolvedValue({ data: { data: [] } });
		mockLoadPreferencesSettings.mockResolvedValue({});
		mockSavePreferencesSettings.mockResolvedValue(undefined);
	});

	it('refetches when the connector switches from AND to OR', async () => {
		render(LibraryPage);
		const lib = getLibrary();

		lib.setDraftConditions([
			{ id: 'c1', field: 'tag', op: 'contains', value: 'orion' },
			{ id: 'c2', field: 'tag', op: 'contains', value: 'lyra' }
		]);
		if (!lib.filterBarOpen) lib.toggleFilterBar();

		// Preference loading and the 600ms filter debounce both settle into their own
		// fetches. Let every one of them land before clearing, so the only call that can
		// follow is the one the connector change causes.
		await waitFor(() => expect(draftBranch()?.type).toBe('and'), { timeout: 2000 });
		await new Promise((resolve) => setTimeout(resolve, 1200));
		mockQueryLibraryEntries.mockClear();

		const toggle = await screen.findByRole('button', { name: 'And' });
		await fireEvent.click(toggle);

		await new Promise((resolve) => setTimeout(resolve, 1200));

		expect(mockQueryLibraryEntries).toHaveBeenCalled();
		expect(draftBranch()?.type).toBe('or');
	});

	it('marks a saved view as modified only once its rules are edited', async () => {
		mockGetSmartList.mockResolvedValue({
			data: {
				id: 'sl1',
				name: 'Alpha',
				filter_expression: {
					type: 'and',
					conditions: [{ type: 'condition', field: 'tag', op: 'contains', value: 'alpha' }]
				}
			}
		});

		render(LibraryPage);
		const lib = getLibrary();
		// Entering the view seeds the filter bar from its saved rules.
		await lib.setSmartList('sl1');

		expect(screen.queryByText('Modified')).toBeNull();

		lib.setDraftConditions([{ id: 'c9', field: 'tag', op: 'contains', value: 'beta' }]);

		await waitFor(() => expect(screen.queryByText('Modified')).not.toBeNull(), { timeout: 2000 });
	});
});
