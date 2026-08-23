import * as apiSdk from '$lib/api';
import { loadPreferencesSettings, savePreferencesSettings } from '$lib/api/settings';
import { SvelteSet } from 'svelte/reactivity';
import type {
	DocumentListEntry,
	ListDensityDto,
	LibraryQueryBody,
	PreferencesSettingsResponse,
	RealtimeEventResponse,
	SidebarModeDto,
	SmartListResponse,
	TriageModeDto
} from '$lib/api';
import { LIBRARY_DOMAIN_EVENT_TYPES } from '$lib/realtime/event-types';
import { getSidebar } from '$lib/stores/sidebar.svelte';
import {
	buildFilterExpression,
	fromApiFilterExpression,
	isFlatFilterExpression,
	parseFilterExpression,
	toApiFilterExpression,
	type FilterExpression,
	type FilterCondition
} from '$lib/utils/filter-expression';
import { t, type MessageKey } from '$lib/i18n';
import { get } from 'svelte/store';

export type TriageTab = 'inbox' | 'later' | 'archive';
export type GroupBy = 'triage' | 'read_status' | 'none';
export type ReadStatusTab = 'unseen' | 'seen';
export type TriageOption = { value: TriageTab; labelKey: MessageKey };
export type SortOrder =
	| 'date_saved_desc'
	| 'date_saved_asc'
	| 'date_published_desc'
	| 'date_published_asc'
	| 'title_asc'
	| 'title_desc'
	| 'reading_progress'
	| 'reading_time';

const PAGE_LIMIT = 50;
const SORT_KEY = 'indelible_library_sort';
const SIDEBAR_MODE_KEY = 'indelible_sidebar_mode';
const SHOW_COUNT_BADGE_KEY = 'indelible_show_count_badge';
const REALTIME_RESET_DEBOUNCE_MS = 500;

function getInitialSort(): SortOrder {
	try {
		return (localStorage.getItem(SORT_KEY) as SortOrder) || 'date_saved_desc';
	} catch {
		return 'date_saved_desc';
	}
}

function getInitialSidebarMode(): SidebarModeDto {
	try {
		return (localStorage.getItem(SIDEBAR_MODE_KEY) as SidebarModeDto) || 'expanded';
	} catch {
		return 'expanded';
	}
}

function getInitialShowCountBadge(): boolean {
	try {
		const stored = localStorage.getItem(SHOW_COUNT_BADGE_KEY);
		return stored === null ? true : stored === 'true';
	} catch {
		return true;
	}
}

const SLUG_TO_API_TYPE: Record<string, string> = {
	articles: 'article',
	books: 'book',
	emails: 'email',
	pdfs: 'pdf',
	tweets: 'tweet',
	videos: 'video',
	podcasts: 'podcast'
};

export function triageOptionsForMode(mode: TriageModeDto): TriageOption[] {
	return mode === 'manual'
		? [
				{ value: 'inbox', labelKey: 'library_triage_saved' },
				{ value: 'archive', labelKey: 'library_triage_archived' }
			]
		: [
				{ value: 'inbox', labelKey: 'library_triage_inbox' },
				{ value: 'later', labelKey: 'library_triage_later' },
				{ value: 'archive', labelKey: 'library_triage_archive' }
			];
}

function coerceTriageTabForMode(tab: TriageTab, mode: TriageModeDto): TriageTab {
	return triageOptionsForMode(mode).some((option) => option.value === tab) ? tab : 'inbox';
}

type BuildLibraryItemsQueryBodyInput = {
	draftConditions: FilterCondition[];
	draftConjunction: 'and' | 'or';
	activeType?: string;
	groupBy: GroupBy;
	triageTab: TriageTab;
	cursor?: string;
	limit: number;
};

export function buildLibraryItemsQueryBody({
	draftConditions,
	draftConjunction,
	activeType,
	groupBy,
	triageTab,
	cursor,
	limit
}: BuildLibraryItemsQueryBodyInput): LibraryQueryBody {
	const scopeConditions: FilterCondition[] = [];
	const hasExplicitItemType = draftConditions.some((condition) => condition.field === 'item_type');
	const hasExplicitTriageState = draftConditions.some(
		(condition) => condition.field === 'triage_state'
	);

	if (!hasExplicitItemType && activeType) {
		scopeConditions.push({
			id: 'scope:item_type',
			field: 'item_type',
			op: 'eq',
			value: SLUG_TO_API_TYPE[activeType] ?? activeType
		});
	}

	if (!hasExplicitTriageState && groupBy === 'triage') {
		scopeConditions.push({
			id: 'scope:triage_state',
			field: 'triage_state',
			op: 'eq',
			value: triageTab
		});
	}

	const draftExpression =
		draftConditions.length > 0
			? (buildFilterExpression([...draftConditions], draftConjunction) as FilterExpression)
			: null;
	const scopeExpression =
		scopeConditions.length > 0
			? (buildFilterExpression(scopeConditions, 'and') as FilterExpression)
			: null;

	let filterExpression: FilterExpression | null = null;
	if (draftExpression && scopeExpression) {
		filterExpression = {
			type: 'and',
			conditions: [draftExpression, scopeExpression]
		};
	} else {
		filterExpression = draftExpression ?? scopeExpression;
	}

	return {
		filter_expression: toApiFilterExpression(filterExpression) ?? null,
		cursor: cursor ?? null,
		limit
	};
}

type BuildSmartListItemsQueryBodyInput = {
	filterExpression: FilterExpression | null;
	cursor?: string;
	limit: number;
};

// A smart list is a complete scope: its expression is sent verbatim, never composed
// with the page's type. An email-only list opened under /library/articles previously
// ANDed item_type=article into the query and emptied the view.
export function buildSmartListItemsQueryBody({
	filterExpression,
	cursor,
	limit
}: BuildSmartListItemsQueryBodyInput): LibraryQueryBody {
	return {
		filter_expression: toApiFilterExpression(filterExpression) ?? null,
		cursor: cursor ?? null,
		limit
	};
}

let items = $state<DocumentListEntry[]>([]);
let cursor = $state<string | undefined>(undefined);
let hasMore = $state(true);
let loading = $state(false);
let loadingMore = $state(false);
let triageTab = $state<TriageTab>('inbox');
let activeType = $state<string | undefined>(undefined);
let sortOrder = $state<SortOrder>(getInitialSort());
let selectedId = $state<string | null>(null);
let fetchError = $state<string | null>(null);
let triageMode = $state<TriageModeDto>('focus');
let listDensity = $state<ListDensityDto>('comfortable');
let sidePanelOpen = $state(true);
let sidebarMode = $state<SidebarModeDto>(getInitialSidebarMode());
let sidebarSessionOverride = $state<'expanded' | 'collapsed' | null>(null);
let cachedPrefs: PreferencesSettingsResponse | null = null;
let prefsLoaded = false;
let groupBy = $state<GroupBy>('triage');
let readStatusTab = $state<ReadStatusTab>('unseen');
let realtimeResetTimer: ReturnType<typeof setTimeout> | null = null;
const handledRealtimeEventIds = new SvelteSet<string>();
const LIBRARY_INVALIDATION_EVENTS = new Set<string>(LIBRARY_DOMAIN_EVENT_TYPES);
const REMOVE_ITEM_EVENTS = new Set(['library_entry.trashed', 'library_entry.permanently_deleted']);

let smartListId = $state<string | null>(null);
let activeSmartList = $state<SmartListResponse | null>(null);
let filterBarOpen = $state(false);
let draftConditions = $state<FilterCondition[]>([]);
let draftConjunction = $state<'and' | 'or'>('and');
// Until the user actually edits a rule, a saved view is served by its stored
// expression. Seeding the draft is lossy for shapes the bar cannot render, so the
// draft only becomes authoritative once someone has deliberately changed it.
let draftTouched = $state(false);
let viewPanelOpen = $state(false);
let showCountBadge = $state(getInitialShowCountBadge());

const selectedItem = $derived(items.find((i) => i.id === selectedId) ?? null);
const isEmpty = $derived(!loading && items.length === 0);

// The filter bar's draft is the single source of truth for what gets queried, so
// the same expression backs the request, the save-as-view fork, and the modified
// indicator. Building it in one place keeps those three from ever disagreeing.
function draftExpressionOrNull(): FilterExpression | null {
	return draftConditions.length > 0
		? buildFilterExpression([...draftConditions], draftConjunction)
		: null;
}

function savedSmartListExpression(): FilterExpression | null {
	return fromApiFilterExpression(activeSmartList?.filter_expression ?? null);
}

// A saved expression the flat bar cannot represent — a nested group or a negation —
// would parse into a draft that means something else entirely, so such a view is
// queried verbatim and never opened for editing.
const smartListAdvanced = $derived(
	Boolean(activeSmartList) && !isFlatFilterExpression(activeSmartList?.filter_expression)
);

const smartListModified = $derived.by(() => {
	if (!activeSmartList || !draftTouched) return false;
	return (
		JSON.stringify(toApiFilterExpression(draftExpressionOrNull())) !==
		JSON.stringify(activeSmartList.filter_expression ?? null)
	);
});

function sortItems(list: DocumentListEntry[]): DocumentListEntry[] {
	const sorted = [...list];
	switch (sortOrder) {
		case 'date_saved_asc':
			return sorted.sort((a, b) => a.saved_at.localeCompare(b.saved_at));
		case 'date_saved_desc':
			return sorted.sort((a, b) => b.saved_at.localeCompare(a.saved_at));
		case 'date_published_desc':
			return sorted.sort((a, b) => (b.published_at ?? '').localeCompare(a.published_at ?? ''));
		case 'date_published_asc':
			return sorted.sort((a, b) => (a.published_at ?? '').localeCompare(b.published_at ?? ''));
		case 'title_asc':
			return sorted.sort((a, b) => a.title.localeCompare(b.title));
		case 'title_desc':
			return sorted.sort((a, b) => b.title.localeCompare(a.title));
		case 'reading_time':
			return sorted.sort((a, b) => (b.reading_time_minutes ?? 0) - (a.reading_time_minutes ?? 0));
		case 'reading_progress':
			return sorted.sort(
				(a, b) =>
					(b.max_progress_percent ?? b.progress_percent ?? 0) -
					(a.max_progress_percent ?? a.progress_percent ?? 0)
			);
		default:
			return sorted;
	}
}

// Monotonic token so a slow response from a superseded fetch (e.g. the type-page fetch
// racing the smart-list fetch on mount) can never overwrite newer results.
let fetchGeneration = 0;

function describeQueryError(error: unknown): string {
	if (error && typeof error === 'object') {
		const e = error as { errors?: Array<{ message?: string }>; detail?: string; title?: string };
		const first = e.errors?.[0]?.message;
		if (first) return first;
		if (e.detail && e.detail !== 'validation error') return e.detail;
		if (e.title) return e.title;
	}
	return get(t)('library_error_load_items');
}

async function fetchPage(): Promise<void> {
	const generation = ++fetchGeneration;
	try {
		// An untouched saved view is served by its stored expression, so it returns
		// exactly its saved rows whatever shape it has; once the user edits a rule the
		// visible draft takes over. Either way the expression is sent verbatim, never
		// composed with the page's item_type scope.
		const body =
			smartListId && activeSmartList
				? buildSmartListItemsQueryBody({
						filterExpression: draftTouched ? draftExpressionOrNull() : savedSmartListExpression(),
						cursor,
						limit: PAGE_LIMIT
					})
				: buildLibraryItemsQueryBody({
						draftConditions,
						draftConjunction,
						activeType,
						groupBy,
						triageTab,
						cursor,
						limit: PAGE_LIMIT
					});
		const { data, error } = await apiSdk.queryLibraryEntries({ body });
		if (generation !== fetchGeneration) return;
		if (error) {
			fetchError = describeQueryError(error);
			return;
		}
		if (data) {
			const incoming: DocumentListEntry[] = data.data ?? [];
			if (cursor === undefined) {
				items = sortItems(incoming);
			} else {
				items = sortItems([...items, ...incoming]);
			}
			hasMore = data.page?.has_more ?? incoming.length >= PAGE_LIMIT;
			cursor = data.page?.next_cursor ?? undefined;
		}
	} catch {
		if (generation === fetchGeneration) {
			fetchError = get(t)('library_error_load_items');
		}
	}
}

async function resetAndFetch(): Promise<void> {
	if (realtimeResetTimer) {
		clearTimeout(realtimeResetTimer);
		realtimeResetTimer = null;
	}
	items = [];
	cursor = undefined;
	hasMore = true;
	fetchError = null;
	selectedId = null;
	loading = true;
	await fetchPage();
	if (selectedId === null && items.length > 0) {
		selectedId = items[0]!.id;
	}
	loading = false;
}

async function loadMore(): Promise<void> {
	if (loadingMore || !hasMore) return;
	loadingMore = true;
	await fetchPage();
	loadingMore = false;
}

function payloadDocumentId(event: RealtimeEventResponse): string | null {
	const payload = event.payload;
	if (payload && typeof payload === 'object' && 'document_id' in payload) {
		const documentId = (payload as { document_id?: unknown }).document_id;
		if (typeof documentId === 'string' && documentId.length > 0) return documentId;
	}
	return event.aggregate_type === 'document' ? event.aggregate_id : null;
}

function hasBackendOwnedActiveFilter(): boolean {
	return Boolean(smartListId || draftConditions.length > 0);
}

function scheduleRealtimeReset(): void {
	if (realtimeResetTimer) clearTimeout(realtimeResetTimer);
	realtimeResetTimer = setTimeout(() => {
		realtimeResetTimer = null;
		void resetAndFetch();
	}, REALTIME_RESET_DEBOUNCE_MS);
}

function removeItemById(itemId: string): void {
	items = items.filter((item) => item.id !== itemId);
	if (selectedId === itemId) selectedId = null;
}

function itemMatchesSimpleView(item: DocumentListEntry): boolean {
	if (activeType && item.item_type !== (SLUG_TO_API_TYPE[activeType] ?? activeType)) {
		return false;
	}
	if (groupBy === 'triage' && item.triage_state !== triageTab) {
		return false;
	}
	return true;
}

function rememberRealtimeEvent(id: string): boolean {
	if (handledRealtimeEventIds.has(id)) return false;
	handledRealtimeEventIds.add(id);
	if (handledRealtimeEventIds.size > 500) {
		const oldest = handledRealtimeEventIds.values().next().value;
		if (oldest) handledRealtimeEventIds.delete(oldest);
	}
	return true;
}

async function handleDomainEvent(event: RealtimeEventResponse): Promise<void> {
	if (!rememberRealtimeEvent(event.id)) return;
	if (!LIBRARY_INVALIDATION_EVENTS.has(event.type)) return;

	const documentId = payloadDocumentId(event);
	if (!documentId) return;

	if (REMOVE_ITEM_EVENTS.has(event.type)) {
		removeItemById(documentId);
		return;
	}

	if (hasBackendOwnedActiveFilter()) {
		scheduleRealtimeReset();
		return;
	}

	try {
		const { data, status } = await apiSdk.getDocumentEntry({ path: { document_id: documentId } });
		if (data) {
			if (!itemMatchesSimpleView(data)) {
				removeItemById(documentId);
				return;
			}
			const existing = items.some((item) => item.id === documentId);
			items = sortItems(
				existing ? items.map((item) => (item.id === documentId ? data : item)) : [data, ...items]
			);
			return;
		}
		// Only drop the item when the document is genuinely gone. A transient failure
		// (rate limit, server error, network) must not evict a still-valid entry.
		if (status === 404) {
			removeItemById(documentId);
		}
	} catch {
		// Network-level failure: leave the item in place; a later event reconciles it.
	}
}

async function loadPreferences(): Promise<void> {
	if (prefsLoaded) return;
	prefsLoaded = true;
	const result = await loadPreferencesSettings();
	if (result.success) {
		applyPreferences(result.data);
	}
}

function applyPreferences(prefs: PreferencesSettingsResponse): void {
	cachedPrefs = prefs;
	triageMode = prefs.workflow.triage_mode;
	triageTab = coerceTriageTabForMode(triageTab, triageMode);
	listDensity = prefs.layout.list_density;
	sidePanelOpen = prefs.layout.side_panel !== 'closed';
	sidebarMode = prefs.layout.sidebar_mode;
	sidebarSessionOverride = null;
	try {
		localStorage.setItem(SIDEBAR_MODE_KEY, prefs.layout.sidebar_mode);
	} catch {
		// localStorage unavailable (SSR or private mode)
	}
}

function toggleSidebarVisibility(currentlyVisible: boolean): void {
	sidebarSessionOverride = currentlyVisible ? 'collapsed' : 'expanded';
}

async function toggleSidePanel(): Promise<void> {
	sidePanelOpen = !sidePanelOpen;
	if (!cachedPrefs) return;
	const updated: PreferencesSettingsResponse = {
		...cachedPrefs,
		layout: { ...cachedPrefs.layout, side_panel: sidePanelOpen ? 'open' : 'closed' }
	};
	cachedPrefs = updated;
	await savePreferencesSettings(updated);
}

async function setSmartList(id: string | null): Promise<void> {
	if (id === smartListId) return;
	smartListId = id;
	viewPanelOpen = false;
	// A new view starts from its own saved rules, so nothing is carried over.
	draftTouched = false;

	if (id) {
		try {
			const resp = await apiSdk.getSmartList({ path: { id } });
			if (resp.data) {
				activeSmartList = resp.data as SmartListResponse;
				// Entering a different view is a deliberate context switch, so the bar
				// adopts that view's rules rather than keeping the previous view's.
				const parsed = parseFilterExpression(activeSmartList.filter_expression);
				draftConditions = parsed.conditions;
				draftConjunction = parsed.conjunction;
				// An already-open bar would keep rendering, and this view's rules only
				// survive parsing intact when the shape is flat.
				if (!isFlatFilterExpression(activeSmartList.filter_expression)) {
					filterBarOpen = false;
				}
			}
		} catch {
			activeSmartList = null;
		}
	} else {
		activeSmartList = null;
		draftConditions = [];
		draftConjunction = 'and';
	}
	await resetAndFetch();
}

function toggleFilterBar(): void {
	// An advanced view has no editable representation here; opening the bar would
	// show rules that are not the ones being queried.
	if (!filterBarOpen && smartListAdvanced) return;
	filterBarOpen = !filterBarOpen;
	if (filterBarOpen && draftConditions.length === 0) {
		draftConditions = [{ id: crypto.randomUUID(), field: 'tag', op: 'contains', value: '' }];
	}
}

function setDraftConditions(conds: FilterCondition[]): void {
	draftConditions = conds;
	draftTouched = true;
}

function setDraftConjunction(c: 'and' | 'or'): void {
	draftConjunction = c;
	draftTouched = true;
}

function getDraftFilterExpression(): FilterExpression {
	return draftExpressionOrNull() ?? buildFilterExpression([], draftConjunction);
}

function toggleViewPanel(): void {
	viewPanelOpen = !viewPanelOpen;
}

function toggleCountBadge(): void {
	showCountBadge = !showCountBadge;
	try {
		localStorage.setItem(SHOW_COUNT_BADGE_KEY, String(showCountBadge));
	} catch {
		// localStorage unavailable (SSR or private mode)
	}
}

async function triageAction(itemId: string, state: TriageTab): Promise<void> {
	const prev = items.find((i) => i.id === itemId);
	if (!prev) return;

	// Optimistic update — remove from current list since triage state changed
	items = items.filter((i) => i.id !== itemId);
	if (selectedId === itemId) {
		selectedId = null;
	}

	try {
		await apiSdk.triageLibraryEntry({
			path: { document_id: itemId },
			body: { state }
		});
	} catch {
		// Revert on failure
		items = sortItems([...items, prev]);
	}
}

async function deleteAction(itemId: string): Promise<void> {
	const prev = items.find((i) => i.id === itemId);
	if (!prev) return;

	// Optimistic removal; the entry is recoverable from Trash after the call.
	items = items.filter((i) => i.id !== itemId);
	if (selectedId === itemId) {
		selectedId = null;
	}

	try {
		await apiSdk.deleteLibraryEntry({ path: { document_id: itemId } });
		await getSidebar().refreshTrashCount();
	} catch {
		// Revert on failure
		items = sortItems([...items, prev]);
	}
}

async function markAllSeen(): Promise<void> {
	const snapshot = [...items];
	items = [];
	selectedId = null;
	await Promise.allSettled(
		snapshot.map((item) =>
			apiSdk.triageLibraryEntry({ path: { document_id: item.id }, body: { state: 'archive' } })
		)
	);
}

async function archiveAll(): Promise<void> {
	const snapshot = [...items];
	items = [];
	selectedId = null;
	await Promise.allSettled(
		snapshot.map((item) =>
			apiSdk.triageLibraryEntry({ path: { document_id: item.id }, body: { state: 'archive' } })
		)
	);
}

export function getLibrary() {
	return {
		get items() {
			return items;
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
		get triageTab() {
			return triageTab;
		},
		get activeType() {
			return activeType;
		},
		get sortOrder() {
			return sortOrder;
		},
		get selectedId() {
			return selectedId;
		},
		get selectedItem() {
			return selectedItem;
		},
		get isEmpty() {
			return isEmpty;
		},
		get fetchError() {
			return fetchError;
		},
		get triageMode() {
			return triageMode;
		},
		get listDensity() {
			return listDensity;
		},
		get sidePanelOpen() {
			return sidePanelOpen;
		},
		get sidebarMode() {
			return sidebarMode;
		},
		get sidebarSessionOverride() {
			return sidebarSessionOverride;
		},
		get smartListId() {
			return smartListId;
		},
		get activeSmartList() {
			return activeSmartList;
		},
		get smartListModified() {
			return smartListModified;
		},
		get smartListAdvanced() {
			return smartListAdvanced;
		},
		get filterBarOpen() {
			return filterBarOpen;
		},
		get draftConditions() {
			return draftConditions;
		},
		get draftConjunction() {
			return draftConjunction;
		},
		get viewPanelOpen() {
			return viewPanelOpen;
		},
		get showCountBadge() {
			return showCountBadge;
		},
		get groupBy() {
			return groupBy;
		},
		get readStatusTab() {
			return readStatusTab;
		},
		setTriageTab(tab: TriageTab) {
			triageTab = tab;
			resetAndFetch();
		},
		ensureTriageTabForMode(mode: TriageModeDto = triageMode) {
			const next = coerceTriageTabForMode(triageTab, mode);
			if (next === triageTab) return;
			triageTab = next;
			resetAndFetch();
		},
		setActiveType(type: string | undefined) {
			activeType = type;
			resetAndFetch();
		},
		setSortOrder(order: SortOrder) {
			sortOrder = order;
			try {
				localStorage.setItem(SORT_KEY, order);
			} catch {
				// localStorage unavailable (SSR or private mode)
			}
			items = sortItems([...items]);
		},
		setSelectedId(id: string | null) {
			selectedId = id;
		},
		updateItemInList(updated: DocumentListEntry): void {
			items = items.map((i) => (i.id === updated.id ? updated : i));
		},
		loadPreferences,
		applyPreferences,
		toggleSidebarVisibility,
		toggleSidePanel,
		setSmartList,
		toggleFilterBar,
		setDraftConditions,
		setDraftConjunction,
		getDraftFilterExpression,
		toggleViewPanel,
		toggleCountBadge,
		setGroupBy(gb: GroupBy) {
			groupBy = gb;
			resetAndFetch();
		},
		setReadStatusTab(tab: ReadStatusTab) {
			readStatusTab = tab;
		},
		resetAndFetch,
		loadMore,
		handleDomainEvent,
		triageAction,
		deleteAction,
		markAllSeen,
		archiveAll
	};
}
