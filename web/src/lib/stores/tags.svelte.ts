import { SvelteSet } from 'svelte/reactivity';
import * as api from '$lib/api';
import { getSidebar } from '$lib/stores/sidebar.svelte';
import type { TagResponse, DocumentListEntry, HighlightResponse } from '$lib/api';
import { fetchAllPages } from '$lib/api/pagination';

export type TagScope = 'all' | 'document' | 'highlight';
export type TagSort = 'name_asc' | 'name_desc' | 'item_count' | 'date_created';

type CreateTagResult = { ok: true; data: TagResponse } | { ok: false; error: string };

const PAGE_LIMIT = 50;

let allTags = $state<TagResponse[]>([]);
let activeScope = $state<TagScope>('all');
let sortOrder = $state<TagSort>('name_asc');
let searchQuery = $state('');
let currentTag = $state<TagResponse | null>(null);
let tagItems = $state<DocumentListEntry[]>([]);
let tagHighlights = $state<HighlightResponse[]>([]);
let itemsCursor = $state<string | undefined>(undefined);
let highlightsCursor = $state<string | undefined>(undefined);
let itemsHasMore = $state(true);
let highlightsHasMore = $state(true);
let loading = $state(false);
let itemsLoading = $state(false);
let highlightsLoading = $state(false);
let itemsLoadingMore = $state(false);
let highlightsLoadingMore = $state(false);
const selectedIds = new SvelteSet<string>();
let fetchError = $state<string | null>(null);

function sortTags(list: TagResponse[]): TagResponse[] {
	const sorted = [...list];
	switch (sortOrder) {
		case 'name_asc':
			return sorted.sort((a, b) => a.name.localeCompare(b.name));
		case 'name_desc':
			return sorted.sort((a, b) => b.name.localeCompare(a.name));
		case 'item_count':
			return sorted.sort(
				(a, b) => b.item_count + b.highlight_count - (a.item_count + a.highlight_count)
			);
		case 'date_created':
			return sorted.sort((a, b) => b.created_at.localeCompare(a.created_at));
		default:
			return sorted;
	}
}

const filteredTags = $derived.by(() => {
	let list = allTags;
	if (activeScope === 'document') {
		list = list.filter((tag) => tag.item_count > 0);
	} else if (activeScope === 'highlight') {
		list = list.filter((tag) => tag.highlight_count > 0);
	}
	if (searchQuery.trim()) {
		const q = searchQuery.trim().toLowerCase();
		list = list.filter(
			(t) => t.name.toLowerCase().includes(q) || t.aliases.some((a) => a.toLowerCase().includes(q))
		);
	}
	return sortTags(list);
});

const isEmpty = $derived(!loading && allTags.length === 0);

async function fetchTags(): Promise<TagResponse[]> {
	return fetchAllPages(async (cursor) => {
		const resp = await api.listTags({
			query: { cursor, limit: 100 }
		});
		if (!resp.data) return undefined;
		return {
			data: resp.data.data as TagResponse[],
			page: { next_cursor: resp.data.page.next_cursor ?? null }
		};
	});
}

async function loadAllTags(): Promise<void> {
	loading = true;
	fetchError = null;
	try {
		allTags = await fetchTags();
	} catch {
		fetchError = 'Failed to load tags';
	} finally {
		loading = false;
	}
}

async function loadTag(id: string): Promise<void> {
	loading = true;
	fetchError = null;
	currentTag = null;
	try {
		const resp = await api.getTag({ path: { id } });
		if (resp.data) {
			currentTag = resp.data as TagResponse;
		} else {
			currentTag = null;
		}
	} catch {
		currentTag = null;
		fetchError = 'Failed to load tag';
	} finally {
		loading = false;
	}
}

async function loadTagItems(tagId: string, reset = false): Promise<void> {
	if (reset) {
		tagItems = [];
		itemsCursor = undefined;
		itemsHasMore = true;
	}
	itemsLoading = reset;
	itemsLoadingMore = !reset;
	try {
		const resp = await api.listTagEntries({
			path: { id: tagId },
			query: { cursor: itemsCursor ?? null, limit: PAGE_LIMIT }
		});
		if (resp.data) {
			const incoming = resp.data.data;
			tagItems = reset ? incoming : [...tagItems, ...incoming];
			itemsHasMore = resp.data.page?.has_more ?? incoming.length >= PAGE_LIMIT;
			itemsCursor = resp.data.page?.next_cursor ?? undefined;
		}
	} catch {
		fetchError = 'Failed to load tag items';
	} finally {
		itemsLoading = false;
		itemsLoadingMore = false;
	}
}

async function deleteTagItem(itemId: string): Promise<void> {
	const prev = tagItems.find((i) => i.id === itemId);
	if (!prev) return;

	// Optimistic removal; the entry is recoverable from Trash after the call.
	tagItems = tagItems.filter((i) => i.id !== itemId);
	try {
		await api.deleteLibraryEntry({ path: { document_id: itemId } });
		await getSidebar().refreshTrashCount();
	} catch {
		tagItems = [prev, ...tagItems];
	}
}

async function loadTagHighlights(tagId: string, reset = false): Promise<void> {
	if (reset) {
		tagHighlights = [];
		highlightsCursor = undefined;
		highlightsHasMore = true;
	}
	highlightsLoading = reset;
	highlightsLoadingMore = !reset;
	try {
		const resp = await api.listTagHighlights({
			path: { id: tagId },
			query: { cursor: highlightsCursor ?? null, limit: PAGE_LIMIT }
		});
		if (resp.data) {
			const incoming = resp.data.data as HighlightResponse[];
			tagHighlights = reset ? incoming : [...tagHighlights, ...incoming];
			highlightsHasMore = resp.data.page?.has_more ?? incoming.length >= PAGE_LIMIT;
			highlightsCursor = resp.data.page?.next_cursor ?? undefined;
		}
	} catch {
		fetchError = 'Failed to load tag highlights';
	} finally {
		highlightsLoading = false;
		highlightsLoadingMore = false;
	}
}

function createTagError(error: unknown): string {
	if (!error || typeof error !== 'object') return 'Failed to create tag';
	const problem = error as Record<string, unknown>;
	const errors = Array.isArray(problem.errors) ? problem.errors : [];
	const firstError = errors[0];
	if (firstError && typeof firstError === 'object') {
		const message = (firstError as Record<string, unknown>).message;
		if (typeof message === 'string' && message.trim()) return message;
	}
	if (typeof problem.detail === 'string' && problem.detail.trim()) return problem.detail;
	if (typeof problem.message === 'string' && problem.message.trim()) return problem.message;
	return 'Failed to create tag';
}

async function createTag(body: {
	name: string;
	color?: string | null;
	parent_id?: string | null;
}): Promise<CreateTagResult> {
	try {
		const resp = await api.createTag({ body });
		if (resp.data) {
			const created = resp.data as TagResponse;
			allTags = [...allTags, created];
			return { ok: true, data: created };
		}
		return { ok: false, error: createTagError(resp.error) };
	} catch (error) {
		return { ok: false, error: createTagError(error) };
	}
}

async function findTagByExactName(name: string): Promise<TagResponse | null> {
	const needle = name.trim().toLowerCase();
	const loaded = allTags.find((tag) => tag.name.toLowerCase() === needle);
	if (loaded) return loaded;
	try {
		const tags = await fetchTags();
		return tags.find((tag) => tag.name.toLowerCase() === needle) ?? null;
	} catch {
		return null;
	}
}

async function updateTag(
	id: string,
	body: { name?: string; color?: string | null; parent_id?: string | null }
): Promise<TagResponse | null> {
	try {
		const resp = await api.updateTag({ path: { id }, body });
		if (resp.data) {
			const updated = resp.data as TagResponse;
			allTags = allTags.map((t) => (t.id === id ? updated : t));
			if (currentTag?.id === id) {
				currentTag = updated;
			}
			return updated;
		}
	} catch {
		fetchError = 'Failed to update tag';
	}
	return null;
}

async function deleteTag(id: string): Promise<boolean> {
	return deleteTags([id]);
}

async function deleteTags(ids: string[]): Promise<boolean> {
	if (ids.length === 0) return true;

	const deletedIds = new SvelteSet<string>();
	try {
		for (const id of ids) {
			await api.deleteTag({ path: { id } });
			deletedIds.add(id);
		}

		if (deletedIds.size === 0) return true;

		allTags = allTags.filter((t) => !deletedIds.has(t.id));
		if (currentTag && deletedIds.has(currentTag.id)) {
			currentTag = null;
		}
		for (const id of deletedIds) selectedIds.delete(id);
		return true;
	} catch {
		if (deletedIds.size > 0) {
			allTags = allTags.filter((t) => !deletedIds.has(t.id));
			if (currentTag && deletedIds.has(currentTag.id)) {
				currentTag = null;
			}
			for (const id of deletedIds) selectedIds.delete(id);
		}
		fetchError = ids.length === 1 ? 'Failed to delete tag' : 'Failed to delete some tags';
		return false;
	}
}

async function mergeTagsAction(sourceIds: string[], targetId: string): Promise<boolean> {
	try {
		await api.mergeTags({
			body: { source_ids: sourceIds, target_id: targetId }
		});
		allTags = allTags.filter((t) => !sourceIds.includes(t.id));
		selectedIds.clear();
		return true;
	} catch {
		fetchError = 'Failed to merge tags';
		return false;
	}
}

function toggleSelection(id: string): void {
	if (selectedIds.has(id)) {
		selectedIds.delete(id);
	} else {
		selectedIds.add(id);
	}
}

function clearSelection(): void {
	selectedIds.clear();
}

export function getTags() {
	return {
		get allTags() {
			return allTags;
		},
		get filteredTags() {
			return filteredTags;
		},
		get activeScope() {
			return activeScope;
		},
		get sortOrder() {
			return sortOrder;
		},
		get searchQuery() {
			return searchQuery;
		},
		get currentTag() {
			return currentTag;
		},
		get tagItems() {
			return tagItems;
		},
		get tagHighlights() {
			return tagHighlights;
		},
		get loading() {
			return loading;
		},
		get itemsLoading() {
			return itemsLoading;
		},
		get highlightsLoading() {
			return highlightsLoading;
		},
		get itemsLoadingMore() {
			return itemsLoadingMore;
		},
		get highlightsLoadingMore() {
			return highlightsLoadingMore;
		},
		get itemsHasMore() {
			return itemsHasMore;
		},
		get highlightsHasMore() {
			return highlightsHasMore;
		},
		get isEmpty() {
			return isEmpty;
		},
		get selectedIds() {
			return selectedIds;
		},
		get fetchError() {
			return fetchError;
		},
		setScope(scope: TagScope) {
			activeScope = scope;
		},
		setSortOrder(order: TagSort) {
			sortOrder = order;
		},
		setSearchQuery(q: string) {
			searchQuery = q;
		},
		toggleSelection,
		clearSelection,
		loadAllTags,
		loadTag,
		loadTagItems,
		deleteTagItem,
		loadTagHighlights,
		createTag,
		findTagByExactName,
		updateTag,
		deleteTag,
		deleteTags,
		mergeTags: mergeTagsAction
	};
}
