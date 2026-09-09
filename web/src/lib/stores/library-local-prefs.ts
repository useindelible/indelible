import type { SidebarModeDto } from '$lib/api';

import type { SortOrder } from './library-query';

export const SORT_KEY = 'indelible_library_sort';
export const SIDEBAR_MODE_KEY = 'indelible_sidebar_mode';
export const SHOW_COUNT_BADGE_KEY = 'indelible_show_count_badge';

export function getInitialSort(): SortOrder {
	try {
		return (localStorage.getItem(SORT_KEY) as SortOrder) || 'date_saved_desc';
	} catch {
		return 'date_saved_desc';
	}
}

export function getInitialSidebarMode(): SidebarModeDto {
	try {
		return (localStorage.getItem(SIDEBAR_MODE_KEY) as SidebarModeDto) || 'expanded';
	} catch {
		return 'expanded';
	}
}

export function getInitialShowCountBadge(): boolean {
	try {
		const stored = localStorage.getItem(SHOW_COUNT_BADGE_KEY);
		return stored === null ? true : stored === 'true';
	} catch {
		return true;
	}
}
