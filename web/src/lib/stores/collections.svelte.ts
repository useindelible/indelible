import * as api from '$lib/api';
import type { CollectionResponse, DocumentListEntry } from '$lib/api';
import { fetchAllPages } from '$lib/api/pagination';

export type CollectionSort = 'name_asc' | 'name_desc' | 'item_count' | 'date_created';

const PAGE_LIMIT = 50;

let allCollections = $state<CollectionResponse[]>([]);
let currentCollection = $state<CollectionResponse | null>(null);
let children = $state<CollectionResponse[]>([]);
let items = $state<DocumentListEntry[]>([]);
let itemsCursor = $state<string | undefined>(undefined);
let itemsHasMore = $state(true);
let loading = $state(false);
let itemsLoading = $state(false);
let itemsLoadingMore = $state(false);
let sortOrder = $state<CollectionSort>('name_asc');
let fetchError = $state<string | null>(null);

function sortCollections(list: CollectionResponse[]): CollectionResponse[] {
	const sorted = [...list];
	switch (sortOrder) {
		case 'name_asc':
			return sorted.sort((a, b) => a.name.localeCompare(b.name));
		case 'name_desc':
			return sorted.sort((a, b) => b.name.localeCompare(a.name));
		case 'item_count':
			return sorted.sort((a, b) => b.item_count - a.item_count);
		case 'date_created':
			return sorted.sort((a, b) => b.created_at.localeCompare(a.created_at));
		default:
			return sorted;
	}
}

const rootCollections = $derived(sortCollections(allCollections.filter((c) => !c.parent_id)));

const isEmpty = $derived(!loading && allCollections.length === 0);
const itemsEmpty = $derived(!itemsLoading && items.length === 0);

async function loadAllCollections(): Promise<void> {
	loading = true;
	fetchError = null;
	try {
		const results = await fetchAllPages(async (cursor) => {
			const resp = await api.listCollections({
				query: { cursor, limit: 100 }
			});
			if (!resp.data) return undefined;
			return {
				data: resp.data.data as CollectionResponse[],
				page: { next_cursor: resp.data.page.next_cursor ?? null }
			};
		});
		allCollections = results;
	} catch {
		fetchError = 'Failed to load collections';
	} finally {
		loading = false;
	}
}

async function loadCollection(id: string): Promise<void> {
	loading = true;
	fetchError = null;
	currentCollection = null;
	try {
		const resp = await api.getCollection({ path: { id } });
		if (resp.data) {
			currentCollection = resp.data as CollectionResponse;
		} else {
			currentCollection = null;
		}
	} catch {
		currentCollection = null;
		fetchError = 'Failed to load collection';
	} finally {
		loading = false;
	}
}

async function loadChildren(parentId: string): Promise<void> {
	try {
		const resp = await api.listChildren({ path: { id: parentId }, query: { limit: 100 } });
		if (resp.data) {
			children = (resp.data.data as CollectionResponse[])
				.slice()
				.sort((a, b) => a.created_at.localeCompare(b.created_at));
		}
	} catch {
		children = [];
	}
}

async function loadItems(collectionId: string, reset = false): Promise<void> {
	if (reset) {
		items = [];
		itemsCursor = undefined;
		itemsHasMore = true;
	}
	itemsLoading = reset;
	itemsLoadingMore = !reset;
	try {
		const resp = await api.listCollectionEntries({
			path: { id: collectionId },
			query: {
				cursor: itemsCursor ?? null,
				limit: PAGE_LIMIT
			}
		});
		if (resp.data) {
			const incoming = resp.data.data;
			items = reset ? incoming : [...items, ...incoming];
			itemsHasMore = resp.data.page?.has_more ?? incoming.length >= PAGE_LIMIT;
			itemsCursor = resp.data.page?.next_cursor ?? undefined;
		}
	} catch {
		fetchError = 'Failed to load items';
	} finally {
		itemsLoading = false;
		itemsLoadingMore = false;
	}
}

async function createCollection(body: {
	name: string;
	description?: string | null;
	icon?: string | null;
	color?: string | null;
	parent_id?: string | null;
}): Promise<CollectionResponse | null> {
	try {
		const resp = await api.createCollection({ body });
		if (resp.data) {
			const created = resp.data as CollectionResponse;
			allCollections = [...allCollections, created];
			return created;
		}
	} catch {
		fetchError = 'Failed to create collection';
	}
	return null;
}

async function updateCollection(
	id: string,
	body: {
		name?: string;
		description?: string | null;
		icon?: string | null;
		color?: string | null;
		parent_id?: string | null;
	}
): Promise<CollectionResponse | null> {
	try {
		const resp = await api.updateCollection({ path: { id }, body });
		if (resp.data) {
			const updated = resp.data as CollectionResponse;
			allCollections = allCollections.map((c) => (c.id === id ? updated : c));
			if (currentCollection?.id === id) {
				currentCollection = updated;
			}
			return updated;
		}
	} catch {
		fetchError = 'Failed to update collection';
	}
	return null;
}

async function deleteCollection(id: string): Promise<boolean> {
	try {
		await api.deleteCollection({ path: { id } });
		allCollections = allCollections.filter((c) => c.id !== id);
		if (currentCollection?.id === id) {
			currentCollection = null;
		}
		return true;
	} catch {
		fetchError = 'Failed to delete collection';
		return false;
	}
}

async function addItem(collectionId: string, libraryEntryId: string): Promise<boolean> {
	try {
		await api.addEntryToCollection({
			path: { id: collectionId },
			body: { library_entry_id: libraryEntryId }
		});
		return true;
	} catch {
		return false;
	}
}

async function removeItem(collectionId: string, libraryEntryId: string): Promise<boolean> {
	try {
		await api.removeEntryFromCollection({
			path: { id: collectionId, library_entry_id: libraryEntryId }
		});
		return true;
	} catch {
		return false;
	}
}

export function getCollections() {
	return {
		get allCollections() {
			return allCollections;
		},
		get rootCollections() {
			return rootCollections;
		},
		get currentCollection() {
			return currentCollection;
		},
		get children() {
			return children;
		},
		get items() {
			return items;
		},
		get loading() {
			return loading;
		},
		get itemsLoading() {
			return itemsLoading;
		},
		get itemsLoadingMore() {
			return itemsLoadingMore;
		},
		get itemsHasMore() {
			return itemsHasMore;
		},
		get isEmpty() {
			return isEmpty;
		},
		get itemsEmpty() {
			return itemsEmpty;
		},
		get sortOrder() {
			return sortOrder;
		},
		get fetchError() {
			return fetchError;
		},
		setSortOrder(order: CollectionSort) {
			sortOrder = order;
		},
		loadAllCollections,
		loadCollection,
		loadChildren,
		loadItems,
		createCollection,
		updateCollection,
		deleteCollection,
		addItem,
		removeItem
	};
}
