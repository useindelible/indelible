import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
	listTagEntries: vi.fn(),
	listCollectionEntries: vi.fn(),
	deleteLibraryEntry: vi.fn(),
	trashCount: vi.fn()
}));

vi.mock('$lib/api', () => mocks);
vi.mock('$lib/api/pagination', () => ({ fetchAllPages: vi.fn() }));

import { getTags } from './tags.svelte';
import { getCollections } from './collections.svelte';

function entry(id: string) {
	return {
		id,
		title: `Title ${id}`,
		item_type: 'article',
		triage_state: 'inbox',
		is_favorite: false,
		saved_at: '2026-08-01T00:00:00Z'
	};
}

function page(ids: string[]) {
	return { data: { data: ids.map(entry), page: { has_more: false, next_cursor: null } } };
}

beforeEach(() => {
	vi.clearAllMocks();
	mocks.trashCount.mockResolvedValue({ data: { count: 1 } });
	mocks.deleteLibraryEntry.mockResolvedValue(undefined);
});

describe('tag page delete', () => {
	it('deletes the entry and removes it from the tag item list', async () => {
		mocks.listTagEntries.mockResolvedValue(page(['doc_1', 'doc_2']));
		const store = getTags();
		await store.loadTagItems('tag_1', true);

		await store.deleteTagItem('doc_1');

		expect(mocks.deleteLibraryEntry).toHaveBeenCalledWith({
			path: { document_id: 'doc_1' }
		});
		expect(store.tagItems.map((i) => i.id)).toEqual(['doc_2']);
		expect(mocks.trashCount).toHaveBeenCalled();
	});

	it('restores the entry when the delete call fails', async () => {
		mocks.listTagEntries.mockResolvedValue(page(['doc_1']));
		const store = getTags();
		await store.loadTagItems('tag_1', true);
		mocks.deleteLibraryEntry.mockRejectedValueOnce(new Error('offline'));

		await store.deleteTagItem('doc_1');

		expect(store.tagItems.map((i) => i.id)).toEqual(['doc_1']);
	});
});

describe('collection page delete', () => {
	it('deletes the entry and removes it from the collection item list', async () => {
		mocks.listCollectionEntries.mockResolvedValue(page(['doc_3', 'doc_4']));
		const store = getCollections();
		await store.loadItems('col_1', true);

		await store.deleteItem('doc_3');

		expect(mocks.deleteLibraryEntry).toHaveBeenCalledWith({
			path: { document_id: 'doc_3' }
		});
		expect(store.items.map((i) => i.id)).toEqual(['doc_4']);
		expect(mocks.trashCount).toHaveBeenCalled();
	});

	it('restores the entry when the delete call fails', async () => {
		mocks.listCollectionEntries.mockResolvedValue(page(['doc_3']));
		const store = getCollections();
		await store.loadItems('col_1', true);
		mocks.deleteLibraryEntry.mockRejectedValueOnce(new Error('offline'));

		await store.deleteItem('doc_3');

		expect(store.items.map((i) => i.id)).toEqual(['doc_3']);
	});
});
