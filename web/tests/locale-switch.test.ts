import { get } from 'svelte/store';
import { beforeAll, describe, expect, it } from 'vitest';

import { setupI18nSync, t } from '$lib/i18n';
import en from '$lib/i18n/locales/en.json';
import fr from '$lib/i18n/locales/fr.json';
import {
	localeOptions,
	selectedLocaleValue
} from '../src/routes/(app)/preferences/reading-appearance/reading-appearance-model';

describe('language preference options', () => {
	beforeAll(() => setupI18nSync({ en, fr }, 'fr'));

	it('offers system language and supported explicit languages', () => {
		expect(localeOptions(get(t), ['fr'])).toEqual([
			{ value: '', label: 'Langue du système (Français)' },
			{ value: 'en', label: 'English' },
			{ value: 'fr', label: 'Français' }
		]);
	});

	it.each([
		['en-GB', 'en'],
		['fr-FR', 'fr'],
		[null, ''],
		['ja', '']
	])('normalises %s to %s', (profileLocale, expected) => {
		expect(selectedLocaleValue(profileLocale)).toBe(expected);
	});
});
