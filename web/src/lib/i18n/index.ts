import {
	addMessages,
	date,
	init,
	isLoading,
	locale,
	number,
	register,
	t as rawT,
	time,
	waitLocale
} from 'svelte-i18n';
import { derived, type Readable } from 'svelte/store';

import type en from './locales/en.json';

export type MessageKey = keyof typeof en;
export type TranslateOptions = {
	values?: Record<string, string | number | boolean | Date | null | undefined>;
};
export type Translate = (key: MessageKey, options?: TranslateOptions) => string;

const catalogLoaders = import.meta.glob<{ default: Record<string, string> }>('./locales/*.json');
let loadersRegistered = false;
let initialized = false;

function registerCatalogLoaders(): void {
	if (loadersRegistered) return;

	for (const [path, loader] of Object.entries(catalogLoaders)) {
		const tag = path.slice('./locales/'.length, -'.json'.length);
		register(tag, loader);
	}
	loadersRegistered = true;
}

export function setupI18n(initialLocale: string): void {
	registerCatalogLoaders();
	if (!initialized) {
		initialized = true;
		void init({ fallbackLocale: 'en', initialLocale });
		return;
	}
	void locale.set(initialLocale);
}

export function setupI18nSync(
	messages: Record<string, Record<string, string>>,
	initialLocale = 'en'
): void {
	for (const [tag, catalog] of Object.entries(messages)) {
		addMessages(tag, catalog);
	}
	initialized = true;
	void init({ fallbackLocale: 'en', initialLocale });
}

export const t: Readable<Translate> = derived(
	rawT,
	($t): Translate =>
		(key: MessageKey, options?: TranslateOptions) =>
			$t(key, options)
);

export { date, isLoading, locale, number, time, waitLocale };
export * from './app-locale';
export * from './locale-match';
