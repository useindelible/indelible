import { get } from 'svelte/store';
import { locale, waitLocale } from 'svelte-i18n';

import { FALLBACK_LOCALE, isRtl, matchLocale, systemLocale } from './locale-match';

export const LOCALE_STORAGE_KEY = 'ind.locale';

export function readStoredLocale(): string | null {
	try {
		return window.localStorage.getItem(LOCALE_STORAGE_KEY);
	} catch {
		return null;
	}
}

export function rememberExplicitLocale(tag: string): void {
	try {
		window.localStorage.setItem(LOCALE_STORAGE_KEY, tag);
	} catch {
		return;
	}
}

export function clearStoredLocale(): void {
	try {
		window.localStorage.removeItem(LOCALE_STORAGE_KEY);
	} catch {
		return;
	}
}

export function applyLocale(tag: string): void {
	const resolved = matchLocale(tag) ?? FALLBACK_LOCALE;
	if (get(locale) !== resolved) void locale.set(resolved);

	if (typeof document !== 'undefined') {
		document.documentElement.lang = resolved;
		document.documentElement.dir = isRtl(resolved) ? 'rtl' : 'ltr';
	}
}

export async function applyProfileLocale(profileLocale: string | null | undefined): Promise<void> {
	if (profileLocale === undefined) return;

	const explicit = matchLocale(profileLocale);
	const resolved = explicit ?? systemLocale();
	if (explicit) rememberExplicitLocale(explicit);
	else clearStoredLocale();

	applyLocale(resolved);
	await waitLocale(resolved);
}
