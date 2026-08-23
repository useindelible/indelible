import * as api from '$lib/api';
import type {
	SmartListResponse,
	CreateSmartListBody,
	UpdateSmartListBody
} from '$lib/api/generated/types.gen';
import type { FilterExpression } from '$lib/utils/filter-expression';
import { fetchAllPages } from '$lib/api/pagination';
import { getSidebar } from './sidebar.svelte';
import { t, type MessageKey } from '$lib/i18n';
import { get } from 'svelte/store';

function message(key: MessageKey): string {
	return get(t)(key);
}

let allSmartLists = $state<SmartListResponse[]>([]);
let loading = $state(false);
let fetchError = $state<string | null>(null);

async function loadAllSmartLists(): Promise<void> {
	loading = true;
	fetchError = null;
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
		allSmartLists = results;
	} catch {
		fetchError = message('smart_list_error_load');
	} finally {
		loading = false;
	}
}

type CreateSmartListInput = Omit<CreateSmartListBody, 'filter_expression'> & {
	filter_expression: CreateSmartListBody['filter_expression'] | FilterExpression;
};

async function createSmartList(body: CreateSmartListInput): Promise<SmartListResponse | null> {
	try {
		const resp = await api.createSmartList({
			body: {
				...body,
				filter_expression: body.filter_expression as CreateSmartListBody['filter_expression']
			}
		});
		if (resp.data) {
			const created = resp.data as SmartListResponse;
			allSmartLists = [...allSmartLists, created];
			getSidebar().refreshSmartLists();
			return created;
		}
	} catch {
		fetchError = message('smart_list_error_create');
	}
	return null;
}

async function updateSmartList(
	id: string,
	body: UpdateSmartListBody
): Promise<SmartListResponse | null> {
	try {
		const resp = await api.updateSmartList({ path: { id }, body });
		if (resp.data) {
			const updated = resp.data as SmartListResponse;
			allSmartLists = allSmartLists.map((sl) => (sl.id === id ? updated : sl));
			getSidebar().refreshSmartLists();
			return updated;
		}
	} catch {
		fetchError = message('smart_list_error_update');
	}
	return null;
}

async function deleteSmartList(id: string): Promise<boolean> {
	try {
		await api.deleteSmartList({ path: { id } });
		allSmartLists = allSmartLists.filter((sl) => sl.id !== id);
		getSidebar().refreshSmartLists();
		return true;
	} catch {
		fetchError = message('smart_list_error_delete');
		return false;
	}
}

async function pinSmartList(id: string, isPinned: boolean): Promise<SmartListResponse | null> {
	try {
		const resp = await api.pinSmartList({ path: { id }, body: { is_pinned: isPinned } });
		if (resp.data) {
			const updated = resp.data as SmartListResponse;
			allSmartLists = allSmartLists.map((sl) => (sl.id === id ? updated : sl));
			getSidebar().refreshSmartLists();
			return updated;
		}
	} catch {
		fetchError = message('smart_list_error_pin');
	}
	return null;
}

export function getSmartLists() {
	return {
		get allSmartLists() {
			return allSmartLists;
		},
		get loading() {
			return loading;
		},
		get fetchError() {
			return fetchError;
		},
		loadAllSmartLists,
		createSmartList,
		updateSmartList,
		deleteSmartList,
		pinSmartList
	};
}
