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

function deferred<T>() {
	let resolve!: (value: T) => void;
	let reject!: (reason: unknown) => void;
	const promise = new Promise<T>((res, rej) => {
		resolve = res;
		reject = rej;
	});
	return { promise, resolve, reject };
}

describe('keyboard mark unread', () => {
	const READ = '2026-05-19T10:00:00Z';

	it('drops the row from the seen tab and selects its neighbour', async () => {
		lib.setGroupBy('read_status');
		await seed([entry('a', READ), entry('b', READ), entry('c')]);
		lib.setReadStatusTab('seen');
		lib.setSelectedId('a');

		selection.markSelectedUnread();

		expect(selection.displayItems.map((i) => i.id)).toEqual(['b']);
		expect(lib.items.find((i) => i.id === 'a')).toMatchObject({
			last_read_at: null,
			progress_percent: null,
			max_progress_percent: null
		});
		expect(lib.selectedId).toBe('b');
		expect(api.markDocumentUnread).toHaveBeenCalledWith({ path: { document_id: 'a' } });
	});

	it('keeps the row, now unread, in a triage list', async () => {
		lib.setGroupBy('triage');
		await seed([entry('a', READ), entry('b')]);
		lib.setSelectedId('a');

		selection.markSelectedUnread();

		expect(lib.items.map((i) => [i.id, i.last_read_at])).toEqual([
			['a', null],
			['b', null]
		]);
		expect(lib.selectedId).toBe('a');
	});

	it('still asks the backend when the list cannot tell the row is read', async () => {
		lib.setGroupBy('triage');
		await seed([entry('a')]);
		lib.setSelectedId('a');

		selection.markSelectedUnread();

		expect(api.markDocumentUnread).toHaveBeenCalledWith({ path: { document_id: 'a' } });
		expect(lib.selectedId).toBe('a');
	});

	it('restores the row and the selection when the request fails', async () => {
		api.markDocumentUnread.mockRejectedValue(new Error('offline'));
		lib.setGroupBy('read_status');
		await seed([entry('a', READ)]);
		lib.setReadStatusTab('seen');
		lib.setSelectedId('a');

		selection.markSelectedUnread();
		expect(lib.selectedId).toBeNull();

		await vi.waitFor(() => expect(lib.items[0]?.last_read_at).toBe(READ));
		expect(lib.items[0]?.max_progress_percent).toBe(60);
		expect(lib.selectedId).toBe('a');
	});

	it('sends one request per row while the first is still in flight', async () => {
		let release!: () => void;
		api.markDocumentUnread.mockReturnValue(
			new Promise((resolve) => {
				release = () => resolve({ data: undefined });
			})
		);
		lib.setGroupBy('triage');
		await seed([entry('a', READ)]);
		lib.setSelectedId('a');

		selection.markSelectedUnread();
		selection.markSelectedUnread();
		expect(api.markDocumentUnread).toHaveBeenCalledOnce();

		release();
		await vi.waitFor(() => expect(lib.items[0]?.last_read_at).toBeNull());
	});

	it('leaves the selection alone when the failed row is hidden by the tab opened meanwhile', async () => {
		const unread = deferred<{ data: undefined }>();
		api.markDocumentUnread.mockReturnValue(unread.promise);
		lib.setGroupBy('read_status');
		await seed([entry('a', READ), entry('b')]);
		lib.setReadStatusTab('seen');
		lib.setSelectedId('a');

		selection.markSelectedUnread();
		lib.setReadStatusTab('unseen');
		lib.setSelectedId('b');
		lib.setSelectedId(null);
		unread.reject(new Error('offline'));
		await vi.waitFor(() => expect(lib.items[0]?.last_read_at).toBe(READ));

		expect(selection.displayItems.map((i) => i.id)).toEqual(['b']);
		expect(lib.selectedId).toBeNull();
	});

	it('does not steal a selection the user made after the optimistic one, even the same row', async () => {
		const unread = deferred<{ data: undefined }>();
		api.markDocumentUnread.mockReturnValue(unread.promise);
		lib.setGroupBy('read_status');
		await seed([entry('a', READ), entry('b', READ), entry('c', READ)]);
		lib.setReadStatusTab('seen');
		lib.setSelectedId('a');

		selection.markSelectedUnread();
		expect(lib.selectedId).toBe('b');
		lib.setSelectedId('c');
		lib.setSelectedId('b');
		unread.reject(new Error('offline'));
		await vi.waitFor(() => expect(lib.items[0]?.last_read_at).toBe(READ));

		expect(lib.selectedId).toBe('b');
	});
});
