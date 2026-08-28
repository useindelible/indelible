import { readdirSync, statSync } from 'node:fs';
import { join, relative, sep } from 'node:path';
import { describe, expect, it } from 'vitest';

import { FALLBACK_TITLE_KEY, routeTitleKey } from '$lib/stores/page-title.svelte';

const ROUTES_DIR = 'src/routes';

// Redirect-only page modules never paint content, so they carry no title of their own.
// A new redirect route must be added here deliberately.
const REDIRECT_ONLY = [
	'src/routes/+page.ts',
	'src/routes/(app)/onboarding/+page.ts',
	'src/routes/(app)/preferences/+page.ts',
	'src/routes/(app)/preferences/imports/+page.ts',
	'src/routes/(app)/preferences/notifications/+page.ts',
	'src/routes/(app)/preferences/profile/+page.ts',
	'src/routes/(app)/preferences/tokens/+page.ts'
];

const DYNAMIC_SAMPLES: Record<string, string> = {
	documentId: 'doc_1',
	id: 'col_1',
	slug: 'ada-lovelace'
};

function walk(dir: string): string[] {
	return readdirSync(dir).flatMap((entry) => {
		const full = join(dir, entry);
		return statSync(full).isDirectory() ? walk(full) : [full];
	});
}

function pageModules(): string[] {
	return walk(ROUTES_DIR)
		.filter((file) => /\+page\.(svelte|ts)$/.test(file))
		.map((file) => relative('.', file).split(sep).join('/'))
		.sort();
}

/** `src/routes/(app)/tags/[id]/+page.svelte` -> `/tags/col_1` */
function toPathname(module: string): string {
	const segments = module
		.replace(/^src\/routes/, '')
		.replace(/\/\+page\.(svelte|ts)$/, '')
		.split('/')
		.filter((segment) => segment !== '' && !/^\(.+\)$/.test(segment))
		.flatMap((segment) => {
			const optional = segment.match(/^\[\[(.+)\]\]$/);
			if (optional) return [];
			const required = segment.match(/^\[(.+)\]$/);
			if (required) return [DYNAMIC_SAMPLES[required[1]!] ?? 'sample'];
			return [segment];
		});
	return `/${segments.join('/')}`;
}

describe('routeTitleKey', () => {
	it.each([
		['/dashboard', 'library_nav_home'],
		['/library', 'common_library'],
		['/library/articles', 'library_nav_articles'],
		['/library/books', 'library_nav_books'],
		['/library/emails', 'library_nav_emails'],
		['/library/pdfs', 'library_nav_pdfs'],
		['/library/tweets', 'library_nav_tweets'],
		['/library/videos', 'library_nav_videos'],
		['/feed', 'common_feed'],
		['/search', 'common_search'],
		['/trash', 'common_trash'],
		['/reader/doc_1', 'reader_view_reader'],
		['/collections', 'library_collections'],
		['/collections/col_1', 'library_collections'],
		['/tags', 'common_tags'],
		['/tags/tag_1', 'common_tags'],
		['/entities/ada-lovelace', 'entity_page_title'],
		['/onboarding/welcome', 'onboarding_page_title'],
		['/preferences', 'settings_preferences'],
		['/preferences/account', 'settings_account'],
		['/preferences/integrations', 'settings_integrations'],
		['/preferences/integrations/notion', 'settings_notion_page_title'],
		['/preferences/integrations/obsidian', 'settings_obsidian_page_title'],
		['/login', 'auth_sign_in_title'],
		['/auth/callback', 'auth_callback_page_title'],
		['/extension/auth', 'extension_auth_page_title']
	])('maps %s to %s', (pathname, expected) => {
		expect(routeTitleKey(pathname)).toBe(expected);
	});

	it.each([
		['/feedback'],
		['/collections-archive'],
		['/login-help'],
		['/preferences-old'],
		['/searching'],
		['/librarian'],
		['/']
	])('falls back for the near-miss path %s', (pathname) => {
		expect(routeTitleKey(pathname)).toBe(FALLBACK_TITLE_KEY);
	});

	it('prefers the more specific rule over its prefix', () => {
		expect(routeTitleKey('/preferences/integrations/notion')).not.toBe(
			routeTitleKey('/preferences/integrations')
		);
		expect(routeTitleKey('/library/books')).not.toBe(routeTitleKey('/library'));
	});

	it('ignores query strings and trailing slashes', () => {
		expect(routeTitleKey('/library/books/')).toBe('library_nav_books');
	});
});

describe('route coverage', () => {
	it('lists only redirect-only modules that still exist', () => {
		const modules = pageModules();
		for (const exempt of REDIRECT_ONLY) expect(modules).toContain(exempt);
	});

	it('resolves a title for every page module that renders', () => {
		const uncovered = pageModules()
			.filter((module) => !REDIRECT_ONLY.includes(module))
			.map((module) => ({ module, pathname: toPathname(module) }))
			.filter(({ pathname }) => routeTitleKey(pathname) === FALLBACK_TITLE_KEY);

		expect(uncovered).toEqual([]);
	});
});
