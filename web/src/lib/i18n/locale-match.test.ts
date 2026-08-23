import { describe, expect, it } from 'vitest';

import {
	FALLBACK_LOCALE,
	isRtl,
	localeDisplayName,
	matchLocale,
	resolveInitialLocale,
	systemLocale
} from './locale-match';

const supported = ['en', 'fr'];

describe('locale matching', () => {
	it.each([
		['fr', 'fr'],
		['fr-CA', 'fr'],
		['en-GB', 'en'],
		['FR-fr', 'fr'],
		['ja', null],
		['!!', null],
		[null, null]
	] as const)('matches %s to %s', (tag, expected) => {
		expect(matchLocale(tag, supported)).toBe(expected);
	});

	it('uses the first supported system locale and otherwise falls back to English', () => {
		expect(systemLocale(['ja', 'fr-CA'])).toBe('fr');
		expect(systemLocale([])).toBe(FALLBACK_LOCALE);
	});

	it('prefers a stored locale over the system locale', () => {
		expect(resolveInitialLocale({ storedLocale: 'fr', navigatorLanguages: ['en'] })).toBe('fr');
		expect(resolveInitialLocale({ navigatorLanguages: ['fr'] })).toBe('fr');
		expect(resolveInitialLocale({ navigatorLanguages: ['ja'] })).toBe('en');
	});

	it('reports writing direction and a native display name', () => {
		expect(isRtl('ar')).toBe(true);
		expect(isRtl('he-IL')).toBe(true);
		expect(isRtl('fr')).toBe(false);
		expect(localeDisplayName('fr')).toBe('Français');
	});
});
