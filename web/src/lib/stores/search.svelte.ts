import * as apiSdk from '$lib/api';
import type {
	SearchEntityCardResponse,
	SearchResultResponse,
	SearchSuggestionResponse,
	RecentSearchResponse
} from '$lib/api/generated/types.gen';

const PAGE_LIMIT = 20;
const DEBOUNCE_MS = 300;

export function resultKey(r: SearchResultResponse): string {
	return r.document_id ?? r.delivery_id ?? r.source_entry_id ?? '';
}

let results = $state<SearchResultResponse[]>([]);
let entityCard = $state<SearchEntityCardResponse | null>(null);
let cursor = $state<string | undefined>(undefined);
let hasMore = $state(false);
let loading = $state(false);
let loadingMore = $state(false);
let query = $state('');
let submittedQuery = $state('');
let selectedId = $state<string | null>(null);
let fetchError = $state<string | null>(null);
let resultCount = $state<string | null>(null);

let suggestionItems = $state<SearchSuggestionResponse[]>([]);
let suggestionsVisible = $state(false);
let suggestionsLoading = $state(false);
let highlightedIndex = $state(-1);

let recentSearches = $state<RecentSearchResponse[]>([]);
let recentLoading = $state(false);

const selectedResult = $derived(results.find((r) => resultKey(r) === selectedId) ?? null);
const isEmpty = $derived(!loading && results.length === 0 && submittedQuery.length > 0);
const showRecent = $derived(submittedQuery.length === 0 && !loading);

let debounceTimer: ReturnType<typeof setTimeout> | undefined;

async function executeSearch(q: string, newCursor?: string): Promise<void> {
	try {
		const { data } = await apiSdk.search({
			query: {
				q,
				cursor: newCursor ?? null,
				limit: PAGE_LIMIT
			}
		});
		if (data) {
			const incoming = data.results ?? [];
			if (!newCursor) {
				results = incoming;
				entityCard = data.entity_card ?? null;
			} else {
				results = [...results, ...incoming];
			}
			hasMore = data.has_more ?? false;
			cursor = data.next_cursor ?? undefined;

			if (!newCursor) {
				const count = incoming.length;
				if (hasMore) {
					resultCount = `${count}+ results for "${q}"`;
				} else {
					resultCount = `${count} result${count !== 1 ? 's' : ''} for "${q}"`;
				}
			}
		}
	} catch {
		fetchError = 'Search failed. Please try again.';
	}
}

async function submitSearch(q?: string): Promise<void> {
	const searchQuery = q ?? query;
	if (!searchQuery.trim()) return;

	submittedQuery = searchQuery.trim();
	query = submittedQuery;
	results = [];
	entityCard = null;
	cursor = undefined;
	hasMore = false;
	fetchError = null;
	selectedId = null;
	resultCount = null;
	suggestionsVisible = false;
	loading = true;

	await executeSearch(submittedQuery);

	if (selectedId === null && results.length > 0) {
		selectedId = resultKey(results[0]!);
	}
	loading = false;
}

async function loadMore(): Promise<void> {
	if (loadingMore || !hasMore || !submittedQuery) return;
	loadingMore = true;
	await executeSearch(submittedQuery, cursor);
	loadingMore = false;
}

async function fetchSuggestions(input: string): Promise<void> {
	suggestionsLoading = true;
	try {
		const { data } = await apiSdk.suggestions({
			query: { q: input, limit: 16 }
		});
		if (data) {
			suggestionItems = data.suggestions ?? [];
			suggestionsVisible = suggestionItems.length > 0;
			highlightedIndex = -1;
		}
	} catch {
		suggestionItems = [];
		suggestionsVisible = false;
	} finally {
		suggestionsLoading = false;
	}
}

function debouncedSuggestions(input: string): void {
	clearTimeout(debounceTimer);
	debounceTimer = setTimeout(() => fetchSuggestions(input), DEBOUNCE_MS);
}

function showSuggestions(): void {
	clearTimeout(debounceTimer);
	void fetchSuggestions(query);
}

async function loadRecentSearches(): Promise<void> {
	recentLoading = true;
	try {
		const { data } = await apiSdk.listRecentSearches({
			query: { limit: 20 }
		});
		if (data) {
			recentSearches = data.items ?? [];
		}
	} catch {
		recentSearches = [];
	} finally {
		recentLoading = false;
	}
}

async function clearAllRecent(): Promise<void> {
	try {
		await apiSdk.clearRecentSearches();
		recentSearches = [];
	} catch {
		// Silently fail
	}
}

async function deleteRecent(id: string): Promise<void> {
	recentSearches = recentSearches.filter((r) => r.id !== id);
	try {
		await apiSdk.deleteRecentSearch({ path: { recent_search_id: id } });
	} catch {
		// Silently fail — already removed optimistically
	}
}

function clearSearch(): void {
	query = '';
	submittedQuery = '';
	results = [];
	entityCard = null;
	cursor = undefined;
	hasMore = false;
	fetchError = null;
	selectedId = null;
	resultCount = null;
	suggestionItems = [];
	suggestionsVisible = false;
	highlightedIndex = -1;
}

function applySuggestion(suggestion: SearchSuggestionResponse): void {
	const tokens = query.trim().split(/\s+/);
	tokens.pop();
	tokens.push(suggestion.insert_text);
	query = tokens.join(' ') + ' ';
	suggestionsVisible = false;
	highlightedIndex = -1;
}

export function getSearch() {
	return {
		get results() {
			return results;
		},
		get entityCard() {
			return entityCard;
		},
		get loading() {
			return loading;
		},
		get loadingMore() {
			return loadingMore;
		},
		get hasMore() {
			return hasMore;
		},
		get query() {
			return query;
		},
		set query(value: string) {
			query = value;
			debouncedSuggestions(value);
		},
		get submittedQuery() {
			return submittedQuery;
		},
		get selectedId() {
			return selectedId;
		},
		get selectedResult() {
			return selectedResult;
		},
		get isEmpty() {
			return isEmpty;
		},
		get showRecent() {
			return showRecent;
		},
		get fetchError() {
			return fetchError;
		},
		get resultCount() {
			return resultCount;
		},
		get suggestionItems() {
			return suggestionItems;
		},
		get suggestionsVisible() {
			return suggestionsVisible;
		},
		get suggestionsLoading() {
			return suggestionsLoading;
		},
		get highlightedIndex() {
			return highlightedIndex;
		},
		set highlightedIndex(value: number) {
			highlightedIndex = value;
		},
		get recentSearches() {
			return recentSearches;
		},
		get recentLoading() {
			return recentLoading;
		},
		setSelectedId(id: string | null) {
			selectedId = id;
		},
		submitSearch,
		loadMore,
		loadRecentSearches,
		clearAllRecent,
		deleteRecent,
		clearSearch,
		applySuggestion,
		showSuggestions,
		hideSuggestions() {
			suggestionsVisible = false;
			highlightedIndex = -1;
		}
	};
}
