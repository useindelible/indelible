import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
	queryLibraryEntries: vi.fn(),
	deleteLibraryEntry: vi.fn(),
	triageLibraryEntry: vi.fn(),
	trashCount: vi.fn()
}));

vi.mock('$lib/api', () => mocks);
vi.mock('$lib/api/settings', () => ({
	loadPreferencesSettings: vi.fn().mockResolvedValue({ data: undefined }),
	savePreferencesSettings: vi.fn().mockResolvedValue(undefined)
}));

import { getLibrary } from './library.svelte';

function entry(id: string) {
	return {
		id,
		title: `Title ${id}`,
		item_type: 'article',
		triage_state: 'inbox',
		is_favorite: false,
		saved_at: '2026-08-01T00:00:00Z',
		created_at: '2026-08-01T00:00:00Z',
		updated_at: '2026-08-01T00:00:00Z'
	};
}

async function seededLibrary(ids: string[]) {
	mocks.queryLibraryEntries.mockResolvedValue({
		data: { data: ids.map(entry), page: { has_more: false, next_cursor: null } }
	});
	const lib = getLibrary();
	await lib.resetAndFetch();
	return lib;
}

describe('library delete action', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		mocks.trashCount.mockResolvedValue({ data: { count: 1 } });
		mocks.deleteLibraryEntry.mockResolvedValue(undefined);
	});

	it('calls the delete endpoint, not triage, and removes the item', async () => {
		const lib = await seededLibrary(['doc_1', 'doc_2']);

		await lib.deleteAction('doc_1');

		expect(mocks.deleteLibraryEntry).toHaveBeenCalledWith({
			path: { document_id: 'doc_1' }
		});
		expect(mocks.triageLibraryEntry).not.toHaveBeenCalled();
		expect(lib.items.map((i) => i.id)).toEqual(['doc_2']);
	});

	it('refreshes the sidebar trash count after a successful delete', async () => {
		const lib = await seededLibrary(['doc_1']);

		await lib.deleteAction('doc_1');

		expect(mocks.trashCount).toHaveBeenCalled();
	});

	it('restores the item when the delete call fails', async () => {
		const lib = await seededLibrary(['doc_1']);
		mocks.deleteLibraryEntry.mockRejectedValueOnce(new Error('offline'));

		await lib.deleteAction('doc_1');

		expect(lib.items.map((i) => i.id)).toEqual(['doc_1']);
	});
});
