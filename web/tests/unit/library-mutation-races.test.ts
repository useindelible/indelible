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

const READ = '2026-05-19T10:00:00Z';

describe('interleaved triage and mark unread on one row', () => {
	it('keeps a triage change that committed while the unread request was pending', async () => {
		const unread = deferred<{ data: undefined }>();
		api.markDocumentUnread.mockReturnValue(unread.promise);
		lib.setGroupBy('none');
		await seed([entry('a', READ)]);
		lib.setSelectedId('a');

		selection.markSelectedUnread();
		selection.triageSelected('archive');
		await vi.waitFor(() => expect(api.triageLibraryEntry).toHaveBeenCalledOnce());
		unread.reject(new Error('offline'));

		await vi.waitFor(() => expect(lib.items[0]?.last_read_at).toBe(READ));
		expect(lib.items[0]?.triage_state).toBe('archive');
	});

	it('keeps the unread state when a later triage request fails', async () => {
		const triage = deferred<{ data: object }>();
		api.triageLibraryEntry.mockReturnValue(triage.promise);
		lib.setGroupBy('none');
		await seed([entry('a', READ)]);
		lib.setSelectedId('a');

		selection.triageSelected('archive');
		selection.markSelectedUnread();
		await vi.waitFor(() => expect(api.markDocumentUnread).toHaveBeenCalledOnce());
		triage.reject(new Error('offline'));

		await vi.waitFor(() => expect(lib.items[0]?.triage_state).toBe('inbox'));
		expect(lib.items[0]?.last_read_at).toBeNull();
	});

	it('reinserts a row as unread when triage removed it and then failed', async () => {
		const triage = deferred<{ data: object }>();
		api.triageLibraryEntry.mockReturnValue(triage.promise);
		lib.setGroupBy('triage');
		await seed([entry('a', READ)]);
		lib.setSelectedId('a');

		selection.markSelectedUnread();
		selection.triageSelected('archive');
		await vi.waitFor(() => expect(api.triageLibraryEntry).toHaveBeenCalledOnce());
		expect(lib.items).toHaveLength(0);
		triage.reject(new Error('offline'));

		await vi.waitFor(() => expect(lib.items).toHaveLength(1));
		expect(lib.items[0]?.last_read_at).toBeNull();
	});

	it('reinserts a row as read when both its unread and its triage request failed', async () => {
		const unread = deferred<{ data: undefined }>();
		const triage = deferred<{ data: object }>();
		api.markDocumentUnread.mockReturnValue(unread.promise);
		api.triageLibraryEntry.mockReturnValue(triage.promise);
		lib.setGroupBy('triage');
		await seed([entry('a', READ)]);
		lib.setSelectedId('a');

		selection.markSelectedUnread();
		selection.triageSelected('archive');
		expect(lib.items).toHaveLength(0);
		unread.reject(new Error('offline'));
		await new Promise((resolve) => setTimeout(resolve, 0));
		triage.reject(new Error('offline'));

		await vi.waitFor(() => expect(lib.items).toHaveLength(1));
		expect(lib.items[0]?.last_read_at).toBe(READ);
		expect(lib.items[0]?.max_progress_percent).toBe(60);
	});

	it('drops an unread rollback that lands after the removal already succeeded', async () => {
		const LATER = '2026-06-01T10:00:00Z';
		const unread = deferred<{ data: undefined }>();
		api.markDocumentUnread.mockReturnValue(unread.promise);
		lib.setGroupBy('triage');
		await seed([entry('a', READ)]);
		lib.setSelectedId('a');

		selection.markSelectedUnread();
		selection.triageSelected('archive');
		await new Promise((resolve) => setTimeout(resolve, 0));
		unread.reject(new Error('offline'));
		await new Promise((resolve) => setTimeout(resolve, 0));

		await seed([entry('a', LATER)]);
		api.triageLibraryEntry.mockRejectedValue(new Error('offline'));
		lib.setSelectedId('a');
		selection.triageSelected('archive');

		await vi.waitFor(() => expect(lib.items).toHaveLength(1));
		expect(lib.items[0]?.last_read_at).toBe(LATER);
	});

	it('keeps a deleted row hidden when an earlier unread refetch brought it back', async () => {
		vi.useFakeTimers();
		const deletion = deferred<{ data: undefined }>();
		api.deleteLibraryEntry.mockReturnValue(deletion.promise);
		lib.setGroupBy('none');
		await seed([entry('a', READ), entry('b')]);
		lib.setDraftConditions([{ id: 'c1', field: 'triage_state', op: 'neq', value: 'archive' }]);
		await vi.runAllTimersAsync();
		lib.setSelectedId('a');

		selection.markSelectedUnread();
		void lib.deleteAction('a');
		await vi.runAllTimersAsync();
		expect(lib.items.map((i) => i.id)).toEqual(['b']);

		api.queryLibraryEntries.mockResolvedValue({
			data: { data: [entry('b')], page: { has_more: false } }
		});
		deletion.resolve({ data: undefined });
		await vi.runAllTimersAsync();

		expect(lib.items.map((i) => i.id)).toEqual(['b']);
	});

	it('shows a row once when a refetch brought it back and its deletion then failed', async () => {
		vi.useFakeTimers();
		const deletion = deferred<{ data: undefined }>();
		api.deleteLibraryEntry.mockReturnValue(deletion.promise);
		lib.setGroupBy('none');
		await seed([entry('a', READ), entry('b')]);
		lib.setDraftConditions([{ id: 'c1', field: 'triage_state', op: 'neq', value: 'archive' }]);
		await vi.runAllTimersAsync();
		lib.setSelectedId('a');

		selection.markSelectedUnread();
		void lib.deleteAction('a');
		await vi.runAllTimersAsync();
		deletion.reject(new Error('offline'));
		await vi.runAllTimersAsync();

		expect(lib.items.map((i) => i.id)).toEqual(['a', 'b']);
	});

	it('keeps an archived row in the archive tab the user opened before the request settled', async () => {
		const triage = deferred<{ data: object }>();
		api.triageLibraryEntry.mockReturnValue(triage.promise);
		lib.setGroupBy('triage');
		await seed([entry('a'), entry('b')]);
		lib.setSelectedId('a');

		selection.triageSelected('archive');
		expect(lib.items.map((i) => i.id)).toEqual(['b']);

		const archived = { ...entry('a'), triage_state: 'archive' as const };
		api.queryLibraryEntries.mockResolvedValue({
			data: { data: [archived], page: { has_more: false } }
		});
		lib.setTriageTab('archive');
		await vi.waitFor(() => expect(lib.loading).toBe(false));
		expect(lib.items).toEqual([]);

		triage.resolve({ data: {} });
		await new Promise((resolve) => setTimeout(resolve, 0));

		expect(lib.items.map((i) => i.id)).toEqual(['a']);
	});

	it('keeps the later triage state when an earlier triage on the same row fails afterwards', async () => {
		const first = deferred<{ data: object }>();
		api.triageLibraryEntry.mockReturnValueOnce(first.promise).mockResolvedValue({ data: {} });
		lib.setGroupBy('none');
		await seed([entry('a')]);
		lib.setSelectedId('a');

		selection.triageSelected('archive');
		selection.triageSelected('later');
		await new Promise((resolve) => setTimeout(resolve, 0));
		expect(lib.items[0]?.triage_state).toBe('later');

		first.reject(new Error('offline'));
		await new Promise((resolve) => setTimeout(resolve, 0));

		expect(lib.items[0]?.triage_state).toBe('later');
	});

	it('never selects a row a refetch brought back while its deletion is pending', async () => {
		vi.useFakeTimers();
		const deletion = deferred<{ data: undefined }>();
		api.deleteLibraryEntry.mockReturnValue(deletion.promise);
		lib.setGroupBy('none');
		await seed([entry('a', READ), entry('b')]);
		lib.setDraftConditions([{ id: 'c1', field: 'triage_state', op: 'neq', value: 'archive' }]);
		await vi.runAllTimersAsync();
		lib.setSelectedId('a');

		selection.markSelectedUnread();
		void lib.deleteAction('a');
		await vi.runAllTimersAsync();
		expect(lib.selectedId).toBe('b');
		expect(lib.selectedItem?.id).toBe('b');

		api.queryLibraryEntries.mockResolvedValue({
			data: { data: [entry('b')], page: { has_more: false } }
		});
		deletion.resolve({ data: undefined });
		await vi.runAllTimersAsync();
		expect(lib.selectedId).toBe('b');
	});

	it('leaves a row with a pending removal out of a bulk archive', async () => {
		const deletion = deferred<{ data: undefined }>();
		api.deleteLibraryEntry.mockReturnValue(deletion.promise);
		lib.setGroupBy('none');
		await seed([entry('a'), entry('b')]);

		void lib.deleteAction('a');
		await lib.archiveAll();

		expect(api.triageLibraryEntry).toHaveBeenCalledOnce();
		expect(api.triageLibraryEntry).toHaveBeenCalledWith({
			path: { document_id: 'b' },
			body: { state: 'archive' }
		});
		deletion.resolve({ data: undefined });
	});

	it('re-sorts by reading progress as soon as a row is marked unread', async () => {
		lib.setGroupBy('none');
		lib.setSortOrder('reading_progress');
		await seed([entry('a', READ), entry('b', READ)]);
		lib.setSelectedId('a');
		expect(lib.items.map((i) => i.id)).toEqual(['a', 'b']);

		selection.markSelectedUnread();

		expect(lib.items.map((i) => i.id)).toEqual(['b', 'a']);
		lib.setSortOrder('date_saved_desc');
	});

	it('shows a row again when its deletion fails after a bulk archive ran around it', async () => {
		const deletion = deferred<{ data: undefined }>();
		api.deleteLibraryEntry.mockReturnValue(deletion.promise);
		lib.setGroupBy('none');
		await seed([entry('a'), entry('b')]);

		void lib.deleteAction('a');
		await lib.archiveAll();
		expect(lib.items).toEqual([]);

		deletion.reject(new Error('offline'));
		await new Promise((resolve) => setTimeout(resolve, 0));

		expect(lib.items.map((i) => i.id)).toEqual(['a']);
	});
});
