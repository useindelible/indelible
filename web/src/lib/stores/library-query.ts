import type { DocumentListEntry, LibraryQueryBody, TriageModeDto } from '$lib/api';
import { t, type MessageKey } from '$lib/i18n';
import { get } from 'svelte/store';
import {
	buildFilterExpression,
	toApiFilterExpression,
	type FilterCondition,
	type FilterExpression
} from '$lib/utils/filter-expression';

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

export const SLUG_TO_API_TYPE: Record<string, string> = {
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

export function coerceTriageTabForMode(tab: TriageTab, mode: TriageModeDto): TriageTab {
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

export function sortItems(list: DocumentListEntry[], sortOrder: SortOrder): DocumentListEntry[] {
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

export function describeQueryError(error: unknown): string {
	if (error && typeof error === 'object') {
		const e = error as { errors?: Array<{ message?: string }>; detail?: string; title?: string };
		const first = e.errors?.[0]?.message;
		if (first) return first;
		if (e.detail && e.detail !== 'validation error') return e.detail;
		if (e.title) return e.title;
	}
	return get(t)('library_error_load_items');
}
