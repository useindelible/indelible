import { get } from 'svelte/store';
import { beforeAll, describe, expect, it } from 'vitest';

import { locale, setupI18nSync, t } from '$lib/i18n';
import en from '$lib/i18n/locales/en.json';
import fr from '$lib/i18n/locales/fr.json';
import { resolveTitle } from '$lib/stores/page-title.svelte';

describe('resolveTitle', () => {
	beforeAll(() => setupI18nSync({ en, fr }, 'en'));

	const base = { pathname: '/dashboard', errorStatus: null, override: null };
	const call = (input: Partial<typeof base> = {}) =>
		resolveTitle({ ...base, ...input, translate: get(t) });

	it('uses the route key when nothing overrides it', () => {
		expect(call()).toBe('Home');
	});

	it('falls back to the brand for an unmapped route', () => {
		expect(call({ pathname: '/nowhere' })).toBe('Indelible');
	});

	it('names the document when an override is present', () => {
		expect(call({ override: 'How to Read a Book' })).toBe('How to Read a Book');
	});

	it('prefers the error title over an override', () => {
		expect(call({ errorStatus: 404, override: 'How to Read a Book' })).toBe('Page not found');
	});

	it('distinguishes 404 from other error statuses', () => {
		expect(call({ errorStatus: 404 })).toBe('Page not found');
		expect(call({ errorStatus: 500 })).toBe('Something went wrong');
	});

	it.each([[null], [''], ['   '], ['\n\t ']])(
		'falls through to the route key for the blank override %j',
		(override) => {
			expect(call({ override })).toBe('Home');
		}
	);

	it('collapses whitespace inside a document name', () => {
		expect(call({ override: '  Multi \n  line   title  ' })).toBe('Multi line title');
	});

	it('does not truncate a long document name', () => {
		const name = 'A'.repeat(300);
		expect(call({ override: name })).toBe(name);
	});

	it('translates the route title for the active locale', () => {
		const translate = ((key, options) => get(t)(key, options)) as Parameters<
			typeof resolveTitle
		>[0]['translate'];
		expect(resolveTitle({ ...base, translate })).toBe('Home');
	});
});

describe('named detail titles', () => {
	beforeAll(() => setupI18nSync({ en, fr }, 'en'));

	it('composes a tag title from the tag name', async () => {
		await locale.set('en');
		expect(get(t)('tag_page_title_named', { values: { name: 'artificial intelligence' } })).toBe(
			'artificial intelligence · Tags'
		);
		await locale.set('fr');
		expect(get(t)('tag_page_title_named', { values: { name: 'intelligence artificielle' } })).toBe(
			'intelligence artificielle · Étiquettes'
		);
		await locale.set('en');
	});

	it('composes a collection title from the collection name', async () => {
		expect(get(t)('collection_page_title_named', { values: { name: 'Design Reading' } })).toBe(
			'Design Reading · Collections'
		);
	});
});
