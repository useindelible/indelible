import { locale, setupI18nSync } from '$lib/i18n';
import { beforeEach, describe, expect, it } from 'vitest';

import en from '../i18n/locales/en.json';
import fr from '../i18n/locales/fr.json';
import { formatReadingTime } from './format';

describe('formatReadingTime', () => {
	beforeEach(() => setupI18nSync({ en, fr }));

	it.each([
		[1, '1 min'],
		[45, '45 min'],
		[60, '1 h'],
		[61, '1 h 1 min']
	])('formats %i minutes', (minutes, expected) => {
		expect(formatReadingTime(minutes)).toBe(expected);
	});

	it('uses the active locale messages', () => {
		locale.set('fr');
		expect(formatReadingTime(61)).toBe('1 h 1 min');
	});
});
