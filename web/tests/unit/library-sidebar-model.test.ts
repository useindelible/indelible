import { describe, expect, it } from 'vitest';

import {
	getDefaultHomePath,
	getInitials,
	getSmartListHref,
	isSidebarPathActive
} from '$lib/components/library/library-sidebar-model';

describe('library sidebar model', () => {
	it('maps default app views to sidebar home paths', () => {
		expect(getDefaultHomePath('feed')).toBe('/feed');
		expect(getDefaultHomePath('search')).toBe('/search');
		expect(getDefaultHomePath('library')).toBe('/library');
		expect(getDefaultHomePath(undefined)).toBe('/library');
	});

	it('matches the library root exactly and nested sections by prefix', () => {
		expect(isSidebarPathActive('/library', '/library')).toBe(true);
		expect(isSidebarPathActive('/library/articles', '/library')).toBe(false);
		expect(isSidebarPathActive('/library/articles/saved', '/library/articles')).toBe(true);
		expect(isSidebarPathActive('/preferences/account', '/preferences')).toBe(true);
	});

	it('builds smart-list links from a resolved library path', () => {
		expect(getSmartListHref('sl_123', '/library')).toBe('/library?smart_list=sl_123');
	});

	it('uses the first two name parts for initials', () => {
		expect(getInitials('Ada Lovelace')).toBe('AL');
		expect(getInitials('Mila')).toBe('M');
		expect(getInitials('')).toBe('');
	});
});
