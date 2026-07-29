import * as api from '$lib/api';
import type { CollectionResponse, SmartListResponse } from '$lib/api/generated/types.gen';
import { fetchAllPages, buildCollectionTree, type CollectionNode } from '$lib/api/pagination';

export type NavMode = 'content-type' | 'collections';

const STORAGE_KEY_NAV_MODE = 'indelible:sidebar_nav_mode';
const STORAGE_KEY_TREE_STATE = 'indelible:sidebar_tree_state';

function loadNavMode(): NavMode {
	try {
		const stored = localStorage.getItem(STORAGE_KEY_NAV_MODE);
		if (stored === 'collections') return 'collections';
	} catch {
		// SSR or localStorage unavailable
	}
	return 'content-type';
}

function loadTreeState(): Record<string, boolean> {
	try {
		const stored = localStorage.getItem(STORAGE_KEY_TREE_STATE);
		if (stored) return JSON.parse(stored);
	} catch {
		// Malformed JSON or SSR
	}
	return {};
}

function saveTreeState(state: Record<string, boolean>): void {
	try {
		localStorage.setItem(STORAGE_KEY_TREE_STATE, JSON.stringify(state));
	} catch {
		// Quota exceeded or SSR
	}
}

let navMode = $state<NavMode>(loadNavMode());
let allCollections = $state<CollectionResponse[]>([]);
let collectionTree = $state<CollectionNode[]>([]);
let expandedState = $state<Record<string, boolean>>(loadTreeState());
let pinnedSmartLists = $state<SmartListResponse[]>([]);
let itemTypeCounts = $state<Record<string, number>>({});
let trashCountValue = $state(0);
let collectionsLoading = $state(false);
let smartListsLoading = $state(false);

function rebuildTree(): void {
	collectionTree = buildCollectionTree(allCollections, expandedState);
}

async function refreshCollections(): Promise<void> {
	collectionsLoading = true;
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
		rebuildTree();
	} catch {
		// Silently fail — sidebar still shows without collections
	} finally {
		collectionsLoading = false;
	}
}

async function refreshSmartLists(): Promise<void> {
	smartListsLoading = true;
	try {
		const results = await fetchAllPages(async (cursor) => {
			const resp = await api.listSmartLists({
				query: { cursor, limit: 100 }
			});
			if (!resp.data) return undefined;
			return {
				data: resp.data.data as SmartListResponse[],
				page: { next_cursor: resp.data.page.next_cursor ?? null }
			};
		});
		pinnedSmartLists = results.filter((sl) => sl.is_pinned);
	} catch {
		// Silently fail
	} finally {
		smartListsLoading = false;
	}
}

async function refreshItemTypeCounts(): Promise<void> {
	try {
		const resp = await api.itemTypeCounts();
		if (resp.data) {
			itemTypeCounts = resp.data.counts;
		}
	} catch {
		// Silently fail — counts just won't show
	}
}

async function refreshTrashCount(): Promise<void> {
	try {
		const resp = await api.trashCount();
		if (resp.data) {
			trashCountValue = resp.data.count;
		}
	} catch {
		// Silently fail
	}
}

function toggleNodeExpanded(collectionId: string): void {
	expandedState = { ...expandedState, [collectionId]: !expandedState[collectionId] };
	saveTreeState(expandedState);
	rebuildTree();
}

function setNavMode(mode: NavMode): void {
	navMode = mode;
	try {
		localStorage.setItem(STORAGE_KEY_NAV_MODE, mode);
	} catch {
		// Quota exceeded or SSR
	}
}

function toggleNavMode(): void {
	setNavMode(navMode === 'content-type' ? 'collections' : 'content-type');
}

async function initSidebar(): Promise<void> {
	await Promise.all([
		refreshCollections(),
		refreshSmartLists(),
		refreshTrashCount(),
		refreshItemTypeCounts()
	]);
}

export function getSidebar() {
	return {
		get navMode() {
			return navMode;
		},
		get collectionTree() {
			return collectionTree;
		},
		get allCollections() {
			return allCollections;
		},
		get pinnedSmartLists() {
			return pinnedSmartLists;
		},
		get itemTypeCounts() {
			return itemTypeCounts;
		},
		get trashCount() {
			return trashCountValue;
		},
		get collectionsLoading() {
			return collectionsLoading;
		},
		get smartListsLoading() {
			return smartListsLoading;
		},
		toggleNavMode,
		setNavMode,
		toggleNodeExpanded,
		refreshCollections,
		refreshSmartLists,
		refreshTrashCount,
		refreshItemTypeCounts,
		initSidebar
	};
}
