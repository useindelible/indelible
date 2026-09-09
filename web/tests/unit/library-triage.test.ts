import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { DocumentListEntry } from '$lib/api';

import { entry } from './library-fixtures';

const api = {
	queryLibraryEntries: vi.fn(),
	triageLibraryEntry: vi.fn(),
	deleteLibraryEntry: vi.fn(),
	markDocumentUnread: vi.fn()
};
vi.mock('$lib/api', () => api);
vi.mock('$lib/stores/sidebar.svelte', () => ({
	getSidebar: () => ({ refreshTrashCount: vi.fn() })
}));

const { getLibrary } = await import('$lib/stores/library.svelte');
const { getLibrarySelection } = await import('$lib/stores/library-selection.svelte');
const lib = getLibrary();
const selection = getLibrarySelection();

async function seed(items: DocumentListEntry[]) {
	api.queryLibraryEntries.mockResolvedValue({ data: { data: items, page: { has_more: false } } });
	await lib.resetAndFetch();
}

beforeEach(() => {
	api.triageLibraryEntry.mockReset().mockResolvedValue({ data: {} });
	api.deleteLibraryEntry.mockReset().mockResolvedValue({ data: undefined });
	api.markDocumentUnread.mockReset().mockResolvedValue({ data: undefined });
});

afterEach(() => {
	lib.setDraftConditions([]);
	lib.setTriageTab('inbox');
	vi.useRealTimers();
});

describe('keyboard triage', () => {
	it('drops the row from a triage-scoped list and selects its neighbour', async () => {
		lib.setGroupBy('triage');
		await seed([entry('a'), entry('b'), entry('c')]);
		lib.setSelectedId('b');

		selection.triageSelected('archive');

		expect(lib.items.map((i) => i.id)).toEqual(['a', 'c']);
		expect(lib.selectedId).toBe('c');
		expect(api.triageLibraryEntry).toHaveBeenCalledWith({
			path: { document_id: 'b' },
			body: { state: 'archive' }
		});
	});

	it('keeps the row, with its new state, in an unscoped list', async () => {
		lib.setGroupBy('none');
		await seed([entry('a'), entry('b')]);
		lib.setSelectedId('a');

		selection.triageSelected('archive');

		expect(lib.items.map((i) => [i.id, i.triage_state])).toEqual([
			['a', 'archive'],
			['b', 'inbox']
		]);
		expect(lib.selectedId).toBe('a');
	});

	it('ignores a selection the current view hides', async () => {
		lib.setGroupBy('read_status');
		await seed([entry('unseen'), entry('seen', '2026-05-19T10:00:00Z')]);
		lib.setReadStatusTab('unseen');
		lib.setSelectedId('seen');

		expect(selection.selectedItem).toBeNull();
		selection.triageSelected('archive');

		expect(api.triageLibraryEntry).not.toHaveBeenCalled();
		expect(lib.items).toHaveLength(2);
	});

	it('leaves membership to the backend when an explicit triage condition is active', async () => {
		vi.useFakeTimers();
		lib.setGroupBy('triage');
		await seed([entry('a'), entry('b')]);
		lib.setDraftConditions([{ id: 'c1', field: 'triage_state', op: 'neq', value: 'archive' }]);
		await vi.runAllTimersAsync();
		lib.setSelectedId('a');
		api.queryLibraryEntries.mockClear();

		selection.triageSelected('later');

		expect(lib.items.map((i) => [i.id, i.triage_state])).toEqual([
			['a', 'later'],
			['b', 'inbox']
		]);
		expect(lib.selectedId).toBe('a');

		await vi.runAllTimersAsync();
		expect(api.queryLibraryEntries).toHaveBeenCalledOnce();
	});

	it('does nothing for the state the row is already in', async () => {
		lib.setGroupBy('triage');
		await seed([entry('a')]);
		lib.setSelectedId('a');

		selection.triageSelected('inbox');

		expect(api.triageLibraryEntry).not.toHaveBeenCalled();
		expect(lib.items).toHaveLength(1);
	});
});
