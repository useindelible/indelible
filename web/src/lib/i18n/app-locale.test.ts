import { get } from 'svelte/store';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { locale, setupI18nSync } from './index';
import {
	LOCALE_STORAGE_KEY,
	applyLocale,
	applyProfileLocale,
	clearStoredLocale,
	readStoredLocale,
	rememberExplicitLocale
} from './app-locale';

const messages = {
	en: { common_ok: 'OK' },
	fr: { common_ok: 'OK' }
};

describe('application locale state', () => {
	beforeEach(() => {
		window.localStorage.clear();
		setupI18nSync(messages);
		applyLocale('en');
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('applies a supported locale without persisting it', () => {
		applyLocale('fr-CA');

		expect(get(locale)).toBe('fr');
		expect(document.documentElement.lang).toBe('fr');
		expect(document.documentElement.dir).toBe('ltr');
		expect(readStoredLocale()).toBeNull();

		applyLocale('ar');
		expect(get(locale)).toBe('en');
	});

	it('stores and clears an explicit preference', () => {
		rememberExplicitLocale('fr');
		expect(window.localStorage.getItem(LOCALE_STORAGE_KEY)).toBe('fr');

		clearStoredLocale();
		expect(readStoredLocale()).toBeNull();
	});

	it('reconciles an explicit profile locale', async () => {
		await applyProfileLocale('fr-FR');

		expect(get(locale)).toBe('fr');
		expect(readStoredLocale()).toBe('fr');
	});

	it('clears stale storage when the profile follows the system locale', async () => {
		rememberExplicitLocale('en');
		vi.stubGlobal('navigator', { languages: ['fr'] });

		await applyProfileLocale(null);

		expect(get(locale)).toBe('fr');
		expect(readStoredLocale()).toBeNull();
	});

	it('does not reconcile an anonymous profile', async () => {
		await applyProfileLocale(undefined);
		expect(get(locale)).toBe('en');
	});
});
